# Retrieval Recipes

## Overview

A `RetrievalRecipe` is a predefined retrieval strategy that specifies which retrievers to use, what verification level to apply, any temporal constraints, cost/time budgets, and minimum confidence thresholds.

Recipes are defined in `sipmem-core/src/recipes.rs` and selected by the `RecipeRouter` based on query text heuristics (or an explicit `recipe_preference` field in `MemoryQuery`).

## Recipe Structure

```rust
pub struct RetrievalRecipe {
    pub name: String,
    pub description: String,
    pub retrievers: Vec<RetrievalType>,      // Which backends to query
    pub verification_level: VerificationLevel, // Rigor of verification
    pub temporal_constraint: Option<TimeRange>, // Time window (None = use query's constraint)
    pub cost_budget_ms: u64,                 // Maximum allowed latency
    pub min_confidence: f64,                 // Minimum confidence for evidence
}
```

## All 10 Recipes

| # | Recipe Name | Retrievers | Verification | Budget | Min Confidence | Use Case |
|---|-------------|-----------|-------------|--------|---------------|----------|
| 1 | `exact_fact_lookup` | SQL | Basic | 200ms | 0.7 | Direct fact queries: serial numbers, status, firmware version |
| 2 | `state_at_time` | SQL, Temporal | Standard | 500ms | 0.7 | Point-in-time queries: "what was the pressure at 3pm?" |
| 3 | `asset_history_summary` | SQL, Temporal, Graph | Standard | 1000ms | 0.6 | Full asset history: all events, replacements, diagnoses |
| 4 | `troubleshooting_similarity` | Vector, SQL | Standard | 800ms | 0.5 | Find similar past issues for diagnostic support |
| 5 | `known_issue_investigation` | Vector, Graph | Strict | 1500ms | 0.6 | Investigate known failure patterns and relationships |
| 6 | `firmware_specific` | SQL | Basic | 200ms | 0.8 | Firmware/software version lookups (high confidence) |
| 7 | `service_bulletin_check` | Vector, SQL | Standard | 600ms | 0.7 | OEM bulletin and recall notice searches |
| 8 | `contradiction_detection` | SQL, Vector, Graph, Temporal | Full | 3000ms | 0.3 | Exhaustive search for conflicting evidence (low threshold to catch everything) |
| 9 | `root_cause_candidate` | Graph, Temporal, Vector | Strict | 2000ms | 0.5 | Trace failure chains to potential root causes |
| 10 | `stale_evidence_detection` | Temporal, SQL | Standard | 500ms | 0.5 | Find superseded or expired evidence |

## Recipe Selection Heuristics

The `RecipeRouter::select()` method (`recipes.rs:182`) uses keyword matching in order of priority:

| Priority | Keywords | Selected Recipe |
|----------|----------|----------------|
| 1 (highest) | `root cause`, `why did` | `root_cause_candidate` |
| 2 | `contradiction`, `conflict` | `contradiction_detection` |
| 3 | `firmware`, `version`, `software` | `firmware_specific` |
| 4 | `bulletin`, `recall`, `oem` | `service_bulletin_check` |
| 5 | `history`, `lifetime` | `asset_history_summary` |
| 6 | `similar`, `like`, `troubleshoot` | `troubleshooting_similarity` |
| 7 | `known issue`, `pattern` | `known_issue_investigation` |
| 8 | `stale`, `superseded`, `expired` | `stale_evidence_detection` |
| 9 | `when`, `time`, `date` | `state_at_time` |
| 10 (fallback) | (none matched) | `exact_fact_lookup` |

If the caller sets `MemoryQuery.recipe_preference`, that recipe is used regardless of query text. If the preferred recipe doesn't exist, the `DefaultRouter` falls back to `troubleshooting_similarity`.

## Execution Plan Flow

When a recipe is selected, the `DefaultRouter::route()` method produces an `ExecutionPlan`:

```rust
ExecutionPlan {
    query:              query.clone(),
    recipe_name:        recipe.name.clone(),
    steps:              vec![RetrievalStep {
        retriever_types:        recipe.retrievers.clone(),
        parallel:               true,    // All retrievers run concurrently in single step
        max_results_per_retriever: 10,
        temporal_filter:        recipe.temporal_constraint.is_some(),
    }],
    verification_level:  recipe.verification_level,
    temporal_constraint: recipe.temporal_constraint,
    cost_budget_ms:      recipe.cost_budget_ms,
}
```

### Parallel vs Sequential Retrieval

Currently, all steps run with `parallel: true`, meaning all retrievers in a step are invoked concurrently. This is appropriate for most recipes since retrievers are independent (SQL, Vector, Graph, Temporal query different backends).

For recipes where one retriever's output feeds into another (e.g., graph traversal results narrow a vector search), set `parallel: false` and use multiple `RetrievalStep` entries in the plan.

### Cost Budgets

The `cost_budget_ms` field is an advisory limit. The pipeline does not enforce hard timeouts—it's used by the router to select between recipes with different cost profiles. The actual execution time depends on backend performance.

| Budget Tier | Recipes | Typical Latency |
|-------------|---------|----------------|
| 200ms | `exact_fact_lookup`, `firmware_specific` | Fast SQL lookups |
| 500-600ms | `state_at_time`, `service_bulletin_check`, `stale_evidence_detection` | Two retrievers with temporal filtering |
| 800-1000ms | `troubleshooting_similarity`, `asset_history_summary` | Two to three retrievers |
| 1500-2000ms | `known_issue_investigation`, `root_cause_candidate` | Strict verification, three retrievers |
| 3000ms | `contradiction_detection` | All four retrievers, Full verification |

## Registering Custom Recipes

### At RecipeRouter Level

```rust
use sipmem_core::{RetrievalRecipe, RetrievalType, VerificationLevel};

let custom_recipe = RetrievalRecipe {
    name: "telemetry_anomaly".into(),
    description: "Correlate telemetry anomalies with work order history".into(),
    retrievers: vec![RetrievalType::SQL, RetrievalType::Temporal, RetrievalType::Vector],
    verification_level: VerificationLevel::Strict,
    temporal_constraint: None,
    cost_budget_ms: 1200,
    min_confidence: 0.5,
};

let mut router = RecipeRouter::new();
router.register(custom_recipe);
```

### In the DefaultRouter

```rust
let mut inner = RecipeRouter::new();
inner.register(RetrievalRecipe::exact_fact_lookup());
inner.register(my_custom_recipe);
let router = DefaultRouter { inner };
```

### Adding Keyword Triggers

The `RecipeRouter::select()` method must be extended to handle new keywords. Either:
1. Fork the `RecipeRouter` with custom selection logic
2. Implement `MemoryRouter` directly with a custom route method
3. Use `recipe_preference` in the query to bypass keyword matching

## RetrievalType Reference

| RetrievalType | Backend | Data Source |
|---------------|---------|-------------|
| `SQL` | PostgreSQL direct queries | Work orders, assets, parts, all structured tables |
| `Vector` | pgvector HNSW similarity search | Work order notes, document chunks, technician notes |
| `Graph` | Recursive CTEs + ltree | Asset hierarchy, location paths, failure chain traversal |
| `Temporal` | Activity log + status history | Change-over-time preservation, history queries |
| `Hybrid` | Combines multiple types | Multi-path retrieval with result merging |
| `Plugin` | Custom retriever implementation | Third-party or domain-specific retrieval backends |
