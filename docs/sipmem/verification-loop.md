# Verification Loop

## Philosophy: Evidence Is Not Truth

SIPmem treats every piece of retrieved context as a **candidate claim**, not an established fact. The verification loop is the mechanism that distinguishes grounded answers from hallucinated ones.

Core principle: a claim is only accepted as "verified" if its text appears verbatim in at least one piece of retrieved evidence. Everything else is either unsupported or contradicted.

## Verification Pipeline

```
Retrieved Evidence
       │
       ▼
  extract_claims()        ← Map each MemoryEvidence → AtomicFact (Pending)
       │
       ▼
  Verifier::verify()      ← Cross-reference claims against evidence content
       │
       ▼
  Verifier::detect_contradictions()  ← Find conflicting claims (same entity + predicate, different values)
       │
       ▼
  classify_claims()       ← Sort into verified / unsupported / contradicted
       │
       ▼
  EvidenceScorer::confidence_score()  ← Average confidence of verified claims
```

## Atomic Claim Extraction

The pipeline extracts one `AtomicFact` per piece of evidence (`SipmemPipeline::extract_claims`, `pipeline.rs:105`). Each claim is initialized with:

- `claim`: first 200 chars of the evidence entry's `content`
- `subject_entity`: first entity reference from the memory entry
- `predicate`: `"described_in"` (descriptive relationship to the query)
- `object_value`: the original query text
- `confidence`: the evidence's `relevance_score`
- `verification_status`: `Pending`

This extraction is deliberately mechanical—it doesn't interpret the text. Interpretation happens at the verifier and synthesis stages.

## Cross-Reference Verification

The `CrossReferenceVerifier` (`sipmem-adapters/src/verifiers.rs:7`) checks whether a claim's text literally appears in any supporting evidence:

```rust
fn verify(facts, evidence) {
    for each fact {
        let has_support = evidence.iter().any(|e| {
            e.entry.content.contains(&fact.claim)
        });
        fact.status = has_support ? Verified : Unsupported;
        fact.confidence = has_support ? 0.85 : 0.1;
    }
}
```

This is a naive implementation. Production verifiers should:
- Use token-based overlap instead of substring matching
- Check entity references, not just text content
- Consider temporal context (was the evidence valid when the claim was made?)
- Delegate to specialist verifiers for specific domains (firmware, telemetry, etc.)

## Verification Levels

Each `RetrievalRecipe` specifies a `VerificationLevel` that controls the rigor of the verification pass:

| Level | Behavior |
|-------|----------|
| `None` | Skip verification entirely. All claims treated as unverified context. Used for exploratory queries. |
| `Basic` | Single-pass verification. One verifier checks evidence presence. Used for exact fact lookups and firmware queries where confidence is high. |
| `Standard` | Full verification pass with staler evidence pruning. Used for troubleshooting, bulletin checks, and asset history. Default level. |
| `Strict` | Multi-verifier pass. Each verifier runs independently, and results are merged. Contradiction detection is always enabled. Used for known issue investigation and root cause analysis. |
| `Full` | All verifiers run exhaustively. No cost budget constraints on verification. Contradiction detection across all evidence. Used for `contradiction_detection` recipe. |

The verification level is set in the `ExecutionPlan` returned by the router and enforced during the pipeline's `verify` step.

## Contradiction Detection

The `BasicVerifier` (`sipmem-core/src/verification.rs:92`) detects contradictions with a simple heuristic:

**Two facts contradict** if they share:
1. The same entity (`subject_entity.entity_id`)
2. The same predicate (`predicate`)
3. Different object values (`object_value`)

Example: Two facts about sensor `sensor_42` both claim a `temperature` predicate but with values `"100C"` and `"80C"`. This triggers a `ContradictionRecord`.

```rust
// Contradiction detection pseudocode
for (i, fact_a) in facts.iter().enumerate() {
    for fact_b in facts.iter().skip(i + 1) {
        if same_entity && same_predicate && different_value {
            record_contradiction(fact_a, fact_b);
        }
    }
}
```

### Contradiction Lifecycle

```
Open → Investigating → Resolved | Accepted | Superseded
```

- **Open**: Newly detected, not yet examined
- **Investigating**: Someone is looking into it
- **Resolved**: The conflict was resolved (e.g., new evidence clarified)
- **Accepted**: Both facts are valid (e.g., temporal difference—temperature was 80C, then later 100C)
- **Superseded**: Newer evidence overrides older facts

Contradictions are persisted in the `FactLedger` and can be queried by entity reference. They surface in `VerifiedAnswer.contradicted_claims` and may generate `recommended_actions`.

## Confidence Scoring

The `DefaultEvidenceScorer::confidence_score` (`sipmem-core/src/evidence.rs:89`) computes the mean confidence across all verified claims:

```rust
fn confidence_score(&self, facts: &[AtomicFact]) -> f64 {
    if facts.is_empty() { return 0.0; }
    facts.iter().map(|f| f.confidence).sum::<f64>() / facts.len() as f64
}
```

Individual claim confidence is set by the verifier:
- `0.85` for cross-reference-verified claims (claim text found in evidence)
- `0.10` for unsupported claims
- Set by specialist verifiers for domain-specific checks

The overall `VerifiedAnswer.confidence` is the average of verified claim confidences.

## VerificationEngine: Plugin Architecture

The `VerificationEngine` (`sipmem-core/src/verification.rs:7`) holds a collection of pluggable `Verifier` implementations:

```rust
pub struct VerificationEngine {
    verifiers: Vec<Box<dyn Verifier>>,
    scorer: Box<dyn EvidenceScorer>,
}

impl VerificationEngine {
    pub fn add_verifier(&mut self, verifier: Box<dyn Verifier>);
    pub async fn verify_claims(&self, claims, evidence) -> Result<Vec<AtomicFact>>;
    pub async fn detect_contradictions(&self, claims) -> Result<Vec<ContradictionRecord>>;
}
```

Verifiers are called in registration order. Each verifier receives the output of the previous verifier, so later verifiers can refine classifications made by earlier ones.

If no verifiers are registered, the engine marks all facts as `Pending`—no claims are assumed true.

## Default Verifiers

### NoopVerifier
Marks everything `Pending`. Used as a safe default when no real verifier is configured.

### BasicVerifier
Entity-reference-based verification. A fact is:
- `Verified` if ≥2 evidence entries reference the same entity type
- `PartiallyVerified` if exactly 1 evidence entry references the same entity type
- `Unsupported` if no evidence references the same entity type

Includes contradiction detection by predicate + value comparison.

### CrossReferenceVerifier
Text-overlap-based verification. A fact is `Verified` if its claim text appears as a substring in any evidence entry's content. Otherwise `Unsupported`.

## Classification into Triage Buckets

After verification, claims are sorted into three buckets (`SipmemPipeline::classify_claims`, `pipeline.rs:130`):

| Bucket | Criteria | Example |
|--------|----------|---------|
| `verified_claims` | Status == `Verified` | "Pump A failed on 2026-04-15" with matching work order evidence |
| `unsupported_claims` | Status == `Unsupported` or `Pending` or `PartiallyVerified` | "Pump A needs a new impeller" with no service record to back it |
| `contradicted_claims` | Status == `Contradicted` | "Temperature was 100C" vs "Temperature was 80C" from same sensor |

The `VerifiedAnswer.verification_status` is derived from the triage outcome:
- `Verified` if there are verified claims and no contradictions
- `PartiallyVerified` if there are verified claims
- `Unsupported` otherwise

## Answer Synthesis

The `synthesize()` method produces the final answer text:

1. **Verified claims** are listed first: "Based on verified evidence (N sources):"
2. **Unsupported claims** are included as "Additional context" only if `verification_level != None`
3. **No evidence** returns: "No evidence found to answer this query."
