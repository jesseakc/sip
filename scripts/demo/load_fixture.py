#!/usr/bin/env python3
"""Load the SMRT test corpus into the demo tenant through SIP's migration pipeline.

Nothing here writes to the database directly: it drives the same public API the
Migration Studio plugin drives — create job, upload source records, validate,
dry-run, import, report — in dependency order, because the importer resolves
cross-references through the migration external-ID map and a work order cannot be
created before the asset it belongs to.

Dependency order (from data/smrt-maintenance-logs/fixture/manifest.json):

    location -> asset_type -> asset -> part -> work_order

Idempotent: each object type gets a deterministic job name, and a job that has
already completed successfully is skipped. ``--force`` re-imports anyway, which
in a fresh tenant would create duplicates — use ``provision.py --reset`` first if
you want to redo a load.

Secrets are read from ``~/sip-demo``; nothing sensitive is printed.
"""
from __future__ import annotations

import argparse
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO / "data" / "smrt-maintenance-logs" / "fixture"
CANONICAL = FIXTURE / "canonical"
DEMO_DIR = Path.home() / "sip-demo"
ACCOUNTS = DEMO_DIR / "accounts.json"
STATE = DEMO_DIR / "state.json"

LOAD_ORDER = ["location", "asset_type", "asset", "part", "work_order"]

# The report endpoint renders job status with Rust's Debug formatting, not the
# snake_case serde form, so these are the strings to match.
JOB_DONE = {"Completed", "CompletedWithWarnings"}

JOB_PREFIX = "SMRT fixture"


class Api:
    def __init__(self, base: str, timeout: int = 300) -> None:
        self.base = base.rstrip("/")
        self.timeout = timeout
        self.token: str | None = None

    def call(self, method: str, path: str, body=None, token: str | None = None,
             timeout: int | None = None):
        url = f"{self.base}{path}"
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(url, data=data, method=method)
        req.add_header("Content-Type", "application/json")
        tok = token or self.token
        if tok:
            req.add_header("Authorization", f"Bearer {tok}")
        try:
            with urllib.request.urlopen(req, timeout=timeout or self.timeout) as r:
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


def unwrap(body: dict):
    """SIP wraps responses as {"data": ...}; return the payload either way."""
    if isinstance(body, dict) and "data" in body:
        return body["data"]
    return body


def as_list(payload) -> list:
    if isinstance(payload, list):
        return payload
    if isinstance(payload, dict):
        for key in ("items", "records", "results", "data"):
            if isinstance(payload.get(key), list):
                return payload[key]
    return []


def read_fixture(verify: bool = True) -> list[dict]:
    """Load plan, taken from the fixture manifest and verified against it.

    The manifest written by ``build_fixture.py`` records each canonical file's
    record count and sha256. Checking the hash here means a stale or hand-edited
    fixture fails loudly instead of silently importing the wrong data.
    """
    import hashlib

    manifest_path = FIXTURE / "manifest.json"
    if not manifest_path.is_file():
        sys.exit(
            f"missing {manifest_path}.\n"
            "Generate it with:\n"
            "  ~/.venvs/sip-data/bin/python data/smrt-maintenance-logs/scripts/build_fixture.py"
        )

    manifest = json.loads(manifest_path.read_text())
    order = manifest.get("load_order", LOAD_ORDER)
    by_type = {
        f["object_type"]: f
        for f in manifest.get("files", [])
        if f.get("file", "").startswith("canonical/")
    }

    plan = []
    for i, object_type in enumerate(order, start=1):
        entry = by_type.get(object_type)
        if not entry:
            sys.exit(f"manifest has no canonical file for object type {object_type!r}")
        path = FIXTURE / entry["file"]
        if not path.is_file():
            sys.exit(f"missing fixture file {path}")

        payload = json.loads(path.read_text())
        records = payload.get("records", [])
        if verify:
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            if digest != entry.get("sha256"):
                sys.exit(
                    f"fixture checksum mismatch for {entry['file']}\n"
                    f"  manifest: {entry.get('sha256')}\n"
                    f"  actual:   {digest}\n"
                    "Regenerate the fixture rather than importing an unverified corpus."
                )
            if len(records) != entry.get("count"):
                sys.exit(
                    f"{entry['file']}: manifest says {entry.get('count')} records, "
                    f"file has {len(records)}"
                )
        plan.append(
            {
                "object_type": object_type,
                "file": entry["file"],
                "count": len(records),
                "order": i,
                "sha256_verified": verify,
            }
        )
    return plan


def existing_jobs(api: Api) -> dict[str, dict]:
    status, body = api.call("GET", "/migrations")
    if status != 200:
        return {}
    jobs = as_list(unwrap(body))
    by_name = {}
    for job in jobs:
        if isinstance(job, dict) and job.get("name"):
            by_name[job["name"]] = job
    return by_name


def job_status(api: Api, job_id: str) -> tuple[str, dict]:
    status, body = api.call("GET", f"/migrations/{job_id}/report")
    if status != 200:
        return f"report HTTP {status}", {}
    data = unwrap(body) or {}
    return (data.get("job", {}) or {}).get("status", "unknown"), data


def run_object(api: Api, entry: dict, chunk: int, import_timeout: int) -> dict:
    obj = entry["object_type"]
    path = FIXTURE / entry["file"]
    payload = json.loads(path.read_text())
    records = payload.get("records", [])
    print(f"\n── {obj}: {len(records)} records from {entry['file']}")

    status, body = api.call("POST", "/migrations", {
        "name": f"{JOB_PREFIX}: {obj}",
        "source_system": "SMRT Maintenance Logs (Hugging Face: ASNIJKDNI/SMRT-Maintenance-Logs)",
        "source_object_type": obj,
        "description": f"{len(records)} {obj} records from the SMRT maintenance-log fixture.",
    })
    if status not in (200, 201):
        sys.exit(f"  job creation failed: HTTP {status} {body}")
    data = unwrap(body) or {}
    job_id = data.get("id") or data.get("job_id")
    if not job_id:
        sys.exit(f"  job creation returned no id: {body}")
    print(f"  job {job_id}")

    # Transport keys: the importer reads external_id from the record and
    # source_object_type decides the target entity. No field mappings are sent —
    # a job with no mappings treats each record as already canonical.
    tagged = [
        {**rec, "source_object_type": obj} if "source_object_type" not in rec else dict(rec)
        for rec in records
    ]

    uploaded = 0
    for i in range(0, len(tagged), chunk):
        batch = tagged[i:i + chunk]
        status, body = api.call("POST", f"/migrations/{job_id}/source-records",
                                {"records": batch})
        if status not in (200, 201):
            sys.exit(f"  upload failed at offset {i}: HTTP {status} {body}")
        uploaded += len(batch)
        print(f"  uploaded {uploaded}/{len(tagged)}", end="\r", flush=True)
    print(f"  uploaded {uploaded}/{len(tagged)} records")

    for step, timeout in (("validate", None), ("dry-run", None), ("import", import_timeout)):
        started = time.time()
        status, body = api.call("POST", f"/migrations/{job_id}/{step}", {}, timeout=timeout)
        elapsed = time.time() - started
        if status not in (200, 201):
            sys.exit(f"  {step} failed: HTTP {status} {body}")
        print(f"  {step:<9} ok in {elapsed:.1f}s")

    status, report = job_status(api, job_id)
    results = report.get("import_results", {})
    print(f"  status={status} created={results.get('created')} "
          f"skipped={results.get('skipped')} failed={results.get('failed')} "
          f"external_id_maps={report.get('external_id_maps_count')}")

    return {
        "object_type": obj,
        "job_id": job_id,
        "source_records": len(records),
        "status": status,
        "import_results": results,
        "validation_issues": report.get("validation_issues", {}),
        "external_id_maps_count": report.get("external_id_maps_count"),
    }


def count_entities(api: Api) -> dict:
    counts = {}
    for label, path in (("locations", "/locations"), ("assets", "/assets"),
                        ("parts", "/parts"), ("work_orders", "/work-orders")):
        status, body = api.call("GET", path)
        counts[label] = len(as_list(unwrap(body))) if status == 200 else f"HTTP {status}"
    return counts


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--base-url", default="http://localhost:8000/api/v1")
    ap.add_argument("--chunk", type=int, default=500)
    ap.add_argument("--import-timeout", type=int, default=3600)
    ap.add_argument("--force", action="store_true", help="re-import even if a job already completed")
    args = ap.parse_args()

    if not ACCOUNTS.is_file():
        sys.exit(f"missing {ACCOUNTS}; run scripts/demo/gen_env.py then provision.py")
    accounts = json.loads(ACCOUNTS.read_text())
    admin = next(a for a in accounts["accounts"] if a["role"] == "ADMIN")

    api = Api(args.base_url)
    token = api.login(admin["email"], admin["password"])
    if not token:
        sys.exit(f"could not sign in as {admin['email']}; run provision.py first")
    api.token = token
    print(f"signed in as the demo administrator ({admin['email']})")

    plan = read_fixture()
    expected = {d["object_type"]: d["count"] for d in plan}
    print(f"load plan: " + " -> ".join(f"{d['object_type']}({d['count']})" for d in plan))

    done = existing_jobs(api)
    results = []
    for entry in plan:
        name = f"{JOB_PREFIX}: {entry['object_type']}"
        prior = done.get(name)
        if prior and not args.force:
            jid = prior.get("id") or prior.get("job_id")
            status, report = job_status(api, jid) if jid else ("unknown", {})
            if status in JOB_DONE:
                created = (report.get("import_results") or {}).get("created", "?")
                print(f"\n── {entry['object_type']}: already imported "
                      f"(job {jid}, {status}, created={created}) — skipping")
                results.append({"object_type": entry["object_type"], "job_id": jid,
                                "source_records": entry["count"], "status": status,
                                "import_results": report.get("import_results", {}),
                                "reused": True})
                continue
        results.append(run_object(api, entry, args.chunk, args.import_timeout))

    print("\n" + "=" * 68)
    print("IMPORT SUMMARY")
    print("=" * 68)
    print(f"{'object':<12} {'expected':>9} {'created':>9} {'skipped':>9} {'failed':>8}  status")
    for r in results:
        ir = r.get("import_results") or {}
        print(f"{r['object_type']:<12} {expected.get(r['object_type'], '?'):>9} "
              f"{str(ir.get('created', '?')):>9} {str(ir.get('skipped', '?')):>9} "
              f"{str(ir.get('failed', '?')):>8}  {r.get('status')}")

    counts = count_entities(api)
    print("\nENTITY COUNTS IN THE DEMO TENANT (actual vs expected)")
    print(f"{'entity':<14} {'actual':>9} {'expected':>9}  match")
    expected_counts = {
        "locations": expected.get("location"),
        "assets": expected.get("asset"),
        "parts": expected.get("part"),
        "work_orders": expected.get("work_order"),
    }
    for key, exp in expected_counts.items():
        actual = counts.get(key)
        match = "yes" if actual == exp else "NO"
        print(f"{key:<14} {str(actual):>9} {str(exp):>9}  {match}")

    state = json.loads(STATE.read_text()) if STATE.is_file() else {}
    state["fixture"] = {
        "source": "ASNIJKDNI/SMRT-Maintenance-Logs",
        "expected": expected,
        "jobs": results,
        "entity_counts": counts,
    }
    STATE.write_text(json.dumps(state, indent=2) + "\n")
    print(f"\nstate written to {STATE}")


if __name__ == "__main__":
    main()
