use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Memory categories. SIPmem does NOT store generic "documents".
/// Every memory entry has an explicit type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryCategory {
    Asset,
    ServiceCase,
    TechnicianNote,
    ManualSection,
    ServiceBulletin,
    FirmwareFact,
    PartReplacement,
    KnownIssue,
    Procedure,
    FailureMode,
    RootCause,
    TelemetrySummary,
    VerifiedFact,
    ContradictionRecord,
    InspectionFinding,
    DocumentChunk,
}

/// Unique identifier for a memory entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

/// A reference to a domain entity (asset, work order, part, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReference {
    pub entity_type: String,
    pub entity_id: uuid::Uuid,
    pub relationship: String,
}

/// Verification status for an entry or fact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    PartiallyVerified,
    Unsupported,
    Contradicted,
    Pending,
}

/// How evidence was retrieved
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalType {
    SQL,
    Vector,
    Graph,
    Temporal,
    Hybrid,
    Plugin,
}

/// The source of a piece of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSource {
    pub source_type: String,
    pub source_id: String,
    pub retrieval_type: RetrievalType,
    pub reliability_score: f64,
}

/// A time range for temporal queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: chrono::DateTime<Utc>,
    pub end: Option<chrono::DateTime<Utc>>,
}

/// Temporal metadata for a fact or entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub observed_at: chrono::DateTime<Utc>,
    pub recorded_at: chrono::DateTime<Utc>,
    pub valid_from: chrono::DateTime<Utc>,
    pub valid_to: Option<chrono::DateTime<Utc>>,
    pub superseded_at: Option<chrono::DateTime<Utc>>,
    pub superseded_by: Option<MemoryId>,
}

/// An atomic, typed memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub category: MemoryCategory,
    pub tenant_id: uuid::Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
    pub entity_references: Vec<EntityReference>,
    pub observed_at: chrono::DateTime<Utc>,
    pub recorded_at: chrono::DateTime<Utc>,
    pub valid_from: chrono::DateTime<Utc>,
    pub valid_to: Option<chrono::DateTime<Utc>>,
    pub confidence: f64,
    pub source: EvidenceSource,
    pub verification_status: VerificationStatus,
}

/// An atomic fact extracted from memory for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A query to the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQuery {
    pub query_text: String,
    pub tenant_id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub entity_filters: Vec<EntityReference>,
    pub temporal_constraint: Option<TimeRange>,
    pub max_results: usize,
    pub min_confidence: f64,
    pub recipe_preference: Option<String>,
}

/// Result from a retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvidence {
    pub entry: MemoryEntry,
    pub relevance_score: f64,
    pub matches_query_entities: bool,
    pub temporal_match: bool,
}

/// The final, verified answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedAnswer {
    pub answer_text: String,
    pub confidence: f64,
    pub verification_status: VerificationStatus,
    pub verified_claims: Vec<AtomicFact>,
    pub unsupported_claims: Vec<AtomicFact>,
    pub contradicted_claims: Vec<AtomicFact>,
    pub evidence_used: Vec<MemoryEvidence>,
    pub retrieval_paths: Vec<RetrievalType>,
    pub recommended_actions: Vec<String>,
    pub generated_at: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_memory_id_eq() {
        let a = MemoryId("abc".into());
        let b = MemoryId("abc".into());
        assert_eq!(a, b);
    }

    #[test]
    fn test_memory_id_ne() {
        let a = MemoryId("abc".into());
        let b = MemoryId("def".into());
        assert_ne!(a, b);
    }

    #[test]
    fn test_serialize_memory_query() {
        let query = MemoryQuery {
            query_text: "failure mode of pump A".into(),
            tenant_id: Uuid::nil(),
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.5,
            recipe_preference: None,
        };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: MemoryQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.query_text, query.query_text);
    }

    #[test]
    fn test_time_range_end_none() {
        let start = chrono::Utc::now();
        let range = TimeRange {
            start,
            end: None,
        };
        assert_eq!(range.start, start);
        assert!(range.end.is_none());
    }
}
