# Temporal Memory

## Temporal-First Design

Every entry in SIPmem carries temporal metadata. This is not an optional annotation layer bolted onto documents—it is part of the core data model. A `MemoryEntry` without valid temporal context cannot be stored or retrieved.

The principle: **facts are true for a bounded time window, not forever**. A firmware version is valid from the moment it's deployed until it's replaced. A pressure reading is valid at the moment it's observed and decays in relevance over time. A known issue is valid from first occurrence until a root cause fix is verified.

## TemporalContext Fields

Every `MemoryEntry` and `AtomicFact` carries a `TemporalContext`:

```rust
pub struct TemporalContext {
    pub observed_at:    DateTime<Utc>,    // When the fact was observed in the real world
    pub recorded_at:    DateTime<Utc>,    // When it was recorded in the system
    pub valid_from:     DateTime<Utc>,    // Start of the validity window
    pub valid_to:       Option<DateTime<Utc>>,  // End of validity (None = still valid)
    pub superseded_at:  Option<DateTime<Utc>>,  // When a newer version replaced this
    pub superseded_by:  Option<MemoryId>,       // Which entry superseded this one
}
```

| Field | Meaning | Example |
|-------|---------|---------|
| `observed_at` | Real-world timestamp of the observation | A technician noted the pump noise at `2026-04-15T14:30:00Z` |
| `recorded_at` | When the observation was entered into SIP | The work order log was created at `2026-04-15T15:05:00Z` |
| `valid_from` | When the fact became true | The firmware update was applied at `2026-03-01T08:00:00Z` |
| `valid_to` | When the fact stopped being true | The asset was decommissioned at `2026-04-30T00:00:00Z` |
| `superseded_at` | When a newer entry replaced this one's authority | A corrected diagnosis was entered at `2026-04-16T09:00:00Z` |
| `superseded_by` | Pointer to the replacing entry | The MemoryId of the corrected diagnosis entry |

## TemporalResolver

The `TemporalResolver` (`sipmem-core/src/temporal.rs:6`) provides the temporal reasoning primitives:

### `is_valid_at(context, time) -> bool`
Checks whether a fact is valid at a specific point in time.

```rust
// Example: Was the firmware version valid on March 15?
let time = DateTime::parse_from_rfc3339("2026-03-15T00:00:00Z").unwrap();
let valid = resolver.is_valid_at(&firmware_context, time);
// Returns true if valid_from <= time <= valid_to
```

### `is_valid_during(context, range) -> bool`
Checks whether a fact's validity window overlaps with a time range.

```rust
// Example: Was this part installed at any point during Q1 2026?
let range = TimeRange {
    start: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap(),
    end: Some(DateTime::parse_from_rfc3339("2026-03-31T23:59:59Z").unwrap()),
};
let valid = resolver.is_valid_during(&part_context, &range);
```

### `is_stale(context, now) -> bool`
Returns `true` if a fact has expired or been superseded.

```rust
// A fact is stale if:
// 1. superseded_at is set and now >= superseded_at
// 2. valid_to is set and now > valid_to
```

### `prune_stale(entries, now) -> Vec<MemoryEvidence>`
Removes all entries that are stale as of `now`. This is the **Temporal Evidence Gate**—it runs during the retrieval step when `temporal_filter: true` is set in the `RetrievalStep`.

```rust
// In the pipeline (pipeline.rs:55):
if step.temporal_filter {
    let temporal = TemporalResolver::new();
    results = temporal.prune_stale(results, Utc::now());
}
```

### `filter_by_range(entries, range) -> Vec<MemoryEvidence>`
Filters entries to those whose validity window overlaps `range`. Used when `MemoryQuery.temporal_constraint` is set.

### `find_changes(entries, from, to) -> Vec<MemoryEntry>`
Returns entries recorded between `from` and `to`. This enables "what changed?" queries.

## Query Patterns

### Pattern 1: "What was true at time T?"

```rust
let query = MemoryQuery {
    query_text: "pump A oil pressure".into(),
    temporal_constraint: Some(TimeRange {
        start: DateTime::parse_from_rfc3339("2026-04-15T14:30:00Z")?,
        end: None,  // Point-in-time: look for facts valid at this moment
    }),
    ..
};
// The temporal_constraint filters entries where:
//   valid_from <= 2026-04-15T14:30:00Z
//   AND (valid_to IS NULL OR valid_to >= 2026-04-15T14:30:00Z)
```

This routes to the `state_at_time` recipe (SQL + Temporal retrievers, Standard verification, 500ms budget, 0.7 min confidence).

### Pattern 2: "What changed between T1 and T2?"

```rust
let query = MemoryQuery {
    query_text: "pump A changes".into(),
    temporal_constraint: Some(TimeRange {
        start: DateTime::parse_from_rfc3339("2026-04-01T00:00:00Z")?,
        end:   Some(DateTime::parse_from_rfc3339("2026-04-30T23:59:59Z")?),
    }),
    ..
};
// find_changes() returns entries with recorded_at between T1 and T2
```

### Pattern 3: "What is stale?"

```rust
let query = MemoryQuery {
    query_text: "stale or superseded knowledge".into(),
    ..
};
// Routes to stale_evidence_detection recipe.
// prune_stale() is inverted to find (rather than remove) stale entries.
```

This routes to the `stale_evidence_detection` recipe (Temporal + SQL retrievers, Standard verification, 500ms budget, 0.5 min confidence).

### Pattern 4: "Full asset history"

```rust
let query = MemoryQuery {
    query_text: "history of pump A".into(),
    // No temporal_constraint — retrieves all facts, ordered by observed_at
    entity_filters: vec![EntityReference {
        entity_type: "asset".into(),
        entity_id: pump_a_uuid,
        relationship: "direct".into(),
    }],
    ..
};
// Routes to asset_history_summary (SQL + Temporal + Graph, 1000ms budget)
```

## Temporal Filtering in the Pipeline

The `MemoryQuery.temporal_constraint` is honored at three points:

1. **Recipe selection**: The `RetrievalRecipe.temporal_constraint` can pre-filter the recipe's scope.
2. **Retrieval step**: The `RetrievalStep.temporal_filter` flag prunes stale entries after retrieval but before verification.
3. **Memory query**: The `TypedMemorySystem::query()` method applies `valid_from <= end AND valid_to >= start` filtering.

## Concrete Examples

```
Asset: Pump A (serial: PUMP-0042)
Timeline:
  2026-03-01T08:00:00Z  Firmware v2.1.0 deployed  [valid_from, observed_at]
  2026-03-15T10:00:00Z  Telemetry: pressure 45 PSI [observed_at]
  2026-03-20T14:00:00Z  WO #142 created: "bearing noise" [recorded_at]
  2026-03-20T14:00:00Z  Diagnosis: bearing wear (valid_from, valid_to: 2026-03-25)
  2026-03-22T09:00:00Z  Part replaced: bearing BRG-89 [valid_from]
  2026-03-25T11:00:00Z  Corrected diagnosis: misalignment [valid_from, supersedes 03-20]
  2026-04-01T06:00:00Z  Firmware v2.2.0 deployed  [valid_from, supersedes v2.1.0]
  2026-04-15T00:00:00Z  Asset operational (valid_to: None, ongoing)

Query: "What was the known diagnosis for Pump A on March 23?"
  → Returns "bearing wear" (diagnosis from 03-20, valid through 03-25)
  → Does NOT return "misalignment" (valid from 03-25, after query time)

Query: "What changed on Pump A between March 1 and April 1?"
  → Returns: firmware v2.1.0, pressure 45 PSI, WO #142, bearing wear,
             bearing replacement, corrected diagnosis, firmware v2.2.0

Query: "Is there stale knowledge about Pump A?"
  → Returns: firmware v2.1.0 (superseded by v2.2.0)
             bearing wear diagnosis (superseded by misalignment at 03-25)
```

## `valid_to: None` Semantics

When `valid_to` is `None`, the fact is considered "still valid unless superseded." An asset status, a current firmware version, or a persistent known issue all have `valid_to: None`. Staleness for these entries is driven entirely by `superseded_at` and `superseded_by`.
