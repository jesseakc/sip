use serde::{Deserialize, Serialize};

use crate::types::*;

/// A persisted contradiction between facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionRecord {
    pub id: String,
    pub claim: String,
    pub conflicting_sources: Vec<EvidenceSource>,
    pub supporting_sources: Vec<EvidenceSource>,
    pub entities: Vec<EntityReference>,
    pub time_range: Option<TimeRange>,
    pub status: ContradictionStatus,
    pub resolution_notes: Option<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContradictionStatus {
    Open,
    Investigating,
    Resolved,
    Accepted,
    Superseded,
}

/// The canonical fact ledger — all knowledge reduces to atomic facts.
pub struct FactLedger {
    facts: Vec<AtomicFact>,
    contradictions: Vec<ContradictionRecord>,
}

impl FactLedger {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            contradictions: Vec::new(),
        }
    }

    pub fn record_fact(&mut self, fact: AtomicFact) {
        self.facts.push(fact);
    }

    pub fn record_contradiction(&mut self, contradiction: ContradictionRecord) {
        self.contradictions.push(contradiction);
    }

    pub fn query_facts(
        &self,
        entity: &EntityReference,
        time: Option<&TimeRange>,
    ) -> Vec<&AtomicFact> {
        self.facts
            .iter()
            .filter(|f| {
                let entity_match = f.subject_entity.as_ref().map_or(false, |e| {
                    e.entity_type == entity.entity_type && e.entity_id == entity.entity_id
                });
                if !entity_match {
                    return false;
                }
                if let Some(range) = time {
                    if let Some(ref tc) = f.temporal_context {
                        let end_limit = range.end.unwrap_or(chrono::Utc::now());
                        tc.valid_from <= end_limit
                            && tc.valid_to.map(|vt| vt >= range.start).unwrap_or(true)
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn get_contradictions(&self, entity: &EntityReference) -> Vec<&ContradictionRecord> {
        self.contradictions
            .iter()
            .filter(|c| {
                c.entities.iter().any(|e| {
                    e.entity_type == entity.entity_type && e.entity_id == entity.entity_id
                })
            })
            .collect()
    }

    pub fn get_facts_by_status(&self, status: VerificationStatus) -> Vec<&AtomicFact> {
        self.facts
            .iter()
            .filter(|f| f.verification_status == status)
            .collect()
    }
}

impl Default for FactLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// The TypedMemorySystem is the central memory store.
/// It holds typed MemoryEntry objects indexed by category, entity, and time.
pub struct TypedMemorySystem {
    entries: Vec<MemoryEntry>,
    fact_ledger: FactLedger,
}

impl TypedMemorySystem {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            fact_ledger: FactLedger::new(),
        }
    }

    pub fn store(&mut self, entry: MemoryEntry) -> MemoryId {
        let id = entry.id.clone();
        self.entries.push(entry);
        id
    }

    pub fn query(&self, query: &MemoryQuery) -> Vec<MemoryEvidence> {
        self.entries
            .iter()
            .filter(|e| e.tenant_id == query.tenant_id)
            .filter(|e| e.confidence >= query.min_confidence)
            .filter(|e| {
                if query.entity_filters.is_empty() {
                    return true;
                }
                query.entity_filters.iter().any(|f| {
                    e.entity_references.iter().any(|er| {
                        er.entity_type == f.entity_type && er.entity_id == f.entity_id
                    })
                })
            })
            .filter(|e| {
                if let Some(ref tc) = query.temporal_constraint {
                    let end_limit = tc.end.unwrap_or(chrono::Utc::now());
                    e.valid_from <= end_limit
                        && e.valid_to.map(|vt| vt >= tc.start).unwrap_or(true)
                } else {
                    true
                }
            })
            .map(|e| MemoryEvidence {
                entry: e.clone(),
                relevance_score: 1.0,
                matches_query_entities: !query.entity_filters.is_empty()
                    && query.entity_filters.iter().any(|f| {
                        e.entity_references.iter().any(|er| {
                            er.entity_type == f.entity_type && er.entity_id == f.entity_id
                        })
                    }),
                temporal_match: query.temporal_constraint.is_some(),
            })
            .take(query.max_results.max(1))
            .collect()
    }

    pub fn get_facts(&self) -> &FactLedger {
        &self.fact_ledger
    }

    pub fn get_facts_mut(&mut self) -> &mut FactLedger {
        &mut self.fact_ledger
    }

    pub fn entries(&self) -> &[MemoryEntry] {
        &self.entries
    }
}

impl Default for TypedMemorySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_entry(id: &str, tenant: Uuid, confidence: f64) -> MemoryEntry {
        MemoryEntry {
            id: MemoryId(id.into()),
            category: MemoryCategory::Asset,
            tenant_id: tenant,
            content: format!("content-{}", id),
            metadata: serde_json::json!({}),
            entity_references: vec![],
            observed_at: chrono::Utc::now(),
            recorded_at: chrono::Utc::now(),
            valid_from: chrono::Utc::now(),
            valid_to: None,
            confidence,
            source: EvidenceSource {
                source_type: "test".into(),
                source_id: "src-1".into(),
                retrieval_type: RetrievalType::SQL,
                reliability_score: 0.9,
            },
            verification_status: VerificationStatus::Verified,
        }
    }

    #[test]
    fn test_store_and_query() {
        let mut sys = TypedMemorySystem::new();
        let tenant = Uuid::new_v4();
        sys.store(make_entry("e1", tenant, 0.8));
        sys.store(make_entry("e2", tenant, 0.3));

        let query = MemoryQuery {
            query_text: "test".into(),
            tenant_id: tenant,
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.5,
            recipe_preference: None,
        };
        let results = sys.query(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, MemoryId("e1".into()));
    }

    #[test]
    fn test_query_different_tenant() {
        let mut sys = TypedMemorySystem::new();
        let t1 = Uuid::new_v4();
        let t2 = Uuid::new_v4();
        sys.store(make_entry("e1", t1, 0.8));

        let query = MemoryQuery {
            query_text: "test".into(),
            tenant_id: t2,
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.0,
            recipe_preference: None,
        };
        let results = sys.query(&query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fact_ledger_record_and_query() {
        let mut ledger = FactLedger::new();
        let fact = AtomicFact {
            id: "f1".into(),
            claim: "Pump A failed".into(),
            subject_entity: Some(EntityReference {
                entity_type: "asset".into(),
                entity_id: Uuid::new_v4(),
                relationship: "direct".into(),
            }),
            predicate: "failed".into(),
            object_value: None,
            confidence: 0.9,
            supporting_evidence: vec![],
            contradicting_evidence: vec![],
            verification_status: VerificationStatus::Verified,
            temporal_context: None,
        };
        ledger.record_fact(fact);
        assert_eq!(ledger.facts.len(), 1);
        assert_eq!(ledger.get_facts_by_status(VerificationStatus::Verified).len(), 1);
        assert_eq!(ledger.get_facts_by_status(VerificationStatus::Pending).len(), 0);
    }

    #[test]
    fn test_contradiction_recording() {
        let mut ledger = FactLedger::new();
        let e1 = EntityReference {
            entity_type: "asset".into(),
            entity_id: Uuid::nil(),
            relationship: "direct".into(),
        };
        let cr = ContradictionRecord {
            id: "cr1".into(),
            claim: "Conflicting pressure readings".into(),
            conflicting_sources: vec![],
            supporting_sources: vec![],
            entities: vec![e1.clone()],
            time_range: None,
            status: ContradictionStatus::Open,
            resolution_notes: None,
            detected_at: chrono::Utc::now(),
            resolved_at: None,
        };
        ledger.record_contradiction(cr);
        let results = ledger.get_contradictions(&e1);
        assert_eq!(results.len(), 1);
    }
}
