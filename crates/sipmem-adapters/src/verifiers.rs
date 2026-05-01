use async_trait::async_trait;
use sipmem_core::{
    AtomicFact, ContradictionRecord, MemoryEvidence, SipmemError, VerificationStatus, Verifier,
};

/// Cross-references facts against evidence by checking if claim text appears in evidence content.
pub struct CrossReferenceVerifier;

#[async_trait]
impl Verifier for CrossReferenceVerifier {
    async fn verify(
        &self,
        facts: &[AtomicFact],
        evidence: &[MemoryEvidence],
    ) -> Result<Vec<AtomicFact>, SipmemError> {
        let mut verified = Vec::new();
        for fact in facts {
            let mut fact = fact.clone();
            let has_support = evidence
                .iter()
                .any(|e| e.entry.content.contains(&fact.claim));
            fact.verification_status = if has_support {
                VerificationStatus::Verified
            } else {
                VerificationStatus::Unsupported
            };
            fact.confidence = if has_support { 0.85 } else { 0.1 };
            verified.push(fact);
        }
        Ok(verified)
    }

    async fn detect_contradictions(
        &self,
        _facts: &[AtomicFact],
    ) -> Result<Vec<ContradictionRecord>, SipmemError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sipmem_core::{
        EntityReference, EvidenceSource, MemoryCategory, MemoryEntry, MemoryId, RetrievalType,
    };

    fn make_evidence(id: &str, content: &str) -> MemoryEvidence {
        MemoryEvidence {
            entry: MemoryEntry {
                id: MemoryId(id.into()),
                category: MemoryCategory::VerifiedFact,
                tenant_id: uuid::Uuid::nil(),
                content: content.into(),
                metadata: serde_json::json!({}),
                entity_references: vec![],
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
            matches_query_entities: false,
            temporal_match: false,
        }
    }

    #[tokio::test]
    async fn test_cross_reference_verifies_supported_fact() {
        let verifier = CrossReferenceVerifier;
        let evidence = vec![make_evidence("e1", "Pump A failed on Tuesday")];
        let facts = vec![AtomicFact {
            id: "f1".into(),
            claim: "Pump A failed".into(),
            subject_entity: None,
            predicate: "failed".into(),
            object_value: None,
            confidence: 0.5,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Pending,
            temporal_context: None,
        }];

        let result = verifier.verify(&facts, &evidence).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].verification_status, VerificationStatus::Verified);
        assert!((result[0].confidence - 0.85).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cross_reference_marks_unsupported_fact() {
        let verifier = CrossReferenceVerifier;
        let evidence = vec![make_evidence("e1", "unrelated content")];
        let facts = vec![AtomicFact {
            id: "f1".into(),
            claim: "Pump B exploded".into(),
            subject_entity: None,
            predicate: "exploded".into(),
            object_value: None,
            confidence: 0.5,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Pending,
            temporal_context: None,
        }];

        let result = verifier.verify(&facts, &evidence).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].verification_status, VerificationStatus::Unsupported);
        assert!((result[0].confidence - 0.1).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_detect_contradictions_returns_empty() {
        let verifier = CrossReferenceVerifier;
        let facts = vec![AtomicFact {
            id: "f1".into(),
            claim: "test".into(),
            subject_entity: Some(EntityReference {
                entity_type: "asset".into(),
                entity_id: uuid::Uuid::new_v4(),
                relationship: "direct".into(),
            }),
            predicate: "test".into(),
            object_value: None,
            confidence: 0.9,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Pending,
            temporal_context: None,
        }];
        let result = verifier.detect_contradictions(&facts).await.unwrap();
        assert!(result.is_empty());
    }
}
