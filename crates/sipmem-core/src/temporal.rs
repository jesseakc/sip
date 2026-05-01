use crate::types::*;
use chrono::{DateTime, Utc};

/// Resolves temporal queries — "what was true at time T?",
/// "what changed between T1 and T2?"
pub struct TemporalResolver;

impl TemporalResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn is_valid_at(&self, context: &TemporalContext, time: DateTime<Utc>) -> bool {
        if time < context.valid_from {
            return false;
        }
        if let Some(valid_to) = context.valid_to {
            if time > valid_to {
                return false;
            }
        }
        true
    }

    pub fn is_valid_during(&self, context: &TemporalContext, range: &TimeRange) -> bool {
        if let Some(ref end) = range.end {
            if context.valid_to.is_some_and(|vt| vt < range.start) {
                return false;
            }
            if context.valid_from > *end {
                return false;
            }
        } else {
            if context.valid_to.is_some_and(|vt| vt < range.start) {
                return false;
            }
        }
        true
    }

    pub fn is_stale(&self, context: &TemporalContext, now: DateTime<Utc>) -> bool {
        if let Some(superseded_at) = context.superseded_at {
            if now >= superseded_at {
                return true;
            }
        }
        if let Some(valid_to) = context.valid_to {
            if now > valid_to {
                return true;
            }
        }
        false
    }

    pub fn prune_stale(
        &self,
        entries: Vec<MemoryEvidence>,
        now: DateTime<Utc>,
    ) -> Vec<MemoryEvidence> {
        entries
            .into_iter()
            .filter(|e| {
                let ctx = TemporalContext {
                    observed_at: e.entry.observed_at,
                    recorded_at: e.entry.recorded_at,
                    valid_from: e.entry.valid_from,
                    valid_to: e.entry.valid_to,
                    superseded_at: None,
                    superseded_by: None,
                };
                !self.is_stale(&ctx, now)
            })
            .collect()
    }

    pub fn filter_by_range(
        &self,
        entries: Vec<MemoryEvidence>,
        range: &TimeRange,
    ) -> Vec<MemoryEvidence> {
        entries
            .into_iter()
            .filter(|e| {
                let ctx = TemporalContext {
                    observed_at: e.entry.observed_at,
                    recorded_at: e.entry.recorded_at,
                    valid_from: e.entry.valid_from,
                    valid_to: e.entry.valid_to,
                    superseded_at: None,
                    superseded_by: None,
                };
                self.is_valid_during(&ctx, range)
            })
            .collect()
    }

    pub fn find_changes(
        &self,
        entries: &[MemoryEntry],
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<MemoryEntry> {
        entries
            .iter()
            .filter(|e| e.recorded_at >= from && e.recorded_at <= to)
            .cloned()
            .collect()
    }
}

impl Default for TemporalResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_context(valid_from: DateTime<Utc>, valid_to: Option<DateTime<Utc>>) -> TemporalContext {
        TemporalContext {
            observed_at: valid_from,
            recorded_at: valid_from,
            valid_from,
            valid_to,
            superseded_at: None,
            superseded_by: None,
        }
    }

    #[test]
    fn test_is_valid_at_inside_range() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let ctx = make_context(now - Duration::hours(2), Some(now + Duration::hours(2)));
        assert!(resolver.is_valid_at(&ctx, now));
    }

    #[test]
    fn test_is_valid_at_before_range() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let ctx = make_context(now + Duration::hours(1), Some(now + Duration::hours(3)));
        assert!(!resolver.is_valid_at(&ctx, now));
    }

    #[test]
    fn test_is_valid_at_after_range() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let ctx = make_context(now - Duration::hours(3), Some(now - Duration::hours(1)));
        assert!(!resolver.is_valid_at(&ctx, now));
    }

    #[test]
    fn test_is_stale_superseded() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let mut ctx = make_context(now - Duration::hours(2), None);
        ctx.superseded_at = Some(now - Duration::hours(1));
        assert!(resolver.is_stale(&ctx, now));
    }

    #[test]
    fn test_is_stale_expired() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let ctx = make_context(now - Duration::hours(3), Some(now - Duration::hours(1)));
        assert!(resolver.is_stale(&ctx, now));
    }

    #[test]
    fn test_is_stale_not_stale() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let ctx = make_context(now - Duration::hours(1), Some(now + Duration::hours(1)));
        assert!(!resolver.is_stale(&ctx, now));
    }

    #[test]
    fn test_find_changes_filters_by_recorded_at() {
        let resolver = TemporalResolver::new();
        let now = Utc::now();
        let entry = MemoryEntry {
            id: MemoryId("e1".into()),
            category: MemoryCategory::Asset,
            tenant_id: uuid::Uuid::nil(),
            content: "test".into(),
            metadata: serde_json::json!({}),
            entity_references: vec![],
            observed_at: now - Duration::hours(2),
            recorded_at: now,
            valid_from: now,
            valid_to: None,
            confidence: 1.0,
            source: EvidenceSource {
                source_type: "test".into(),
                source_id: "s1".into(),
                retrieval_type: RetrievalType::SQL,
                reliability_score: 1.0,
            },
            verification_status: VerificationStatus::Verified,
        };
        let changes =
            resolver.find_changes(&[entry], now - Duration::hours(1), now + Duration::hours(1));
        assert_eq!(changes.len(), 1);
    }
}
