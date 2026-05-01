use serde::{Deserialize, Serialize};

use crate::traits::VerificationLevel;
use crate::types::*;
use std::collections::HashMap;

/// A RetrievalRecipe defines exact behavior for a query pattern.
/// It specifies which retrievers, verification level, temporal constraints, and cost budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRecipe {
    pub name: String,
    pub description: String,
    pub retrievers: Vec<RetrievalType>,
    pub verification_level: VerificationLevel,
    pub temporal_constraint: Option<TimeRange>,
    pub cost_budget_ms: u64,
    pub min_confidence: f64,
}

impl RetrievalRecipe {
    pub fn exact_fact_lookup() -> Self {
        Self {
            name: "exact_fact_lookup".into(),
            description: "Direct SQL lookup for known facts".into(),
            retrievers: vec![RetrievalType::SQL],
            verification_level: VerificationLevel::Basic,
            temporal_constraint: None,
            cost_budget_ms: 200,
            min_confidence: 0.7,
        }
    }

    pub fn state_at_time() -> Self {
        Self {
            name: "state_at_time".into(),
            description: "What was true at a specific point in time".into(),
            retrievers: vec![RetrievalType::SQL, RetrievalType::Temporal],
            verification_level: VerificationLevel::Standard,
            temporal_constraint: None,
            cost_budget_ms: 500,
            min_confidence: 0.7,
        }
    }

    pub fn asset_history_summary() -> Self {
        Self {
            name: "asset_history_summary".into(),
            description: "Full historical view of an asset".into(),
            retrievers: vec![
                RetrievalType::SQL,
                RetrievalType::Temporal,
                RetrievalType::Graph,
            ],
            verification_level: VerificationLevel::Standard,
            temporal_constraint: None,
            cost_budget_ms: 1000,
            min_confidence: 0.6,
        }
    }

    pub fn troubleshooting_similarity() -> Self {
        Self {
            name: "troubleshooting_similarity".into(),
            description: "Find similar past issues for troubleshooting".into(),
            retrievers: vec![RetrievalType::Vector, RetrievalType::SQL],
            verification_level: VerificationLevel::Standard,
            temporal_constraint: None,
            cost_budget_ms: 800,
            min_confidence: 0.5,
        }
    }

    pub fn known_issue_investigation() -> Self {
        Self {
            name: "known_issue_investigation".into(),
            description: "Investigate known failure patterns".into(),
            retrievers: vec![RetrievalType::Vector, RetrievalType::Graph],
            verification_level: VerificationLevel::Strict,
            temporal_constraint: None,
            cost_budget_ms: 1500,
            min_confidence: 0.6,
        }
    }

    pub fn firmware_specific() -> Self {
        Self {
            name: "firmware_specific".into(),
            description: "Lookup firmware/software version facts".into(),
            retrievers: vec![RetrievalType::SQL],
            verification_level: VerificationLevel::Basic,
            temporal_constraint: None,
            cost_budget_ms: 200,
            min_confidence: 0.8,
        }
    }

    pub fn service_bulletin_check() -> Self {
        Self {
            name: "service_bulletin_check".into(),
            description: "Check OEM bulletins and recall notices".into(),
            retrievers: vec![RetrievalType::Vector, RetrievalType::SQL],
            verification_level: VerificationLevel::Standard,
            temporal_constraint: None,
            cost_budget_ms: 600,
            min_confidence: 0.7,
        }
    }

    pub fn contradiction_detection() -> Self {
        Self {
            name: "contradiction_detection".into(),
            description: "Exhaustive search for conflicting evidence".into(),
            retrievers: vec![
                RetrievalType::SQL,
                RetrievalType::Vector,
                RetrievalType::Graph,
                RetrievalType::Temporal,
            ],
            verification_level: VerificationLevel::Full,
            temporal_constraint: None,
            cost_budget_ms: 3000,
            min_confidence: 0.3,
        }
    }

    pub fn root_cause_candidate() -> Self {
        Self {
            name: "root_cause_candidate".into(),
            description: "Trace failure chains to potential root causes".into(),
            retrievers: vec![
                RetrievalType::Graph,
                RetrievalType::Temporal,
                RetrievalType::Vector,
            ],
            verification_level: VerificationLevel::Strict,
            temporal_constraint: None,
            cost_budget_ms: 2000,
            min_confidence: 0.5,
        }
    }

    pub fn stale_evidence_detection() -> Self {
        Self {
            name: "stale_evidence_detection".into(),
            description: "Find evidence that has been superseded or expired".into(),
            retrievers: vec![RetrievalType::Temporal, RetrievalType::SQL],
            verification_level: VerificationLevel::Standard,
            temporal_constraint: None,
            cost_budget_ms: 500,
            min_confidence: 0.5,
        }
    }
}

/// Recipe router: selects the best recipe for a query
pub struct RecipeRouter {
    recipes: HashMap<String, RetrievalRecipe>,
}

impl RecipeRouter {
    pub fn new() -> Self {
        let mut router = Self {
            recipes: HashMap::new(),
        };
        router.register(RetrievalRecipe::exact_fact_lookup());
        router.register(RetrievalRecipe::state_at_time());
        router.register(RetrievalRecipe::asset_history_summary());
        router.register(RetrievalRecipe::troubleshooting_similarity());
        router.register(RetrievalRecipe::known_issue_investigation());
        router.register(RetrievalRecipe::firmware_specific());
        router.register(RetrievalRecipe::service_bulletin_check());
        router.register(RetrievalRecipe::contradiction_detection());
        router.register(RetrievalRecipe::root_cause_candidate());
        router.register(RetrievalRecipe::stale_evidence_detection());
        router
    }

    pub fn register(&mut self, recipe: RetrievalRecipe) {
        self.recipes.insert(recipe.name.clone(), recipe);
    }

    pub fn select(&self, query: &MemoryQuery) -> Option<&RetrievalRecipe> {
        // If caller specified a preference, return that recipe
        if let Some(ref pref) = query.recipe_preference {
            return self.recipes.get(pref);
        }

        // Heuristic selection based on query content
        let lower = query.query_text.to_lowercase();

        if lower.contains("root cause") || lower.contains("why did") {
            return self.recipes.get("root_cause_candidate");
        }
        if lower.contains("contradiction") || lower.contains("conflict") {
            return self.recipes.get("contradiction_detection");
        }
        if lower.contains("firmware") || lower.contains("version") || lower.contains("software") {
            return self.recipes.get("firmware_specific");
        }
        if lower.contains("bulletin") || lower.contains("recall") || lower.contains("oem") {
            return self.recipes.get("service_bulletin_check");
        }
        if lower.contains("history") || lower.contains("lifetime") {
            return self.recipes.get("asset_history_summary");
        }
        if lower.contains("similar") || lower.contains("like") || lower.contains("troubleshoot") {
            return self.recipes.get("troubleshooting_similarity");
        }
        if lower.contains("known issue") || lower.contains("pattern") {
            return self.recipes.get("known_issue_investigation");
        }
        if lower.contains("stale") || lower.contains("superseded") || lower.contains("expired") {
            return self.recipes.get("stale_evidence_detection");
        }
        if lower.contains("when") || lower.contains("time") || lower.contains("date") {
            return self.recipes.get("state_at_time");
        }

        // Default: exact fact lookup
        self.recipes.get("exact_fact_lookup")
    }
}

impl Default for RecipeRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_query(text: &str, preference: Option<&str>) -> MemoryQuery {
        MemoryQuery {
            query_text: text.into(),
            tenant_id: Uuid::nil(),
            user_id: None,
            entity_filters: vec![],
            temporal_constraint: None,
            max_results: 10,
            min_confidence: 0.5,
            recipe_preference: preference.map(|s| s.into()),
        }
    }

    #[test]
    fn test_explicit_preference_used() {
        let router = RecipeRouter::new();
        let query = make_query("anything", Some("firmware_specific"));
        let recipe = router.select(&query).unwrap();
        assert_eq!(recipe.name, "firmware_specific");
    }

    #[test]
    fn test_heuristic_root_cause() {
        let router = RecipeRouter::new();
        let query = make_query("why did pump A fail root cause", None);
        let recipe = router.select(&query).unwrap();
        assert_eq!(recipe.name, "root_cause_candidate");
    }

    #[test]
    fn test_heuristic_firmware() {
        let router = RecipeRouter::new();
        let query = make_query("what firmware version is on asset X", None);
        let recipe = router.select(&query).unwrap();
        assert_eq!(recipe.name, "firmware_specific");
    }

    #[test]
    fn test_default_fallback() {
        let router = RecipeRouter::new();
        let query = make_query("pump status", None);
        let recipe = router.select(&query).unwrap();
        assert_eq!(recipe.name, "exact_fact_lookup");
    }

    #[test]
    fn test_heuristic_temporal() {
        let router = RecipeRouter::new();
        let query = make_query(
            "what was the oil pressure at 3pm when the alarm triggered",
            None,
        );
        let recipe = router.select(&query).unwrap();
        // "when" and "time" keywords should match state_at_time
        let matches = ["state_at_time", "root_cause_candidate"];
        assert!(matches.contains(&recipe.name.as_str()));
    }

    #[test]
    fn test_all_recipes_registered() {
        let router = RecipeRouter::new();
        assert_eq!(router.recipes.len(), 10);
    }
}
