# Security policy

## Reporting a vulnerability

Please do **not** open a public issue for an unpatched vulnerability.

Report privately through GitHub's advisory flow:
**Security → Advisories → Report a vulnerability** on this repository's main page.

Include what you can: affected version or commit, a reproduction, and the impact you
believe it has. If you cannot use the advisory flow, open a minimal issue that says only
that you have a security report and how to reach you — no details.

## What to expect

- Acknowledgement within a few days.
- A private channel for the details and for coordinating a fix.
- Credit in the advisory if you want it.

## Scope

SIP is a multi-tenant operations platform. Reports about cross-tenant data access,
authentication or session handling, authorization bypass, and credential exposure are
especially welcome.

## Known risk: partially applied migrations

SIP's schema and data migrations are ordered steps, and a database that stops part-way
through is **not** a safe intermediate state.

Early migrations seed a demonstration tenant with accounts whose passwords are written in
the migration files and therefore public. A later migration removes that tenant. A database
that has run the seeding migration but has not reached the removal migration still contains
eight accounts with known credentials.

**That is a vulnerability in any environment reachable by anyone else.** An attacker does not
need to exploit anything: the credentials are in the repository. Treating it as "not a
vulnerability, just an incomplete upgrade" is incorrect, and this policy does not do that.

### Required upgrade procedure

1. Run **every** migration to completion before the service accepts traffic.
2. After upgrading, verify the seeded accounts are gone — query the users table for the
   seeded addresses and expect zero rows.
3. If a database was ever served while it held those accounts, treat its data as
   potentially accessed, rotate the credentials the accounts could reach, and rotate the
   token-signing secret. Removing the accounts does not undo sessions or data already
   obtained with them.
4. Do not restore a dump taken before the removal migration into a reachable environment
   without repeating step 2 first.
5. A failed migration run must leave the service **down**, not serving from a partially
   migrated schema.

## Not a vulnerability

- Public repository metadata, such as the maintainer's GitHub username or the repository URL.
- A demo value that is synthetic, published intentionally, and cannot grant access to a
  real system. If you are unsure which category a value falls into, report it.
