# Background jobs, outbox and notification providers

Priority: P1
Status: planning only

## Scope
Wire sip-outbox, optional workers, durable jobs, PostgreSQL local fallback, optional Redis, SMTP and hosted email/SMS adapters. Retry/idempotency/dead-letter tests.

## Acceptance
- [ ] Extend existing SIP crates without duplicating architecture
- [ ] Support independent commercial operators and optional cloud providers
- [ ] Include local, hosted and hybrid configuration documentation
- [ ] Include tests for tenant isolation, security, provider failures and upgrades
- [ ] Preserve backwards compatibility and document migrations
