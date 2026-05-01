# Plugin Contracts

## Architecture

SIPmem is composed entirely of pluggable traits. Every retrieval strategy, verification approach, scoring function, and routing decision can be replaced with a custom implementation. The system binds implementations at construction time via `Box<dyn Trait>`.

## Trait Reference

All traits are defined in `sipmem-core/src/traits.rs`.

### Retriever

```rust
#[async_trait]
pub trait Retriever: Send + Sync {
    fn retriever_type(&self) -> RetrievalType;
    async fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<MemoryEvidence>, SipmemError>;
    async fn health_check(&self) -> Result<bool, SipmemError>;
}
```

**Contract:**
- `retriever_type()` returns the enum variant that identifies this retriever. Used by the execution plan to match `RetrievalType` variants to retriever instances.
- `retrieve()` fetches candidate evidence for the given query. The implementation is responsible for enforcing tenant isolation (RLS, organization_id filter). Returns empty `Vec` on no results, not an error.
- `health_check()` verifies the backend is reachable. Returns `Ok(true)` or an error.

**Error surface:** Use `SipmemError::RetrievalError` for backend failures, `SipmemError::AuthorizationError` for permission denials.

### Verifier

```rust
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
```

**Contract:**
- `verify()` receives pending facts and supporting evidence. It must return the same set of facts with updated `verification_status` and `confidence` fields. It should NOT remove or add facts—only mutate.
- `detect_contradictions()` examines facts for logical conflicts. It returns `ContradictionRecord` entries for any detected conflicts. Return empty `Vec` if no contradictions or if contradiction detection is not applicable.

**Verification status values to assign:**
- `VerificationStatus::Verified` — evidence supports the claim
- `VerificationStatus::PartiallyVerified` — some but not sufficient evidence
- `VerificationStatus::Unsupported` — no evidence found
- `VerificationStatus::Contradicted` — evidence contradicts the claim
- `VerificationStatus::Pending` — not yet evaluated (used by NoopVerifier)

### EvidenceScorer

```rust
pub trait EvidenceScorer: Send + Sync {
    fn score_evidence(
        &self,
        evidence: &[MemoryEvidence],
        query: &MemoryQuery,
    ) -> Vec<MemoryEvidence>;
    fn confidence_score(&self, facts: &[AtomicFact]) -> f64;
}
```

**Contract:**
- `score_evidence()` re-ranks evidence by relevance. Returns a sorted `Vec` (highest relevance first). The implementation should consider `relevance_score`, `matches_query_entities`, `temporal_match`, and the original query text.
- `confidence_score()` computes an aggregate confidence (0.0–1.0) across verified facts. The default implementation returns the mean confidence.

### TemporalResolverTrait

```rust
pub trait TemporalResolverTrait: Send + Sync {
    fn is_valid_at(&self, context: &TemporalContext, time: DateTime<Utc>) -> bool;
    fn is_stale(&self, context: &TemporalContext, now: DateTime<Utc>) -> bool;
    fn prune(&self, entries: Vec<MemoryEvidence>, now: DateTime<Utc>) -> Vec<MemoryEvidence>;
    fn filter(&self, entries: Vec<MemoryEvidence>, range: &TimeRange) -> Vec<MemoryEvidence>;
}
```

### MemoryRouter

```rust
#[async_trait]
pub trait MemoryRouter: Send + Sync {
    async fn route(&self, query: &MemoryQuery) -> Result<ExecutionPlan, SipmemError>;
}
```

**Contract:**
- `route()` classifies the query and returns an `ExecutionPlan` specifying which retrievers to call, verification level, temporal constraints, and cost budget.
- The plan's `steps` are executed in order. Within each step, retrievers run in parallel (if `parallel: true`) or sequentially.
- The `temporal_filter` flag on each step controls whether stale pruning occurs after retrieval.

## SipmemError

All plugin methods return `Result<T, SipmemError>`. Use the appropriate variant:

```rust
pub enum SipmemError {
    RetrievalError(String),      // Backend access failure
    VerificationError(String),   // Verification logic failure
    TemporalError(String),       // Temporal resolution failure
    InvalidQuery(String),        // Malformed or unauthorized query
    AuthorizationError(String),  // Permission denied
    ProviderError(String),       // External provider failure
    NotImplemented(String),      // Feature not yet implemented
}
```

## Example: Custom SQL-Backed Retriever

```rust
use async_trait::async_trait;
use sipmem_core::{MemoryEvidence, MemoryQuery, Retriever, RetrievalType, SipmemError};
use sqlx::PgPool;

pub struct PostgresRetriever {
    pool: PgPool,
}

impl PostgresRetriever {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Retriever for PostgresRetriever {
    fn retriever_type(&self) -> RetrievalType {
        RetrievalType::SQL
    }

    async fn retrieve(
        &self,
        query: &MemoryQuery,
    ) -> Result<Vec<MemoryEvidence>, SipmemError> {
        let rows = sqlx::query_as::<_, MemoryEvidence>(
            r#"
            SELECT ...
            FROM work_orders
            WHERE tenant_id = $1
              AND title ILIKE '%' || $2 || '%'
            LIMIT $3
            "#,
        )
        .bind(query.tenant_id)
        .bind(&query.query_text)
        .bind(query.max_results as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipmemError::RetrievalError(e.to_string()))?;

        Ok(rows)
    }

    async fn health_check(&self) -> Result<bool, SipmemError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| true)
            .map_err(|e| SipmemError::RetrievalError(e.to_string()))
    }
}
```

## Example: Specialist Verifier for Firmware Domain

```rust
use async_trait::async_trait;
use sipmem_core::{
    AtomicFact, ContradictionRecord, MemoryEvidence, SipmemError,
    VerificationStatus, Verifier,
};

pub struct FirmwareVerifier;

#[async_trait]
impl Verifier for FirmwareVerifier {
    async fn verify(
        &self,
        facts: &[AtomicFact],
        evidence: &[MemoryEvidence],
    ) -> Result<Vec<AtomicFact>, SipmemError> {
        let mut result = facts.to_vec();

        for fact in &mut result {
            // Only process firmware-related facts
            if fact.predicate != "firmware_version" {
                continue;
            }

            let version_in_evidence = evidence.iter().any(|e| {
                e.entry.content.contains(&fact.object_value.clone().unwrap_or_default())
            });

            if version_in_evidence {
                // Check if the version is the latest known
                let is_latest = evidence.iter().any(|e| {
                    e.entry.content.contains("latest")
                        && e.entry.content.contains(&fact.object_value.clone().unwrap_or_default())
                });

                fact.verification_status = if is_latest {
                    VerificationStatus::Verified
                } else {
                    VerificationStatus::PartiallyVerified
                };
                fact.confidence = if is_latest { 0.95 } else { 0.6 };
            } else {
                fact.verification_status = VerificationStatus::Unsupported;
                fact.confidence = 0.05;
            }
        }

        Ok(result)
    }

    async fn detect_contradictions(
        &self,
        facts: &[AtomicFact],
    ) -> Result<Vec<ContradictionRecord>, SipmemError> {
        // FirmwareVerifier doesn't handle contradiction detection
        Ok(vec![])
    }
}
```

## Registering Plugins

### With VerificationEngine

```rust
use sipmem_core::{
    VerificationEngine, DefaultEvidenceScorer,
    BasicVerifier, NoopVerifier,
};

// Create engine with a scorer
let mut engine = VerificationEngine::new(
    Box::new(DefaultEvidenceScorer)
);

// Register verifiers in priority order (first registered runs first)
engine.add_verifier(Box::new(BasicVerifier));
engine.add_verifier(Box::new(CrossReferenceVerifier));

// Specialist verifier runs after general verifiers
// Its output refines the classifications made by earlier verifiers
engine.add_verifier(Box::new(FirmwareVerifier));

// Use the engine
let verified = engine.verify_claims(&claims, &evidence).await?;
let contradictions = engine.detect_contradictions(&claims).await?;
```

### With SipmemPipeline

`SipmemPipeline` does not currently expose public methods for injecting custom retrievers, router, verifier, or scorer. The constructor `SipmemPipeline::new()` hardcodes the defaults.

To inject custom plugins, you can:

**Option A: Build the struct directly**

```rust
let pipeline = SipmemPipeline {
    retrievers: vec![
        Box::new(SQLRetriever),
        Box::new(VectorRetriever),
        Box::new(GraphRetriever),
        Box::new(TemporalRetriever),
        Box::new(MyCustomRetriever),  // Add custom retriever
    ],
    router: Box::new(MyCustomRouter::new()),
    verifier: Box::new(MyCustomVerifier),
    scorer: Box::new(MyCustomScorer),
    memory: TypedMemorySystem::new(),
};
```

**Option B: Implement a builder for SipmemPipeline**

```rust
impl SipmemPipeline {
    pub fn builder() -> SipmemPipelineBuilder {
        SipmemPipelineBuilder::default()
    }
}

pub struct SipmemPipelineBuilder {
    retrievers: Vec<Box<dyn Retriever>>,
    router: Option<Box<dyn MemoryRouter>>,
    verifier: Option<Box<dyn Verifier>>,
    scorer: Option<Box<dyn EvidenceScorer>>,
    memory: TypedMemorySystem,
}

impl SipmemPipelineBuilder {
    pub fn with_retriever(mut self, r: Box<dyn Retriever>) -> Self {
        self.retrievers.push(r);
        self
    }
    pub fn with_router(mut self, r: Box<dyn MemoryRouter>) -> Self {
        self.router = Some(r);
        self
    }
    pub fn with_verifier(mut self, v: Box<dyn Verifier>) -> Self {
        self.verifier = Some(v);
        self
    }
    pub fn with_scorer(mut self, s: Box<dyn EvidenceScorer>) -> Self {
        self.scorer = Some(s);
        self
    }
    pub fn build(self) -> SipmemPipeline {
        SipmemPipeline {
            retrievers: self.retrievers,
            router: self.router.unwrap_or_else(|| Box::new(DefaultRouter::new())),
            verifier: self.verifier.unwrap_or_else(|| Box::new(CrossReferenceVerifier)),
            scorer: self.scorer.unwrap_or_else(|| Box::new(DefaultEvidenceScorer)),
            memory: self.memory,
        }
    }
}
```

## Implementing a Custom RetrieverType

The `RetrievalType` enum in `types.rs:50` supports a `Plugin` variant for custom backends:

```rust
pub enum RetrievalType {
    SQL,
    Vector,
    Graph,
    Temporal,
    Hybrid,
    Plugin,  // <-- Use this for custom retrievers
}
```

If the existing variants don't cover your use case (e.g., you need a new variant like `Telemetry`), add it to the enum and reference it in your retriever's `retriever_type()` and in recipe definitions.

## Implementing a Custom MemoryRouter

```rust
use async_trait::async_trait;
use sipmem_core::{ExecutionPlan, MemoryQuery, MemoryRouter, SipmemError};

pub struct LLMClassifierRouter {
    // Could call an LLM to classify query intent
}

#[async_trait]
impl MemoryRouter for LLMClassifierRouter {
    async fn route(&self, query: &MemoryQuery) -> Result<ExecutionPlan, SipmemError> {
        // Classify query intent via LLM
        let intent = classify_via_llm(&query.query_text).await?;

        // Select recipe based on intent
        let recipe = match intent {
            "firmware" => RetrievalRecipe::firmware_specific(),
            "root_cause" => RetrievalRecipe::root_cause_candidate(),
            _ => RetrievalRecipe::troubleshooting_similarity(),
        };

        Ok(ExecutionPlan {
            query: query.clone(),
            recipe_name: recipe.name.clone(),
            steps: vec![RetrievalStep {
                retriever_types: recipe.retrievers,
                parallel: true,
                max_results_per_retriever: 10,
                temporal_filter: recipe.temporal_constraint.is_some(),
            }],
            verification_level: recipe.verification_level,
            temporal_constraint: recipe.temporal_constraint,
            cost_budget_ms: recipe.cost_budget_ms,
        })
    }
}
```
