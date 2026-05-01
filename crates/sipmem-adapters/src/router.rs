use async_trait::async_trait;
use sipmem_core::{
    ExecutionPlan, MemoryQuery, MemoryRouter, RetrievalRecipe, RetrievalStep, SipmemError,
};

/// Default router that selects recipes based on query text patterns.
pub struct DefaultRouter {
    inner: sipmem_core::RecipeRouter,
}

impl DefaultRouter {
    pub fn new() -> Self {
        let mut inner = sipmem_core::RecipeRouter::new();
        inner.register(RetrievalRecipe::exact_fact_lookup());
        inner.register(RetrievalRecipe::state_at_time());
        inner.register(RetrievalRecipe::asset_history_summary());
        inner.register(RetrievalRecipe::troubleshooting_similarity());
        inner.register(RetrievalRecipe::known_issue_investigation());
        inner.register(RetrievalRecipe::firmware_specific());
        inner.register(RetrievalRecipe::service_bulletin_check());
        inner.register(RetrievalRecipe::contradiction_detection());
        inner.register(RetrievalRecipe::root_cause_candidate());
        inner.register(RetrievalRecipe::stale_evidence_detection());
        Self { inner }
    }
}

#[async_trait]
impl MemoryRouter for DefaultRouter {
    async fn route(&self, query: &MemoryQuery) -> Result<ExecutionPlan, SipmemError> {
        let recipe = self
            .inner
            .select(query)
            .cloned()
            .unwrap_or_else(|| RetrievalRecipe::troubleshooting_similarity());

        Ok(ExecutionPlan {
            query: query.clone(),
            recipe_name: recipe.name.clone(),
            steps: vec![RetrievalStep {
                retriever_types: recipe.retrievers.clone(),
                parallel: true,
                max_results_per_retriever: 10,
                temporal_filter: recipe.temporal_constraint.is_some(),
            }],
            verification_level: recipe.verification_level,
            temporal_constraint: recipe.temporal_constraint,
            cost_budget_ms: recipe.cost_budget_ms,
        })
    }
}

impl Default for DefaultRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sipmem_core::{MemoryQuery, VerificationLevel};

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
    async fn test_router_selects_recipe() {
        let router = DefaultRouter::new();
        let plan = router.route(&make_query("root cause of pump failure")).await;
        assert!(plan.is_ok());
        let plan = plan.unwrap();
        assert_eq!(plan.recipe_name, "root_cause_candidate");
        assert!(!plan.steps.is_empty());
    }

    #[tokio::test]
    async fn test_router_fallback_to_troubleshooting() {
        let router = DefaultRouter::new();
        let query = MemoryQuery {
            query_text: "generic search".into(),
            tenant_id: uuid::Uuid::new_v4(),
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.0,
            recipe_preference: Some("nonexistent_recipe".into()),
        };
        let plan = router.route(&query).await.unwrap();
        assert_eq!(plan.recipe_name, "troubleshooting_similarity");
    }

    #[tokio::test]
    async fn test_router_uses_explicit_preference() {
        let router = DefaultRouter::new();
        let query = MemoryQuery {
            query_text: "anything".into(),
            tenant_id: uuid::Uuid::new_v4(),
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.0,
            recipe_preference: Some("firmware_specific".into()),
        };
        let plan = router.route(&query).await.unwrap();
        assert_eq!(plan.recipe_name, "firmware_specific");
    }

    #[tokio::test]
    async fn test_router_execution_plan_structure() {
        let router = DefaultRouter::new();
        let plan = router.route(&make_query("firmware version check")).await.unwrap();
        assert_eq!(plan.recipe_name, "firmware_specific");
        assert_eq!(plan.steps.len(), 1);
        assert!(plan.steps[0].parallel);
        assert_eq!(plan.steps[0].max_results_per_retriever, 10);
        assert_eq!(plan.verification_level, VerificationLevel::Basic);
    }
}
