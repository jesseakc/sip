# Pluggable object storage providers

Priority: P1
Status: planning only

## Scope
Wire existing storage into document upload/download; local MinIO and managed S3-compatible providers, tenant-scoped keys, signed URLs, encryption, retention and migration.

## Acceptance
- [ ] Extend existing SIP crates without duplicating architecture
- [ ] Support independent commercial operators and optional cloud providers
- [ ] Include local, hosted and hybrid configuration documentation
- [ ] Include tests for tenant isolation, security, provider failures and upgrades
- [ ] Preserve backwards compatibility and document migrations
