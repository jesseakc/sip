# SIP family-of-brands distribution framework

Priority: P1
Status: planning only

## Scope
Industry manifests for theme, navigation, terminology, enabled plugins, domains and Sippy variants; demonstrate Property and Automotive using same core.

## Acceptance
- [ ] Extend existing SIP crates without duplicating architecture
- [ ] Support independent commercial operators and optional cloud providers
- [ ] Include local, hosted and hybrid configuration documentation
- [ ] Include tests for tenant isolation, security, provider failures and upgrades
- [ ] Preserve backwards compatibility and document migrations

## Repository boundary (explicit owner decision)
This repository contains SIP Core, APIs, infrastructure, deployment automation, provider adapters, plugin architecture, identity/tenant/brand configuration, and technical documentation only. The public marketing/landing website and all industry-specific marketing websites will be developed in separate repositories. Do not create marketing pages, marketing Next.js starters, landing page templates, SEO/content tooling, or marketing UI here. Existing SIP application frontend remains in scope as product software. This repository may expose brand configuration, authentication interfaces, documented APIs, domain/CORS integration contracts, and deployment examples needed by external website repositories. No website implementation should be added to this repository.

Brand distribution manifests and product application UI configuration remain in scope. Marketing sites, public landing pages, SEO and website templates are out of scope.
