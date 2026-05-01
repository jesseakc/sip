use serde::{Deserialize, Serialize};

use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait Retriever: Send + Sync {
    fn retriever_type(&self) -> RetrievalType;
    async fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<MemoryEvidence>, SipmemError>;
    async fn health_check(&self) -> Result<bool, SipmemError>;
}

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
    ) -> Result<Vec<crate::memory::ContradictionRecord>, SipmemError>;
}

pub trait EvidenceScorer: Send + Sync {
    fn score_evidence(
        &self,
        evidence: &[MemoryEvidence],
        query: &MemoryQuery,
    ) -> Vec<MemoryEvidence>;
    fn confidence_score(&self, facts: &[AtomicFact]) -> f64;
}

pub trait TemporalResolverTrait: Send + Sync {
    fn is_valid_at(&self, context: &TemporalContext, time: chrono::DateTime<chrono::Utc>) -> bool;
    fn is_stale(&self, context: &TemporalContext, now: chrono::DateTime<chrono::Utc>) -> bool;
    fn prune(
        &self,
        entries: Vec<MemoryEvidence>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Vec<MemoryEvidence>;
    fn filter(&self, entries: Vec<MemoryEvidence>, range: &TimeRange) -> Vec<MemoryEvidence>;
}

#[async_trait]
pub trait MemoryRouter: Send + Sync {
    async fn route(&self, query: &MemoryQuery) -> Result<ExecutionPlan, SipmemError>;
}

/// An ExecutionPlan describes which retrievers to call, in what order,
/// with what temporal filters and verification steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub query: MemoryQuery,
    pub recipe_name: String,
    pub steps: Vec<RetrievalStep>,
    pub verification_level: VerificationLevel,
    pub temporal_constraint: Option<TimeRange>,
    pub cost_budget_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalStep {
    pub retriever_types: Vec<RetrievalType>,
    pub parallel: bool,
    pub max_results_per_retriever: usize,
    pub temporal_filter: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationLevel {
    None,
    Basic,
    Standard,
    Strict,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum SipmemError {
    #[error("Retrieval error: {0}")]
    RetrievalError(String),
    #[error("Verification error: {0}")]
    VerificationError(String),
    #[error("Temporal error: {0}")]
    TemporalError(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sipmem_error_display() {
        let err = SipmemError::InvalidQuery("missing tenant".into());
        assert_eq!(err.to_string(), "Invalid query: missing tenant");
    }

    #[test]
    fn test_sipmem_error_not_implemented() {
        let err = SipmemError::NotImplemented("graph retriever".into());
        assert_eq!(err.to_string(), "Not implemented: graph retriever");
    }

    #[test]
    fn test_verification_level_eq() {
        assert_eq!(VerificationLevel::None, VerificationLevel::None);
        assert_ne!(VerificationLevel::None, VerificationLevel::Full);
    }

    #[test]
    fn test_execution_plan_defaults() {
        let plan = ExecutionPlan {
            query: MemoryQuery {
                query_text: "test".into(),
                tenant_id: uuid::Uuid::nil(),
                user_id: None,
                entity_filters: vec![],
                temporal_constraint: None,
                max_results: 10,
                min_confidence: 0.0,
                recipe_preference: None,
            },
            recipe_name: "default".into(),
            steps: vec![],
            verification_level: VerificationLevel::Basic,
            temporal_constraint: None,
            cost_budget_ms: 1000,
        };
        assert_eq!(plan.recipe_name, "default");
        assert_eq!(plan.verification_level, VerificationLevel::Basic);
    }
}
