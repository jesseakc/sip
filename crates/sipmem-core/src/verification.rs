use crate::memory::ContradictionRecord;
use crate::traits::{EvidenceScorer, SipmemError, Verifier};
use crate::types::*;

/// The VerificationEngine is the core of SIPmem's "evidence engine" philosophy.
/// It does NOT trust retrieved context as truth.
pub struct VerificationEngine {
    verifiers: Vec<Box<dyn Verifier>>,
    scorer: Box<dyn EvidenceScorer>,
}

impl VerificationEngine {
    pub fn new(scorer: Box<dyn EvidenceScorer>) -> Self {
        Self {
            verifiers: Vec::new(),
            scorer,
        }
    }

    pub fn add_verifier(&mut self, verifier: Box<dyn Verifier>) {
        self.verifiers.push(verifier);
    }

    pub async fn verify_claims(
        &self,
        claims: &[AtomicFact],
        evidence: &[MemoryEvidence],
    ) -> Result<Vec<AtomicFact>, SipmemError> {
        if self.verifiers.is_empty() {
            // No verifiers registered — mark everything as Pending
            let mut facts = claims.to_vec();
            for f in &mut facts {
                f.verification_status = VerificationStatus::Pending;
            }
            return Ok(facts);
        }

        let mut facts = claims.to_vec();
        for verifier in &self.verifiers {
            facts = verifier.verify(&facts, evidence).await?;
        }
        Ok(facts)
    }

    pub async fn detect_contradictions(
        &self,
        claims: &[AtomicFact],
    ) -> Result<Vec<ContradictionRecord>, SipmemError> {
        let mut all_contradictions = Vec::new();
        for verifier in &self.verifiers {
            let contradictions = verifier.detect_contradictions(claims).await?;
            all_contradictions.extend(contradictions);
        }
        Ok(all_contradictions)
    }

    pub fn scorer(&self) -> &dyn EvidenceScorer {
        self.scorer.as_ref()
    }
}

/// A default verifier that marks facts as Pending when no external verifier is registered.
#[derive(Default)]
pub struct NoopVerifier;

#[async_trait::async_trait]
impl Verifier for NoopVerifier {
    async fn verify(
        &self,
        facts: &[AtomicFact],
        _evidence: &[MemoryEvidence],
    ) -> Result<Vec<AtomicFact>, SipmemError> {
        let mut result = facts.to_vec();
        for f in &mut result {
            f.verification_status = VerificationStatus::Pending;
        }
        Ok(result)
    }

    async fn detect_contradictions(
        &self,
        _facts: &[AtomicFact],
    ) -> Result<Vec<ContradictionRecord>, SipmemError> {
        Ok(vec![])
    }
}

/// A basic verifier that uses evidence presence to verify facts.
pub struct BasicVerifier;

#[async_trait::async_trait]
impl Verifier for BasicVerifier {
    async fn verify(
        &self,
        facts: &[AtomicFact],
        evidence: &[MemoryEvidence],
    ) -> Result<Vec<AtomicFact>, SipmemError> {
        let mut result = facts.to_vec();
        for fact in &mut result {
            let supporting_count = evidence
                .iter()
                .filter(|e| {
                    if let Some(ref subject) = fact.subject_entity {
                        e.entry
                            .entity_references
                            .iter()
                            .any(|er| er.entity_type == subject.entity_type)
                    } else {
                        false
                    }
                })
                .count();

            fact.verification_status = if supporting_count >= 2 {
                VerificationStatus::Verified
            } else if supporting_count == 1 {
                VerificationStatus::PartiallyVerified
            } else {
                VerificationStatus::Unsupported
            };
        }
        Ok(result)
    }

    async fn detect_contradictions(
        &self,
        facts: &[AtomicFact],
    ) -> Result<Vec<ContradictionRecord>, SipmemError> {
        let mut contradictions = Vec::new();
        for (i, fact_a) in facts.iter().enumerate() {
            for fact_b in facts.iter().skip(i + 1) {
                // Simple heuristic: same subject but different predicate values
                if let (Some(ref sub_a), Some(ref sub_b)) =
                    (&fact_a.subject_entity, &fact_b.subject_entity)
                {
                    if sub_a.entity_id == sub_b.entity_id
                        && fact_a.predicate == fact_b.predicate
                        && fact_a.object_value != fact_b.object_value
                    {
                        contradictions.push(ContradictionRecord {
                            id: format!("contra-{}-{}", fact_a.id, fact_b.id),
                            claim: format!(
                                "Contradiction: '{}' vs '{}' for {}",
                                fact_a.claim, fact_b.claim, fact_a.predicate
                            ),
                            conflicting_sources: vec![],
                            supporting_sources: vec![],
                            entities: vec![sub_a.clone()],
                            time_range: None,
                            status: crate::memory::ContradictionStatus::Open,
                            resolution_notes: None,
                            detected_at: chrono::Utc::now(),
                            resolved_at: None,
                        });
                    }
                }
            }
        }
        Ok(contradictions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::DefaultEvidenceScorer;

    #[tokio::test]
    async fn test_noop_verifier_marks_pending() {
        let engine = VerificationEngine::new(Box::new(DefaultEvidenceScorer));
        let facts = vec![AtomicFact {
            id: "f1".into(),
            claim: "test".into(),
            subject_entity: None,
            predicate: "is".into(),
            object_value: None,
            confidence: 0.9,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Verified,
            temporal_context: None,
        }];
        // Without any verifier added, engine marks as Pending
        let result = engine.verify_claims(&facts, &[]).await.unwrap();
        assert!(result[0].verification_status == VerificationStatus::Pending);
    }

    #[tokio::test]
    async fn test_basic_verifier_with_evidence() {
        let mut engine = VerificationEngine::new(Box::new(DefaultEvidenceScorer));
        engine.add_verifier(Box::new(BasicVerifier));

        let eid = uuid::Uuid::new_v4();
        let facts = vec![AtomicFact {
            id: "f1".into(),
            claim: "pump failed".into(),
            subject_entity: Some(EntityReference {
                entity_type: "asset".into(),
                entity_id: eid,
                relationship: "direct".into(),
            }),
            predicate: "failed".into(),
            object_value: None,
            confidence: 0.9,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Pending,
            temporal_context: None,
        }];
        let evidence = vec![MemoryEvidence {
            entry: MemoryEntry {
                id: MemoryId("e1".into()),
                category: MemoryCategory::Asset,
                tenant_id: uuid::Uuid::nil(),
                content: "pump failure".into(),
                metadata: serde_json::json!({}),
                entity_references: vec![EntityReference {
                    entity_type: "asset".into(),
                    entity_id: eid,
                    relationship: "direct".into(),
                }],
                observed_at: chrono::Utc::now(),
                recorded_at: chrono::Utc::now(),
                valid_from: chrono::Utc::now(),
                valid_to: None,
                confidence: 1.0,
                source: EvidenceSource {
                    source_type: "test".into(),
                    source_id: "s1".into(),
                    retrieval_type: RetrievalType::SQL,
                    reliability_score: 1.0,
                },
                verification_status: VerificationStatus::Verified,
            },
            relevance_score: 1.0,
            matches_query_entities: true,
            temporal_match: false,
        }];

        let result = engine.verify_claims(&facts, &evidence).await.unwrap();
        assert_eq!(
            result[0].verification_status,
            VerificationStatus::PartiallyVerified
        );
    }

    #[tokio::test]
    async fn test_basic_verifier_detects_contradictions() {
        let mut engine = VerificationEngine::new(Box::new(DefaultEvidenceScorer));
        engine.add_verifier(Box::new(BasicVerifier));

        let eid = uuid::Uuid::new_v4();
        let facts = vec![
            AtomicFact {
                id: "f1".into(),
                claim: "temperature is 100C".into(),
                subject_entity: Some(EntityReference {
                    entity_type: "sensor".into(),
                    entity_id: eid,
                    relationship: "direct".into(),
                }),
                predicate: "temperature".into(),
                object_value: Some("100C".into()),
                confidence: 0.9,
                supporting_evidence: vec![],
                contradicting_evidence: vec![],
                verification_status: VerificationStatus::Pending,
                temporal_context: None,
            },
            AtomicFact {
                id: "f2".into(),
                claim: "temperature is 80C".into(),
                subject_entity: Some(EntityReference {
                    entity_type: "sensor".into(),
                    entity_id: eid,
                    relationship: "direct".into(),
                }),
                predicate: "temperature".into(),
                object_value: Some("80C".into()),
                confidence: 0.9,
                supporting_evidence: vec![],
                contradicting_evidence: vec![],
                verification_status: VerificationStatus::Pending,
                temporal_context: None,
            },
        ];

        let contradictions = engine.detect_contradictions(&facts).await.unwrap();
        assert_eq!(contradictions.len(), 1);
    }
}
