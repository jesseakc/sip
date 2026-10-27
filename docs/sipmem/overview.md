# SIPmem overview

[Documentation home](../README.md)

SIPmem is the memory and evidence engine design for physical assets and service operations. Its components exist in `sipmem-core` and `sipmem-adapters`, but the full pipeline is not connected to the running API. The live chat path is described in the [application architecture](../architecture.md).

## Architecture

```
Query → Auth Gate → Recipe Selection → Hybrid Retrieval → Evidence Scoring
  → Claim Extraction → Verification → Contradiction Detection → Synthesize
```

Evidence is NOT truth. Every retrieved record is a **candidate** that must survive verification before it can be used in an answer.

## Typed Memory (14 Categories)

SIPmem does not store generic "documents." Every memory entry has an explicit type:

| Category | Example Content |
|----------|----------------|
| Asset | Asset record, serial number, specification, criticality |
| ServiceCase | Work order, service ticket, repair request |
| TechnicianNote | Resolution notes, field observations, troubleshooting steps |
| ManualSection | Excerpt from maintenance/procedure manuals |
| ServiceBulletin | OEM bulletin, recall notice, advisory |
| FirmwareFact | Software/firmware version, update history |
| PartReplacement | Part swapped, replacement date, reason |
| KnownIssue | Pattern of failures across similar assets |
| Procedure | Step-by-step repair/maintenance procedure |
| FailureMode | How an asset class fails |
| RootCause | Why a specific failure occurred |
| TelemetrySummary | Aggregated sensor/telemetry insight |
| VerifiedFact | Atomic claim that passed verification |
| ContradictionRecord | Documented conflict between sources |

## Fact Ledger

The fact ledger represents claims as **atomic facts.** Each fact tracks:

- What entity it describes (asset, work order, part)
- When it was observed vs. when it was recorded
- Its validity window (valid_from → valid_to)
- Which evidence supports it
- Whether it has been verified or contradicted

## Retrieval Router & Recipes

Queries are routed to one of 10 **retrieval recipes**, each defining which retrievers to use, what verification level is needed, what temporal constraints apply, and the cost budget.

| Recipe | Retriever Mix | Verification | Example Query |
|--------|--------------|-------------|---------------|
| Exact Fact Lookup | SQL | Basic | "What is the serial number?" |
| State at Time | SQL + Temporal | Standard | "What was the status on June 1?" |
| Asset History Summary | SQL + Temporal + Graph | Standard | "Summarize the service history" |
| Troubleshooting Similarity | Vector + SQL | Standard | "Have we seen this symptom before?" |
| Known Issue Investigation | Vector + Graph | Strict | "Is this a known failure pattern?" |
| Firmware Specific | SQL | Basic | "What firmware version is installed?" |
| Service Bulletin Check | Vector + SQL | Standard | "Are there active bulletins?" |
| Contradiction Detection | All | Full | "Do any sources conflict?" |
| Root Cause Candidate | Graph + Temporal + Vector | Strict | "What is the most likely cause?" |
| Stale Evidence Detection | Temporal + SQL | Basic | "What facts are out of date?" |

## Pipeline (Rust-native)

The `SipmemPipeline` (in `sipmem-adapters`) orchestrates the full flow. The pipeline is assembled from pluggable components implementing these traits:

| Trait | Crate | Purpose |
|-------|-------|---------|
| `Retriever` | `sipmem-core` | Retrieves candidate evidence (SQL, Vector, Graph, Temporal) |
| `MemoryRouter` | `sipmem-core` | Selects the best retrieval recipe for a query |
| `Verifier` | `sipmem-core` | Cross-references claims against evidence, detects contradictions |
| `EvidenceScorer` | `sipmem-core` | Scores evidence relevance and confidence |

**SIPmem Docs:**

- [Architecture](architecture.md): full pipeline design
- [Verification Loop](verification-loop.md): evidence engine philosophy
- [Temporal Memory](temporal-memory.md): time-scoped facts and pruning
- [Retrieval Recipes](retrieval-recipes.md): recipe selection and custom recipes
- [Plugin Contracts](plugin-contracts.md): implementing custom retrievers and verifiers

## Principles

| # | Principle |
|---|-----------|
| **P1** | Evidence is not truth. Every record must be verified before use. |
| **P2** | Typed memory beats generic documents. Categories constrain what can be retrieved. |
| **P3** | Time is a first-class dimension. Every fact has a validity window. |
| **P4** | Contradictions are data, not errors. Conflicts are persisted and tracked. |
| **P5** | The pipeline is pluggable. Retrievers, verifiers, and scorers are traits. |
