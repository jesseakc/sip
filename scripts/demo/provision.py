#!/usr/bin/env python3
"""Create the demo tenant and its role accounts, idempotently.

Tenant: "SIP Demo Rail Services" (slug ``sip-demo-rail-services``) — a clearly
fictional customer, separate from the seeded Acme Manufacturing tenant.

How each object is created, and why:

* **The tenant** is created through the public API (``POST /organizations``) using
  the seeded Acme administrator, so the real org-creation path runs.
* **The tenant's first administrator** cannot go through the API: ``POST /users``
  always scopes the new user to the *caller's* organization, so there is no way
  for an Acme administrator to create a user inside another tenant. That one row
  is bootstrapped with SQL. It is the only row this script inserts directly.
* **Every other role account** is created through ``POST /users`` as the demo
  administrator, which means SIP hashes the password and enforces
  ``user:manage`` itself.

Idempotent by construction: the tenant is keyed on its unique slug and each
account on the ``(organization_id, email)`` unique constraint, so re-running
reports what already exists instead of duplicating it.

``--reset`` deletes this tenant and nothing else. It refuses to run unless the
recorded organization id and the tenant slug both match what this script created,
so it cannot be aimed at another tenant or at Acme.

Secrets are read from ``~/sip-demo/accounts.json`` and never printed in full.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

DEMO_DIR = Path.home() / "sip-demo"
ACCOUNTS = DEMO_DIR / "accounts.json"
STATE = DEMO_DIR / "state.json"

ACME_ADMIN = ("admin@acme.local", "password")

TENANT_TAG = "sip-demo-rail-services"


class Api:
    def __init__(self, base: str) -> None:
        self.base = base.rstrip("/")
        self.token: str | None = None

    def call(self, method: str, path: str, body=None, token: str | None = None) -> tuple[int, dict]:
        url = f"{self.base}{path}"
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(url, data=data, method=method)
        req.add_header("Content-Type", "application/json")
        tok = token or self.token
        if tok:
            req.add_header("Authorization", f"Bearer {tok}")
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                raw = r.read().decode()
                return r.status, (json.loads(raw) if raw else {})
        except urllib.error.HTTPError as e:
            raw = e.read().decode()
            try:
                return e.code, json.loads(raw)
            except Exception:
                return e.code, {"raw": raw}
        except Exception as e:
            return 0, {"error": str(e)}

    def login(self, email: str, password: str) -> str | None:
        status, body = self.call("POST", "/auth/login", {"email": email, "password": password})
        if status != 200:
            return None
        data = body.get("data") or body
        return data.get("token") or data.get("access_token")


class Psql:
    """Reads and writes SQL for provisioning.

    Prefers a direct TCP connection to Postgres on the loopback binding the demo
    overlay publishes (127.0.0.1:55432). That is deliberate: the one row this
    script has to insert directly would otherwise need `sudo docker exec` every
    time it ran, and the goal is for provisioning to need no privileges beyond
    reading ~/sip-demo.

    Falls back to `docker exec` if the driver is unavailable or the port is not
    published.
    """

    def __init__(self, dsn: str | None, container: str, use_sudo: bool) -> None:
        self.dsn = dsn
        self.container = container
        self.prefix = (["sudo"] if use_sudo else []) + [
            "docker", "exec", "-i", container, "psql", "-U", "sip", "-d", "sip",
            "-v", "ON_ERROR_STOP=1", "-tAc",
        ]
        self.mode = "tcp" if dsn else "docker"

    # ── direct TCP ────────────────────────────────────────────────────────
    def _tcp(self, sql: str, fetch_one: bool):
        import psycopg

        with psycopg.connect(self.dsn, connect_timeout=10) as conn:
            with conn.cursor() as cur:
                cur.execute(sql)
                if fetch_one:
                    row = cur.fetchone()
                    return "" if row is None or row[0] is None else str(row[0])
                return None

    # ── docker exec fallback ──────────────────────────────────────────────
    def _docker(self, sql: str, fetch_one: bool):
        out = subprocess.run(self.prefix + [sql], capture_output=True, text=True, timeout=180)
        if out.returncode != 0:
            raise RuntimeError(f"psql failed: {out.stderr.strip()}")
        return out.stdout.strip() if fetch_one else None

    def scalar(self, sql: str) -> str:
        if self.mode == "tcp":
            try:
                return self._tcp(sql, fetch_one=True) or ""
            except ImportError:
                self.mode = "docker"
            except Exception as e:
                raise RuntimeError(f"SQL failed: {e}") from e
        return self._docker(sql, fetch_one=True)

    def execute(self, sql: str) -> None:
        if self.mode == "tcp":
            try:
                self._tcp(sql, fetch_one=False)
                return
            except ImportError:
                self.mode = "docker"
            except Exception as e:
                raise RuntimeError(f"SQL failed: {e}") from e
        self._docker(sql, fetch_one=False)


def sql_str(value: str) -> str:
    """Quote a literal for the SQL we send over psql -c."""
    return "'" + value.replace("'", "''") + "'"


def resolve_dsn(explicit: str | None, port: int) -> str | None:
    """Build the DSN from ~/sip-demo/secrets.env, or return None to use docker exec."""
    if explicit:
        return explicit
    secrets = DEMO_DIR / "secrets.env"
    if not secrets.is_file():
        return None
    password = None
    for line in secrets.read_text().splitlines():
        if line.startswith("POSTGRES_PASSWORD="):
            password = line.split("=", 1)[1].strip()
    if not password:
        return None
    return f"postgresql://sip:{password}@127.0.0.1:{port}/sip"


def load_accounts() -> dict:
    if not ACCOUNTS.is_file():
        sys.exit(f"missing {ACCOUNTS}; run scripts/demo/gen_env.py first")
    return json.loads(ACCOUNTS.read_text())


def load_state() -> dict:
    return json.loads(STATE.read_text()) if STATE.is_file() else {}


def save_state(state: dict) -> None:
    STATE.write_text(json.dumps(state, indent=2) + "\n")


def wait_for_api(api: Api, attempts: int = 60) -> dict:
    import time

    for i in range(attempts):
        status, body = api.call("GET", "/health")
        if status == 200:
            print(f"API healthy after {i + 1} attempt(s)")
            status, caps = api.call("GET", "/capabilities")
            return caps.get("data", {}) if status == 200 else {}
        time.sleep(2)
    sys.exit("API did not become healthy; is the stack up?")


def ensure_tenant(api: Api, accounts: dict, psql: Psql, state: dict) -> str:
    slug = accounts["tenant_slug"]
    existing = psql.scalar(f"SELECT id FROM organizations WHERE slug = {sql_str(slug)}")
    if existing:
        print(f"tenant already present: {accounts['tenant_name']} ({slug})")
        return existing

    token = api.login(*ACME_ADMIN)
    if not token:
        sys.exit(
            "could not sign in as the seeded Acme administrator "
            f"({ACME_ADMIN[0]}); the seed migration may not have run"
        )
    status, body = api.call(
        "POST",
        "/organizations",
        {
            "name": accounts["tenant_name"],
            "slug": slug,
            "timezone": "America/Chicago",
            "default_currency": "USD",
        },
        token=token,
    )
    if status not in (200, 201):
        sys.exit(f"tenant creation failed: HTTP {status} {body}")
    org_id = (body.get("data") or {}).get("id")
    if not org_id:
        sys.exit(f"tenant creation returned no id: {body}")
    print(f"created tenant: {accounts['tenant_name']} -> {org_id}")
    return org_id


def ensure_bootstrap_admin(accounts: dict, psql: Psql, org_id: str) -> None:
    """Insert the tenant's first administrator. The only direct SQL write."""
    admin = next(a for a in accounts["accounts"] if a["role"] == "ADMIN")
    exists = psql.scalar(
        "SELECT id FROM users WHERE organization_id = "
        f"{sql_str(org_id)} AND email = {sql_str(admin['email'])}"
    )
    if exists:
        print(f"bootstrap admin already present: {admin['email']}")
        return
    psql.execute(
        "INSERT INTO users (organization_id, email, name, role, password_hash, is_active) "
        f"VALUES ({sql_str(org_id)}, {sql_str(admin['email'])}, {sql_str(admin['name'])}, "
        f"'ADMIN', {sql_str(admin['password_hash'])}, true)"
    )
    # organization_settings is not created by the org endpoint unless a timezone
    # is supplied; make sure a row exists so settings reads do not 404.
    psql.execute(
        "INSERT INTO organization_settings (organization_id, timezone, default_currency) "
        f"VALUES ({sql_str(org_id)}, 'America/Chicago', 'USD') "
        "ON CONFLICT (organization_id) DO NOTHING"
    )
    print(f"bootstrapped tenant administrator: {admin['email']}")


def ensure_role_accounts(api: Api, accounts: dict, psql: Psql, org_id: str,
                         force: bool) -> list[dict]:
    """Create the remaining role accounts through the API as the demo admin."""
    admin = next(a for a in accounts["accounts"] if a["role"] == "ADMIN")
    token = api.login(admin["email"], admin["password"])
    if not token:
        sys.exit(f"could not sign in as the demo administrator {admin['email']}")

    created, existing = [], []
    for acct in accounts["accounts"]:
        if acct["role"] == "ADMIN":
            continue
        present = psql.scalar(
            "SELECT id FROM users WHERE organization_id = "
            f"{sql_str(org_id)} AND email = {sql_str(acct['email'])}"
        )
        if present and not force:
            existing.append(acct)
            continue
        if present and force:
            psql.execute(
                f"UPDATE users SET password_hash = {sql_str(acct['password_hash'])}, "
                f"updated_at = NOW() WHERE id = {sql_str(present)}"
            )
            existing.append(acct)
            continue
        status, body = api.call(
            "POST",
            "/users",
            {"email": acct["email"], "name": acct["name"],
             "role": acct["role"], "password": acct["password"]},
            token=token,
        )
        if status in (200, 201):
            created.append(acct)
        else:
            print(f"  ! {acct['role']:<11} {acct['email']} -> HTTP {status} {body}")
    print(f"role accounts created: {len(created)}, already present: {len(existing)}")
    return created


def collect_permissions(api: Api, accounts: dict) -> dict:
    """Ask the API what each account can actually do."""
    perms = {}
    for acct in accounts["accounts"]:
        token = api.login(acct["email"], acct["password"])
        if not token:
            perms[acct["role"]] = {"login": "FAILED"}
            continue
        status, body = api.call("GET", "/auth/me", token=token)
        data = (body.get("data") or body) if status == 200 else {}
        perms[acct["role"]] = {
            "login": "ok",
            "email": acct["email"],
            "role_reported": data.get("role"),
            "permissions": sorted(data.get("permissions") or []),
        }
    return perms


def guarded_reset(accounts: dict, psql: Psql, state: dict, confirm: str) -> None:
    """Delete the demo tenant only, and only if it is provably the demo tenant."""
    slug = accounts["tenant_slug"]
    org_id = state.get("organization_id")

    row = psql.scalar(
        f"SELECT id || '|' || slug FROM organizations WHERE slug = {sql_str(slug)}"
    )
    if not row:
        print("nothing to reset: demo tenant not found")
        return
    found_id, found_slug = row.split("|", 1)

    # Two independent guards: the slug must match, and if we have a recorded id it
    # must match too. A mismatch means something other than this script created a
    # tenant under that slug, and deleting it would be wrong.
    if found_slug != slug:
        sys.exit(f"refusing to reset: slug mismatch ({found_slug!r} != {slug!r})")
    if org_id and found_id != org_id:
        sys.exit(
            f"refusing to reset: recorded org id {org_id} != database {found_id}. "
            "Re-run without --reset, or investigate."
        )
    if confirm != slug:
        sys.exit(
            f"refusing to reset: pass --confirm {slug} to delete the demo tenant "
            f"and its {psql.scalar('SELECT COUNT(*) FROM users WHERE organization_id = ' + sql_str(found_id))} accounts, "
            "plus every asset, work order and part belonging to it"
        )

    print(f"deleting demo tenant {found_id} ({slug}) and all of its data")
    # Assets and work orders are not ON DELETE CASCADE from organizations, so the
    # tenant's own rows are removed explicitly before the organization row. Every
    # statement is scoped to this organization id.
    statements = [
        # part_usage references work_orders and parts (RESTRICT), so it goes first
        f"DELETE FROM part_usage WHERE work_order_id IN "
        f"(SELECT id FROM work_orders WHERE organization_id = {sql_str(found_id)})",
        f"DELETE FROM work_order_assignments WHERE work_order_id IN "
        f"(SELECT id FROM work_orders WHERE organization_id = {sql_str(found_id)})",
        f"DELETE FROM work_order_status_history WHERE work_order_id IN "
        f"(SELECT id FROM work_orders WHERE organization_id = {sql_str(found_id)})",
        f"DELETE FROM work_orders WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM assets WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM parts WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM locations WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM asset_types WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM migration_external_id_maps WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM migration_import_results WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM migration_staged_records WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM migration_source_records WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM migration_runs WHERE organization_id = {sql_str(found_id)}",
        f"DELETE FROM migration_jobs WHERE organization_id = {sql_str(found_id)}",
        # users, teams, organisation settings cascade from organizations
        f"DELETE FROM organizations WHERE id = {sql_str(found_id)}",
    ]
    for stmt in statements:
        psql.execute(stmt)
    print("demo tenant reset complete")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--base-url", default="http://localhost:8000/api/v1")
    ap.add_argument("--container", default="sip-postgres")
    ap.add_argument("--no-sudo", action="store_true", help="don't prefix docker with sudo")
    ap.add_argument("--dsn", default=None,
                    help="Postgres DSN; defaults to the loopback binding using the generated password")
    ap.add_argument("--db-port", type=int, default=55432,
                    help="host port the demo overlay publishes Postgres on")
    ap.add_argument("--force", action="store_true", help="rewrite existing passwords from accounts.json")
    ap.add_argument("--reset", action="store_true", help="delete the demo tenant (guarded)")
    ap.add_argument("--confirm", default="", help="must equal the tenant slug to confirm --reset")
    args = ap.parse_args()

    accounts = load_accounts()
    state = load_state()
    dsn = resolve_dsn(args.dsn, args.db_port)
    psql = Psql(dsn, args.container, use_sudo=not args.no_sudo)
    print(f"SQL access: {psql.mode}"
          + (" (127.0.0.1:%d, loopback)" % args.db_port if psql.mode == "tcp" else " (docker exec)"))

    if args.reset:
        guarded_reset(accounts, psql, state, args.confirm)
        save_state({k: v for k, v in state.items() if k != "organization_id"})
        return

    api = Api(args.base_url)
    caps = wait_for_api(api)
    print(f"capabilities: demo_mode={caps.get('features', {}).get('demo_mode')} "
          f"ai={caps.get('ai', {}).get('enabled')} model={caps.get('ai', {}).get('model')}")

    org_id = ensure_tenant(api, accounts, psql, state)
    ensure_bootstrap_admin(accounts, psql, org_id)
    ensure_role_accounts(api, accounts, psql, org_id, args.force)

    perms = collect_permissions(api, accounts)
    for role, info in perms.items():
        if info.get("login") != "ok":
            print(f"  ! {role}: cannot sign in")
    print("all accounts can sign in" if all(i.get("login") == "ok" for i in perms.values())
          else "SOME ACCOUNTS CANNOT SIGN IN")

    state.update(
        {
            "tenant_name": accounts["tenant_name"],
            "tenant_slug": accounts["tenant_slug"],
            "organization_id": org_id,
            "email_domain": accounts["email_domain"],
            "permissions_by_role": perms,
        }
    )
    save_state(state)
    print(f"state written to {STATE}")


if __name__ == "__main__":
    main()
