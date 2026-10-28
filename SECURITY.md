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

## Not a vulnerability

- The demo tenant created by early migrations being present after a *partial* migration
  run is a known operational hazard; run all migrations. (See the migration notes.)
- Public repository metadata such as the maintainer's GitHub username.
