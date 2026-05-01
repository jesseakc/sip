use crate::traits::EvidenceScorer;
use crate::types::*;

/// Candidate evidence is NOT truth. It must be verified.
pub struct EvidenceMatcher;

impl EvidenceMatcher {
    pub fn new() -> Self {
        Self
    }

    /// Match atomic facts against candidate evidence.
    /// Returns each fact paired with the evidence that supports it.
    pub fn match_evidence(
        &self,
        facts: &[AtomicFact],
        candidates: &[MemoryEvidence],
    ) -> Vec<(AtomicFact, Vec<MemoryEvidence>)> {
        facts
            .iter()
            .map(|fact| {
                let matched: Vec<MemoryEvidence> = candidates
                    .iter()
                    .filter(|candidate| {
                        // Match by shared entity references
                        if let Some(ref subject) = fact.subject_entity {
                            candidate.entry.entity_references.iter().any(|er| {
                                er.entity_type == subject.entity_type
                                    && er.entity_id == subject.entity_id
                            })
                        } else {
                            // No subject entity — match by text overlap
                            candidate
                                .entry
                                .content
                                .to_lowercase()
                                .contains(&fact.claim.to_lowercase())
                        }
                    })
                    .cloned()
                    .collect();
                (fact.clone(), matched)
            })
            .collect()
    }

    /// Score how well a fact matches a piece of evidence.
    pub fn score_match(&self, fact: &AtomicFact, evidence: &MemoryEvidence) -> f64 {
        let entity_match = if let Some(ref subject) = fact.subject_entity {
            evidence.entry.entity_references.iter().any(|er| {
                er.entity_type == subject.entity_type && er.entity_id == subject.entity_id
            }) as u8 as f64
        } else {
            0.5
        };

        let confidence_factor = fact.confidence * evidence.entry.confidence;
        0.4 * entity_match + 0.6 * confidence_factor
    }
}

impl Default for EvidenceMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Default evidence scorer
pub struct DefaultEvidenceScorer;

impl EvidenceScorer for DefaultEvidenceScorer {
    fn score_evidence(
        &self,
        evidence: &[MemoryEvidence],
        _query: &MemoryQuery,
    ) -> Vec<MemoryEvidence> {
        let mut scored: Vec<MemoryEvidence> = evidence.to_vec();
        scored.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }

    fn confidence_score(&self, facts: &[AtomicFact]) -> f64 {
        if facts.is_empty() {
            return 0.0;
        }
        let sum: f64 = facts.iter().map(|f| f.confidence).sum();
        sum / facts.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_evidence(content: &str, entity_type: &str, entity_id: Uuid) -> MemoryEvidence {
        MemoryEvidence {
            entry: MemoryEntry {
                id: MemoryId(format!("e-{}", content)),
                category: MemoryCategory::Asset,
                tenant_id: Uuid::nil(),
                content: content.into(),
                metadata: serde_json::json!({}),
                entity_references: vec![EntityReference {
                    entity_type: entity_type.into(),
                    entity_id,
                    relationship: "direct".into(),
                }],
                observed_at: chrono::Utc::now(),
                recorded_at: chrono::Utc::now(),
                valid_from: chrono::Utc::now(),
                valid_to: None,
                confidence: 0.8,
                source: EvidenceSource {
                    source_type: "test".into(),
                    source_id: "s1".into(),
                    retrieval_type: RetrievalType::SQL,
                    reliability_score: 0.9,
                },
                verification_status: VerificationStatus::Verified,
            },
            relevance_score: 0.8,
            matches_query_entities: true,
            temporal_match: false,
        }
    }

    #[test]
    fn test_match_evidence_by_entity() {
        let matcher = EvidenceMatcher::new();
        let eid = Uuid::new_v4();
        let fact = AtomicFact {
            id: "f1".into(),
            claim: "Pump A failed".into(),
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
        };
        let evidence = make_evidence("Pump A failed on Tuesday", "asset", eid);
        let matches = matcher.match_evidence(&[fact], &[evidence]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1.len(), 1);
    }

    #[test]
    fn test_match_evidence_no_entity() {
        let matcher = EvidenceMatcher::new();
        let fact = AtomicFact {
            id: "f2".into(),
            claim: "temperature exceeded threshold".into(),
            subject_entity: None,
            predicate: "exceeded".into(),
            object_value: None,
            confidence: 0.9,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Pending,
            temporal_context: None,
        };
        let evidence = make_evidence(
            "temperature exceeded threshold at sensor 3",
            "sensor",
            Uuid::new_v4(),
        );
        let matches = matcher.match_evidence(&[fact], &[evidence]);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1.len(), 1);
    }

    #[test]
    fn test_score_match_perfect() {
        let matcher = EvidenceMatcher::new();
        let eid = Uuid::new_v4();
        let fact = AtomicFact {
            id: "f3".into(),
            claim: "bearing worn".into(),
            subject_entity: Some(EntityReference {
                entity_type: "asset".into(),
                entity_id: eid,
                relationship: "direct".into(),
            }),
            predicate: "worn".into(),
            object_value: None,
            confidence: 1.0,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Pending,
            temporal_context: None,
        };
        let evidence = make_evidence("bearing worn", "asset", eid);
        let score = matcher.score_match(&fact, &evidence);
        assert!(score > 0.8);
    }

    #[test]
    fn test_default_scorer_confidence() {
        let scorer = DefaultEvidenceScorer;
        let facts = vec![
            AtomicFact {
                id: "a".into(),
                claim: "a".into(),
                subject_entity: None,
                predicate: "is".into(),
                object_value: None,
                confidence: 0.5,
                supporting_evidence: vec![],
                contradicting_evidence: vec![],
                verification_status: VerificationStatus::Pending,
                temporal_context: None,
            },
            AtomicFact {
                id: "b".into(),
                claim: "b".into(),
                subject_entity: None,
                predicate: "is".into(),
                object_value: None,
                confidence: 0.9,
                supporting_evidence: vec![],
                contradicting_evidence: vec![],
                verification_status: VerificationStatus::Pending,
                temporal_context: None,
            },
        ];
        let score = scorer.confidence_score(&facts);
        assert!((score - 0.7).abs() < 0.001);
    }
}
