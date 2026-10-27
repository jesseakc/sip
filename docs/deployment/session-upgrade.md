# Session and authorization upgrade

Run migrations before starting this build. JWTs now require an explicit ACCESS or REFRESH purpose and a user auth_epoch. Existing tokens are intentionally rejected; users must sign in again after deployment. Access tokens cannot renew sessions, and refresh tokens cannot authenticate API calls.

Disabled users are rejected at sign-in, renewal, and on protected requests. Protected requests also reject tokens whose user role changed. Logout advances auth_epoch and revokes all of the user's existing access and refresh tokens, across devices. The frontend calls the revocation endpoint before removing its token and reports revocation failures.

User creation and role administration require administrator wildcard access. Managers no longer receive user:manage. Generic work-order transitions require the review and cancellation permissions used by dedicated routes.

This PR does not add per-device sessions, refresh-token rotation/replay detection, automatic frontend refresh, tenant selection at login, captcha, or MFA. Ambiguous email matches across tenants fail closed; a tenant-aware login flow remains a follow-up. The existing pool/RLS connection issue remains a release blocker and is not resolved by session validation.
