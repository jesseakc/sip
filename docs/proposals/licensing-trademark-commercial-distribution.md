# SIP Licensing, Trademark, and Commercial Distribution Policy

**Priority:** P0 | **Status:** Proposal for implementation and legal review

## Governing principles
1. SIP Core remains AGPL-3.0 licensed. Independent operators may use, modify, redistribute, host, and commercially sell services built on SIP, including services competing with official SIP Cloud, subject to applicable license obligations.
2. No technical or commercial dependency on official SIP Cloud is required to run an independent instance.
3. The official SIP name, logo, Sippy mascot, and SIP family-of-brands identities have a separately documented brand-use policy. Do not suggest copyright licensing automatically establishes or registers trademarks.
4. White-label distributions may use independently created branding. Preserve legally required copyright, license, source, and attribution notices even when the UI is white-labeled.
5. Official SIP Cloud software features should remain reproducible from publicly available source; optional managed operations and support may be charged for.

## Requested deliverables
- [ ] Add `docs/legal/COMMERCIAL_USE.md` explaining AGPLv3 rights and obligations for self-hosting, modifying, redistribution, SaaS/network interaction, conveying source, copyright notices, and upstream modifications; link to the canonical AGPLv3 text and existing LICENSE.
- [ ] Add `TRADEMARKS.md` as a **proposed policy**, distinguishing permitted truthful nominative references, community references, compatibility statements, and prohibited suggestions of official affiliation or endorsement. Explicitly clarify which marks are actually claimed/registered and who owns them; do not assert registration without evidence.
- [ ] Add `docs/legal/WHITE_LABEL.md` with operator rebranding guidance, notices that must remain, and optional brand asset replacement. No forced phone-home or mandatory central identity.
- [ ] Add `docs/legal/PLUGIN_LICENSING.md` explaining first-party and third-party plugin license choices, distribution and linking considerations, network-copyleft analysis, third-party notices, dependencies and marketplace disclosures. Do not categorically promise proprietary plugins are compatible with AGPLv3.
- [ ] Add `docs/legal/COMMERCIAL_OPERATOR_CHECKLIST.md` for independent SaaS operators: source offer/access, modified source, notices, brand substitution, privacy, security, and applicable commercial/legal responsibilities.
- [ ] Update README and contributor docs to link these policies and distinguish software licensing from trademark permission.
- [ ] Review current repository license headers, dependencies, generated artifacts and marketing statements for consistency. Avoid changing the LICENSE or relicensing existing contributions without appropriate authorization.
- [ ] Include a legal-review checklist for counsel before publishing binding trademark terms or definitive interpretations of complex AGPL/plugin scenarios.

## Acceptance criteria
- Documentation explicitly confirms competing commercial SIP hosting is permitted under AGPLv3 conditions.
- No implication that users need permission or a paid SIP license to commercially use AGPL software.
- No claim that AGPL automatically grants official logo/mascot/trademark rights.
- No assertion that white-labeling permits stripping mandatory legal notices.
- No blanket assurance that closed-source plugins are AGPL-compatible.
- Independent commercial operators have a clear, actionable compliance checklist.
- Policy text is reviewed for consistency with the actual repository LICENSE and existing dependencies.
- This PR is documentation planning only, not legal advice or an already-adopted trademark policy.

## Dependencies
Coordinate with reproducible SIP distribution, operator/tenant/brand isolation, plugin marketplace, white-label marketing starter and installer wizard proposals.
