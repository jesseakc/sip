use chrono::{Datelike, Utc};
use sip_domain::{
    entity::schedule::ScheduleTriggerType,
    entity::work_order::{
        WorkOrder, WorkOrderPriority, WorkOrderSourceType, WorkOrderStatus, WorkOrderType,
    },
    error::SipError,
    id::{OrganizationId, UserId, WorkOrderId},
    repository::{ScheduleRepository, WorkOrderRepository},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::{PgScheduleRepository, PgWorkOrderRepository};
use sqlx::PgPool;

use crate::due_date::calculate_next_due;

pub struct CronEngine {
    pool: PgPool,
}

impl CronEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn tick(&self) -> Result<Vec<WorkOrder>, SipError> {
        let now = Utc::now();
        let schedule_repo = PgScheduleRepository::new(self.pool.clone());
        let wo_repo = PgWorkOrderRepository::new(self.pool.clone());

        let org_ids = sqlx::query_as::<_, (uuid::Uuid,)>(
            "SELECT id FROM organizations WHERE archived_at IS NULL",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        let mut created_work_orders = Vec::new();

        for (org_uuid,) in org_ids {
            let org_id = OrganizationId::from(org_uuid);
            let ctx = TenantContext::new(org_id, None);

            let schedules = match schedule_repo.list_active(&ctx).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "Cron tick: failed to list schedules for org {}: {}",
                        org_id,
                        e
                    );
                    continue;
                }
            };

            for schedule in schedules {
                if schedule.trigger_type != ScheduleTriggerType::Cron {
                    continue;
                }

                let Some(due) = schedule.next_due else {
                    continue;
                };

                if due > now {
                    continue;
                }

                let cron_expr = schedule
                    .trigger_config
                    .as_ref()
                    .and_then(|c| c.get("cron_expression"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if cron_expr.is_empty() {
                    continue;
                }

                let open_wo_exists = sqlx::query_as::<_, (uuid::Uuid,)>(
                    "SELECT id FROM work_orders WHERE schedule_id = $1 AND asset_id = $2 AND status NOT IN ('COMPLETED', 'CLOSED', 'CANCELLED') LIMIT 1"
                )
                .bind(uuid::Uuid::from(schedule.id))
                .bind(uuid::Uuid::from(schedule.asset_id))
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?
                .is_some();

                if open_wo_exists {
                    let next_due = calculate_next_due(cron_expr, due);
                    let patch = serde_json::json!({"next_due": next_due.map(|d| d.to_rfc3339())});
                    if let Err(e) = schedule_repo.update(&ctx, schedule.id, patch).await {
                        tracing::warn!(
                            "Cron tick: failed to update schedule {} next_due: {}",
                            schedule.id,
                            e
                        );
                    }
                    continue;
                }

                let template = schedule.work_order_template.as_ref();
                let title = template
                    .and_then(|t| t.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&schedule.name)
                    .to_string();
                let description = template
                    .and_then(|t| t.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let priority_str = template
                    .and_then(|t| t.get("priority"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("MEDIUM");
                let priority = match priority_str.to_uppercase().as_str() {
                    "LOW" => WorkOrderPriority::Low,
                    "HIGH" => WorkOrderPriority::High,
                    "CRITICAL" => WorkOrderPriority::Critical,
                    _ => WorkOrderPriority::Medium,
                };
                let wo_type_str = template
                    .and_then(|t| t.get("work_order_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("PREVENTIVE");
                let wo_type = match wo_type_str.to_uppercase().as_str() {
                    "CORRECTIVE" => WorkOrderType::Corrective,
                    "INSPECTION" => WorkOrderType::Inspection,
                    "EMERGENCY" => WorkOrderType::Emergency,
                    _ => WorkOrderType::Preventive,
                };

                let year = now.year();
                let max_disp = wo_repo.get_max_display_number_for_year(&ctx, year).await?;
                let next_num = match max_disp {
                    Some(num) => {
                        let suffix = num.split('-').next_back().unwrap_or("0");
                        suffix.parse::<i32>().unwrap_or(0) + 1
                    }
                    None => 1,
                };
                let display_number = format!("WO-{}-{:05}", year, next_num);

                let work_order = WorkOrder {
                    id: WorkOrderId::new(),
                    organization_id: org_id,
                    asset_id: schedule.asset_id,
                    parent_id: None,
                    schedule_id: Some(schedule.id),
                    work_order_type: wo_type,
                    priority,
                    status: WorkOrderStatus::Open,
                    title,
                    display_number,
                    description,
                    scheduled_start: None,
                    scheduled_end: None,
                    actual_start: None,
                    actual_end: None,
                    due_at: Some(now + chrono::Duration::days(7)),
                    estimated_hours: None,
                    actual_hours: None,
                    resolution_notes: None,
                    failure_code: None,
                    root_cause: None,
                    created_by_id: UserId::from(uuid::Uuid::nil()),
                    source_type: WorkOrderSourceType::Schedule,
                    source_system: Some("scheduler".to_string()),
                    external_id: None,
                    external_url: None,
                    reopened_count: 0,
                    last_reopened_at: None,
                    last_reopened_by_id: None,
                    version: 1,
                    archived_at: None,
                    archived_by_id: None,
                    archive_reason: None,
                    metadata: None,
                    created_at: now,
                    updated_at: now,
                };

                match wo_repo.create_work_order(&ctx, &work_order).await {
                    Ok(wo) => {
                        created_work_orders.push(wo);

                        let next_due = calculate_next_due(cron_expr, now);
                        let patch = serde_json::json!({
                            "next_due": next_due.map(|d| d.to_rfc3339()),
                            "last_triggered": now.to_rfc3339()
                        });
                        if let Err(e) = schedule_repo.update(&ctx, schedule.id, patch).await {
                            tracing::warn!(
                                "Cron tick: failed to update schedule {}: {}",
                                schedule.id,
                                e
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Cron tick: failed to create work order for schedule {}: {}",
                            schedule.id,
                            e
                        );
                    }
                }
            }
        }

        Ok(created_work_orders)
    }

    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.tick().await {
                    tracing::error!("Cron engine tick error: {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        })
    }
}
