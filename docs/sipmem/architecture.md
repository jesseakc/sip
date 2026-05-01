# SIPmem Architecture

## What SIPmem Is (And Isn't)

SIPmem is **not** a RAG (Retrieval-Augmented Generation) system. It is an **evidence engine**.

A RAG system retrieves context and passes it to an LLM for answer generation. SIPmem retrieves candidate evidence, but then subjects it to a verification loop: cross-referencing claims against source records, detecting contradictions, scoring confidence, and persisting conflicting evidence instead of discarding it.

The core workflow is:

```
Query → Auth Gate → Decompose → Recipe Selection → Retrieve → Evidence Assembly → Verify → Synthesize
```

At each stage, authorization is enforced (same RBAC/RLS as the SIP REST API). The system does not answer questions it cannot ground in retrievable source records.

## Pipeline Stages

### 1. Auth Gate
Every query carries a `tenant_id` and optional `user_id`. Before any retrieval, the system verifies the caller's permissions. AI agents use the same permission model as human users (`AuthService` + `TenantContext`).

### 2. Decompose
The raw query text is analyzed to extract entity references, temporal constraints, and intent signals. This feeds into recipe selection.

### 3. Recipe Selection
A `MemoryRouter` implementation classifies the query and selects a `RetrievalRecipe`. Recipes are pre-defined retrieval strategies (see [retrieval-recipes.md](./retrieval-recipes.md)). The router can use keyword heuristics (`RecipeRouter::select`) or be swapped for an LLM-based classifier.

### 4. Retrieve
The recipe's specified retrievers are invoked. Retrievers implement the `Retriever` trait and can execute in parallel or sequentially based on the `ExecutionPlan`. Results are collected as `Vec<MemoryEvidence>`.

### 5. Evidence Assembly
Raw retrieval results pass through the `EvidenceScorer` for relevance ranking, then through the `TemporalResolver` to prune stale entries and filter by time range.

### 6. Verify
Candidate claims are extracted from evidence and passed through every registered `Verifier`. Verifiers cross-reference claims against evidence content, detect contradictions, and assign verification statuses. See [verification-loop.md](./verification-loop.md).

### 7. Synthesize
The pipeline produces a `VerifiedAnswer` containing:
- `answer_text`: synthesized summary
- `confidence`: average confidence across verified claims
- `verified_claims`, `unsupported_claims`, `contradicted_claims`: triaged claim lists
- `evidence_used`: the scored evidence that produced the answer
- `retrieval_paths`: which retrieval types were used
- `recommended_actions`: suggested next steps

## Typed Memory: 14 Categories

SIPmem does not store generic "documents" or "chunks". Every memory entry has an explicit `MemoryCategory`:

| Category | Purpose |
|----------|---------|
| `Asset` | Physical asset records |
| `ServiceCase` | Service/work order case data |
| `TechnicianNote` | Free-text notes from technicians |
| `ManualSection` | Extracted sections from service manuals |
| `ServiceBulletin` | OEM bulletins, recall notices |
| `FirmwareFact` | Firmware/software version and configuration |
| `PartReplacement` | Parts replaced during service events |
| `KnownIssue` | Identified failure patterns |
| `Procedure` | Step-by-step repair procedures |
| `FailureMode` | Specific ways an asset can fail |
| `RootCause` | Determined root causes of failures |
| `TelemetrySummary` | Aggregated telemetry observations |
| `VerifiedFact` | Facts that passed verification |
| `ContradictionRecord` | Persisted conflicts between facts |
| `InspectionFinding` | Structured inspection results |
| `DocumentChunk` | Text segments from documents |

Typed memory enables precise retrieval routing: a query about firmware versions routes to `SQL` over `FirmwareFact`, while a query about symptom similarity routes to `Vector` over `TechnicianNote` and `ServiceCase`.

## The Fact Ledger

All knowledge in SIPmem reduces to `AtomicFact` records in the `FactLedger`.

```rust
pub struct AtomicFact {
    pub id: String,
    pub claim: String,
    pub subject_entity: Option<EntityReference>,
    pub predicate: String,
    pub object_value: Option<String>,
    pub confidence: f64,
    pub supporting_evidence: Vec<MemoryId>,
    pub contradicting_evidence: Vec<MemoryId>,
    pub verification_status: VerificationStatus,
    pub temporal_context: Option<TemporalContext>,
}
```

Each fact is a subject-predicate-object triple tied to a real entity (asset, work order, part, etc.). The ledger supports querying by entity, time range, and verification status. Contradictions are stored as first-class `ContradictionRecord` entities rather than discarded—conflicting evidence is itself evidence.

## Contradiction Memory

When two facts about the same entity share a predicate but have different object values, the system records a `ContradictionRecord`:

```rust
pub struct ContradictionRecord {
    pub id: String,
    pub claim: String,
    pub conflicting_sources: Vec<EvidenceSource>,
    pub supporting_sources: Vec<EvidenceSource>,
    pub entities: Vec<EntityReference>,
    pub time_range: Option<TimeRange>,
    pub status: ContradictionStatus,
    pub resolution_notes: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}
```

Contradiction lifecycle: `Open → Investigating → Resolved | Accepted | Superseded`. Unresolved contradictions don't block answers—they appear in `contradicted_claims` with recommendations.

## Temporal-First Design

Every `MemoryEntry` and `AtomicFact` carries temporal metadata. The `TemporalResolver` answers "what was true at time T?", "what changed between T1 and T2?", and "what is stale?". See [temporal-memory.md](./temporal-memory.md).

## Plugin Architecture

Everything is a trait. The system is composed by injecting implementations:

| Trait | Role | Location |
|-------|------|----------|
| `Retriever` | Fetch candidate evidence from a source | `sipmem-core/src/traits.rs:7` |
| `Verifier` | Validate facts against evidence | `sipmem-core/src/traits.rs:14` |
| `EvidenceScorer` | Rank evidence by relevance | `sipmem-core/src/traits.rs:26` |
| `TemporalResolverTrait` | Resolve temporal validity | `sipmem-core/src/traits.rs:35` |
| `MemoryRouter` | Classify queries and select recipes | `sipmem-core/src/traits.rs:51` |

See [plugin-contracts.md](./plugin-contracts.md) for implementation guides.

### Trait Signatures

```rust
#[async_trait]
pub trait Retriever: Send + Sync {
    fn retriever_type(&self) -> RetrievalType;
    async fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<MemoryEvidence>, SipmemError>;
    async fn health_check(&self) -> Result<bool, SipmemError>;
}

#[async_trait]
pub trait Verifier: Send + Sync {
    async fn verify(
        &self,
        facts: &[AtomicFact],
        evidence: &[MemoryEvidence],
    ) -> Result<Vec<AtomicFact>, SipmemError>;
    async fn detect_contradictions(
        &self,
        facts: &[AtomicFact],
    ) -> Result<Vec<ContradictionRecord>, SipmemError>;
}

#[async_trait]
pub trait MemoryRouter: Send + Sync {
    async fn route(&self, query: &MemoryQuery) -> Result<ExecutionPlan, SipmemError>;
}
```

### ExecutionPlan Structure

```rust
pub struct ExecutionPlan {
    pub query: MemoryQuery,
    pub recipe_name: String,
    pub steps: Vec<RetrievalStep>,
    pub verification_level: VerificationLevel,
    pub temporal_constraint: Option<TimeRange>,
    pub cost_budget_ms: u64,
}

pub struct RetrievalStep {
    pub retriever_types: Vec<RetrievalType>,
    pub parallel: bool,           // Run retrievers concurrently or sequentially
    pub max_results_per_retriever: usize,
    pub temporal_filter: bool,    // Prune stale entries after retrieval
}
```

### Pipeline Execution Flow

```
SipmemPipeline::execute(query)
│
├─ router.route(&query)
│  └─ MemoryRouter::route → ExecutionPlan
│
├─ For each RetrievalStep in plan.steps:
│  ├─ For each retriever_type in step.retriever_types:
│  │  └─ Retriever::retrieve(&query) → Vec<MemoryEvidence>
│  └─ If step.temporal_filter:
│     └─ TemporalResolver::prune_stale(results, now)
│
├─ EvidenceScorer::score_evidence(&all_evidence, &query)
│
├─ extract_claims(evidence) → Vec<AtomicFact>
│
├─ verifier.verify(&claims, &evidence) → Vec<AtomicFact>
│
├─ verifier.detect_contradictions(&claims) → Vec<ContradictionRecord>
│
├─ classify_claims(verified_claims, contradictions)
│  └─ (verified, unsupported, contradicted)
│
├─ synthesize(verified, unsupported, plan.verification_level)
│
└─ scorer.confidence_score(&verified) → f64
   │
   └─ VerifiedAnswer
```

## Crate Structure

```
sipmem-core (crates/sipmem-core/)
│  Core types, traits, and domain logic.
│  Zero external dependencies beyond async-trait, serde, chrono, uuid.
│
│  src/types.rs        — MemoryEntry, AtomicFact, MemoryQuery, VerifiedAnswer, etc.
│  src/traits.rs       — Retriever, Verifier, EvidenceScorer, MemoryRouter traits
│  src/memory.rs       — TypedMemorySystem, FactLedger, ContradictionRecord
│  src/temporal.rs     — TemporalResolver
│  src/evidence.rs     — EvidenceMatcher, DefaultEvidenceScorer
│  src/verification.rs — VerificationEngine, BasicVerifier, NoopVerifier
│  src/recipes.rs      — RetrievalRecipe definitions, RecipeRouter
│
└─ sipmem-adapters (crates/sipmem-adapters/)
   │  Concrete implementations of core traits. Depends on sipmem-core.
   │
   │  src/retrievers.rs — SQLRetriever, VectorRetriever, GraphRetriever, TemporalRetriever
   │  src/verifiers.rs  — CrossReferenceVerifier
   │  src/router.rs     — DefaultRouter (wraps RecipeRouter)
   │  src/pipeline.rs   — SipmemPipeline (orchestrates full execution)
   │
   └─ sip-application (crates/sip-application/)
      Consumes sipmem-adapters via SipmemPipeline.
      The AIService wires pipeline execution into the chat/QA endpoints.
```

The layering is: `sip-application` → `sipmem-adapters` → `sipmem-core`. The core crate defines abstractions; adapters provide implementations; the application binds them together.

## Key Design Decisions

1. **Evidence is not truth.** Retrieved context is a candidate. Every claim must be verified against source records.

2. **Contradictions are preserved.** Conflicting evidence is a signal, not noise. Persist it at `min_confidence: 0.3` so it surfaces in future queries.

3. **Temporal gate before retrieval.** Stale evidence is pruned at the retrieval step level (`temporal_filter: true` on `RetrievalStep`), not post-hoc.

4. **Typed memory enables routing.** A query's intent determines which `RetrievalType` backends are called and which `MemoryCategory` entries are prioritized.

5. **Authorization at retrieval time.** Same RLS/RBAC as the REST API. Queries carry `tenant_id`; retrievers enforce it.
