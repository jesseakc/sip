# SIP Notifications: Grounded AI Communications

Status: Proposal, not implementation. Depends on notification events and approval workflow.

## Goal
Optional model-agnostic AI generation of role-appropriate communications grounded exclusively in authorized service facts. Must work with no AI enabled through deterministic templates. No provider/model defaults; installer requires explicit provider, model, credential reference and capabilities when AI is enabled.

## Processing
Retrieve authorized event, asset status, service notes and approved facts through SIP APIs. Construct structured evidence with source identifiers and timestamps. Classify recipient audience and sensitivity. Generate structured draft (subject, body, fact citations, uncertainty, omissions, language). Validate against schema, source evidence and disclosure policy. Send to human approval queue as configured; do not autonomously assert unverified diagnosis, ETA, restoration or customer impact.

## Safeguards
Treat service notes and attachments as untrusted prompt-injection surfaces. No recipient selection or policy overrides by LLM. Tenant-scoped retrieval and redaction, configurable provider data handling, audit prompt/template/model versions without storing secrets, strict output validation, hallucination checks and fail-closed fallback to templates or manual review. Avoid sending sensitive data to external models unless explicitly permitted.

## User experience
Show original documented facts alongside draft, why recipients were selected, edits and approval history. Provide regenerate and compare, tone and reading-level controls, source links, multilingual support in future. Explain uncertainty rather than fabricate.

## Acceptance tests
Missing ETA remains unknown; suspected diagnosis stays tentative; adversarial note cannot redirect model; no cross-tenant facts; disabled AI still sends templates; malformed output is held; approved edits and evidence are auditable; model/provider switch requires no core changes.
