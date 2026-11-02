# DIST-004: AWS Marketplace and Salesforce AgentExchange

**Status:** Implementation proposal. No seller enrollment, commercial offer, paid subscription, security-review submission, or public listing is created.
**Checked:** 2026-11-02. **SIP baseline:** `c3ad101ab5b60ca6cce3d23277bf8ce7a37c35ca`.

## Common architecture

Expose the shared #5 tools through one versioned service contract. Marketplace fulfillment, buyer identity, runtime authorization, and listing metadata need separate adapters. Reuse the distribution-readiness evidence model; do not fork SIP's maintenance logic or equate a marketplace entitlement with permission to read another customer's tenant.

Two small marketplace-specific implementation PRs may follow this proposal. Neither platform's success counts as the other's validation.

## AWS Marketplace

The seller guide supports SaaS API-based agents/tools with MCP endpoints, configurable authentication and fulfillment. It documents limited-visibility testing and a free-product path. Free publication is possible but is not an automatic choice: the documented free conversion has commercial constraints, and the operator must review the intended offer before any irreversible pricing action.

Proposed slices:

1. Confirm seller eligibility, product type, licensing/support obligations, AWS account ownership, and the selected free/paid offer.
2. Choose static versus buyer-specific endpoint fulfillment and the current supported auth model. Map each entitled buyer to its authorized SIP grant; no cross-buyer fallback.
3. Implement any required subscription/entitlement or fulfillment operations for the selected product. Do not assume a free listing removes onboarding or integration obligations.
4. Run limited-visibility pilot tests, including cancellation/revocation, incorrect buyer context, credential rotation, and setup failures.
5. Treat Bedrock AgentCore connectivity as optional and separately tested. Different auth modes can require different integration artifacts.
6. Prepare the public offer packet and record approval/publication/discovery evidence when the operator approves release.

Proposed future directory: `integrations/distribution/aws-marketplace/`. Store no live seller credentials, account secrets, tokens, or invented offer IDs.

## Salesforce AgentExchange

Salesforce documents AgentExchange MCP cards and tools that can be added to Agentforce. Confirm the precise MCP listing type, partner onboarding, review, packaging, and commercial conditions for SIP; do not assume an arbitrary managed-package listing procedure is sufficient for an external MCP service.

Proposed slices:

1. Obtain the current partner/MCP-server listing route and its review requirements.
2. Choose the supported credential/account-linking model and map the Salesforce requester or approved service principal to a tenant-bound SIP grant.
3. Prove tool discovery and an evidence-linked fixture call in Agentforce Builder and the applicable registry/card.
4. Test a second customer organization, denied permissions, revoked access, and an unavailable SIP deployment.
5. Supply accurate listing, support/legal, data-flow, and security-review evidence through the appropriate partner channel.
6. Record public listing and clean-account Agentforce discovery separately from successful direct server registration.

Proposed future directory: `integrations/distribution/agentexchange/`. Customer Salesforce records and SIP service histories are not seed data for the listing.

## Acceptance criteria

- [ ] Seller/partner eligibility and the exact product/listing type are confirmed independently.
- [ ] Commercial, license, data-processing, and support decisions are reviewed by the operator.
- [ ] A synthetic buyer/customer organization can complete the documented setup.
- [ ] Tenant identity cannot be replaced by a model-supplied buyer, account, or organization value.
- [ ] Revocation and cancellation remove the intended access; retries do not duplicate writes.
- [ ] Marketplace metadata/tool contracts match the tested release.
- [ ] Each store has its own review receipt, published URL, and discovery/install test.
- [ ] No paid action, term acceptance, or public publication occurs merely from merging this document.

## Sequencing and primary references

Follows observability/cloud deployment, #5, and the readiness work. The internal SIP plugin marketplace in #38 remains separate.

- [AWS seller listing flow, including MCP and free pricing](https://docs.aws.amazon.com/marketplace/latest/userguide/listing-saas-ai-agents.html)
- [AWS MCP integration guidance](https://docs.aws.amazon.com/marketplace/latest/userguide/integrating-mcp.html)
- [Agentforce and AgentExchange MCP discovery](https://developer.salesforce.com/blogs/2026/10/developers-guide-to-the-winter-27-release)
- [AgentExchange listing entry point](https://developer.salesforce.com/docs/platform/isvforce/guide/appexchange-publish-listings.html)
- [AgentExchange review overview](https://developer.salesforce.com/docs/platform/isvforce/guide/security-review-how-it-works.html)

The Salesforce listing pages can require interactive or partner access. Exact intake details remain an explicit research deliverable; the public MCP discovery evidence alone is not treated as listing approval.
