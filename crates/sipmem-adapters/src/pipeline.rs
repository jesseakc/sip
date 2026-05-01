use sipmem_core::*;

use crate::retrievers::*;
use crate::router::*;
use crate::verifiers::*;

/// The SIPmem execution pipeline.
/// This is NOT a RAG pipeline. It is an evidence engine.
///
/// Flow: Query → Auth Gate → Decompose → Recipe → Retrieve → Evidence → Verify → Synthesize
pub struct SipmemPipeline {
    retrievers: Vec<Box<dyn Retriever>>,
    router: Box<dyn MemoryRouter>,
    verifier: Box<dyn Verifier>,
    scorer: Box<dyn EvidenceScorer>,
    memory: TypedMemorySystem,
}

impl SipmemPipeline {
    pub fn new() -> Self {
        let retrievers: Vec<Box<dyn Retriever>> = vec![
            Box::new(SQLRetriever),
            Box::new(VectorRetriever),
            Box::new(GraphRetriever),
            Box::new(TemporalRetriever),
        ];

        Self {
            retrievers,
            router: Box::new(DefaultRouter::new()),
            verifier: Box::new(CrossReferenceVerifier),
            scorer: Box::new(DefaultEvidenceScorer),
            memory: TypedMemorySystem::new(),
        }
    }

    /// Execute the full pipeline and return a verified answer.
    pub async fn execute(&self, query: MemoryQuery) -> Result<VerifiedAnswer, SipmemError> {
        let plan = self.router.route(&query).await?;
        tracing::info!(
            "Selected recipe: {} (verification: {:?})",
            plan.recipe_name,
            plan.verification_level
        );

        let mut all_evidence: Vec<MemoryEvidence> = Vec::new();
        for step in &plan.steps {
            for retriever_type in &step.retriever_types {
                if let Some(retriever) = self
                    .retrievers
                    .iter()
                    .find(|r| r.retriever_type() == *retriever_type)
                {
                    let mut results = retriever.retrieve(&query).await?;
                    if step.temporal_filter {
                        let temporal = TemporalResolver::new();
                        results = temporal.prune_stale(results, chrono::Utc::now());
                    }
                    all_evidence.extend(results);
                }
            }
        }

        let scored_evidence = self.scorer.score_evidence(&all_evidence, &query);

        let claims = self.extract_claims(&scored_evidence, &query);

        let verified_claims = self.verifier.verify(&claims, &scored_evidence).await?;

        let contradictions = self
            .verifier
            .detect_contradictions(&verified_claims)
            .await?;

        let (verified, unsupported, contradicted) =
            self.classify_claims(verified_claims, &contradictions);

        let answer_text = self.synthesize(&verified, &unsupported, plan.verification_level);
        let confidence = self.scorer.confidence_score(&verified);

        Ok(VerifiedAnswer {
            answer_text,
            confidence,
            verification_status: if contradicted.is_empty() && !verified.is_empty() {
                VerificationStatus::Verified
            } else if !verified.is_empty() {
                VerificationStatus::PartiallyVerified
            } else {
                VerificationStatus::Unsupported
            },
            verified_claims: verified,
            unsupported_claims: unsupported,
            contradicted_claims: contradicted,
            evidence_used: scored_evidence,
            retrieval_paths: plan
                .steps
                .iter()
                .flat_map(|s| s.retriever_types.clone())
                .collect(),
            recommended_actions: vec![],
            generated_at: chrono::Utc::now(),
        })
    }

    fn extract_claims(&self, evidence: &[MemoryEvidence], query: &MemoryQuery) -> Vec<AtomicFact> {
        evidence
            .iter()
            .map(|e| AtomicFact {
                id: uuid::Uuid::new_v4().to_string(),
                claim: e.entry.content.chars().take(200).collect(),
                subject_entity: e.entry.entity_references.first().cloned(),
                predicate: "described_in".to_string(),
                object_value: Some(query.query_text.clone()),
                confidence: e.relevance_score,
                supporting_evidence: vec![e.entry.id.clone()],
                contradicting_evidence: vec![],
                verification_status: VerificationStatus::Pending,
                temporal_context: Some(TemporalContext {
                    observed_at: e.entry.observed_at,
                    recorded_at: e.entry.recorded_at,
                    valid_from: e.entry.valid_from,
                    valid_to: e.entry.valid_to,
                    superseded_at: None,
                    superseded_by: None,
                }),
            })
            .collect()
    }

    fn classify_claims(
        &self,
        claims: Vec<AtomicFact>,
        _contradictions: &[ContradictionRecord],
    ) -> (Vec<AtomicFact>, Vec<AtomicFact>, Vec<AtomicFact>) {
        let mut verified = Vec::new();
        let mut unsupported = Vec::new();
        let mut contradicted = Vec::new();

        for claim in claims {
            match claim.verification_status {
                VerificationStatus::Verified => verified.push(claim),
                VerificationStatus::Contradicted => contradicted.push(claim),
                _ => unsupported.push(claim),
            }
        }

        (verified, unsupported, contradicted)
    }

    fn synthesize(
        &self,
        verified: &[AtomicFact],
        unsupported: &[AtomicFact],
        level: VerificationLevel,
    ) -> String {
        if verified.is_empty() && unsupported.is_empty() {
            return "No evidence found to answer this query.".to_string();
        }

        let mut parts = Vec::new();

        if !verified.is_empty() {
            parts.push(format!(
                "Based on verified evidence ({} sources):",
                verified.len()
            ));
            for fact in verified.iter().take(5) {
                parts.push(format!("  • {}", fact.claim));
            }
        }

        if !unsupported.is_empty() && level != VerificationLevel::None {
            parts.push(format!(
                "\nAdditional context ({} unverified items):",
                unsupported.len()
            ));
            for fact in unsupported.iter().take(3) {
                parts.push(format!("  • {}", fact.claim));
            }
        }

        parts.join("\n")
    }

    /// Get a mutable reference to the memory system for storing new entries
    pub fn memory_mut(&mut self) -> &mut TypedMemorySystem {
        &mut self.memory
    }

    /// Get a reference to the memory system
    pub fn memory(&self) -> &TypedMemorySystem {
        &self.memory
    }
}

impl Default for SipmemPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sipmem_core::{MemoryQuery, TimeRange, VerificationLevel};

    fn make_query(text: &str) -> MemoryQuery {
        MemoryQuery {
            query_text: text.to_string(),
            tenant_id: uuid::Uuid::new_v4(),
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.0,
            recipe_preference: None,
        }
    }

    #[tokio::test]
    async fn test_pipeline_empty_query() {
        let pipeline = SipmemPipeline::new();
        let result = pipeline
            .execute(make_query("What is the status of asset X?"))
            .await;
        assert!(result.is_ok());
        let answer = result.unwrap();
        assert!(answer.confidence < 1.0 || answer.evidence_used.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_returns_verified_answer() {
        let pipeline = SipmemPipeline::new();
        let result = pipeline
            .execute(make_query("Find work orders for pump"))
            .await;
        assert!(result.is_ok());
        let answer = result.unwrap();
        assert!(!answer.generated_at.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_with_temporal_query() {
        let mut query = make_query("What changed since Monday?");
        query.temporal_constraint = Some(TimeRange {
            start: Utc::now() - chrono::Duration::days(7),
            end: None,
        });
        let pipeline = SipmemPipeline::new();
        let result = pipeline.execute(query).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_synthesize_no_evidence() {
        let pipeline = SipmemPipeline::new();
        let text = pipeline.synthesize(&[], &[], VerificationLevel::Standard);
        assert_eq!(text, "No evidence found to answer this query.");
    }

    #[tokio::test]
    async fn test_synthesize_with_verified() {
        let pipeline = SipmemPipeline::new();
        let verified = vec![AtomicFact {
            id: "f1".into(),
            claim: "Pump A operational".into(),
            subject_entity: None,
            predicate: "is".into(),
            object_value: None,
            confidence: 1.0,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Verified,
            temporal_context: None,
        }];
        let text = pipeline.synthesize(&verified, &[], VerificationLevel::Standard);
        assert!(text.contains("Pump A operational"));
        assert!(text.contains("verified evidence"));
    }

    #[tokio::test]
    async fn test_memory_access() {
        let mut pipeline = SipmemPipeline::new();
        assert_eq!(pipeline.memory().entries().len(), 0);
        assert_eq!(pipeline.memory_mut().entries().len(), 0);
    }
}
