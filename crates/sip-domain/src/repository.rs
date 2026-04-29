use async_trait::async_trait;

use crate::entity::activity::Activity;
use crate::entity::ai_conversation::{AIConversation, AIMessage, AIRetrievalTrace};
use crate::entity::asset::Asset;
use crate::entity::document::Document;
use crate::entity::embedding_record::EmbeddingRecord;
use crate::entity::inspection::{Inspection, InspectionChecklistItem};
use crate::entity::part::PartUsage;
use crate::entity::part::Part;
use crate::entity::schedule::Schedule;
use crate::entity::work_order::{WorkOrder, WorkOrderAssignment, WorkOrderStatusHistory};
use crate::error::SipError;
use crate::id::{AIConversationId, AIMessageId, AssetId, DocumentId, InspectionId, PartId, ScheduleId, UserId, WorkOrderId, WorkOrderAssignmentId};
use crate::tenant::TenantContext;

#[async_trait]
pub trait AssetRepository {
    async fn create_asset(
        &self,
        ctx: &TenantContext,
        asset: &Asset,
    ) -> Result<Asset, SipError>;

    async fn get_asset(
        &self,
        ctx: &TenantContext,
        id: AssetId,
    ) -> Result<Option<Asset>, SipError>;

    async fn update_asset(
        &self,
        ctx: &TenantContext,
        id: AssetId,
        expected_version: i32,
        patch: serde_json::Value,
    ) -> Result<Asset, SipError>;

    async fn list_assets(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<Asset>, SipError>;

    async fn archive_asset(
        &self,
        ctx: &TenantContext,
        id: AssetId,
    ) -> Result<Asset, SipError>;

    async fn list_children(
        &self,
        ctx: &TenantContext,
        parent_id: AssetId,
    ) -> Result<Vec<Asset>, SipError>;
}

#[async_trait]
pub trait WorkOrderRepository {
    async fn create_work_order(
        &self,
        ctx: &TenantContext,
        work_order: &WorkOrder,
    ) -> Result<WorkOrder, SipError>;

    async fn get_work_order(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
    ) -> Result<Option<WorkOrder>, SipError>;

    async fn update_work_order(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        expected_version: i32,
        patch: serde_json::Value,
    ) -> Result<WorkOrder, SipError>;

    async fn list_work_orders(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<WorkOrder>, SipError>;

    async fn archive_work_order(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        archived_by_id: crate::id::UserId,
    ) -> Result<WorkOrder, SipError>;

    async fn list_work_orders_by_asset(
        &self,
        ctx: &TenantContext,
        asset_id: AssetId,
    ) -> Result<Vec<WorkOrder>, SipError>;

    async fn get_max_display_number_for_year(
        &self,
        ctx: &TenantContext,
        year: i32,
    ) -> Result<Option<String>, SipError>;
}

#[async_trait]
pub trait WorkOrderAssignmentRepository {
    async fn create_assignment(
        &self,
        ctx: &TenantContext,
        assignment: &WorkOrderAssignment,
    ) -> Result<WorkOrderAssignment, SipError>;

    async fn list_assignments_by_work_order(
        &self,
        ctx: &TenantContext,
        work_order_id: WorkOrderId,
    ) -> Result<Vec<WorkOrderAssignment>, SipError>;

    async fn delete_assignment(
        &self,
        ctx: &TenantContext,
        id: WorkOrderAssignmentId,
    ) -> Result<(), SipError>;

    async fn update_assignment(
        &self,
        ctx: &TenantContext,
        id: WorkOrderAssignmentId,
        status: crate::entity::work_order::AssignmentStatus,
    ) -> Result<crate::entity::work_order::WorkOrderAssignment, SipError>;
}

#[async_trait]
pub trait PartUsageRepository {
    async fn create_part_usage(
        &self,
        ctx: &TenantContext,
        usage: &PartUsage,
    ) -> Result<PartUsage, SipError>;

    async fn list_part_usage_by_work_order(
        &self,
        ctx: &TenantContext,
        work_order_id: WorkOrderId,
    ) -> Result<Vec<PartUsage>, SipError>;
}

#[async_trait]
pub trait WorkOrderStatusHistoryRepository {
    async fn create_status_history(
        &self,
        ctx: &TenantContext,
        record: &WorkOrderStatusHistory,
    ) -> Result<WorkOrderStatusHistory, SipError>;
}

#[async_trait]
pub trait DocumentRepository {
    async fn create_document(
        &self,
        ctx: &TenantContext,
        document: &Document,
    ) -> Result<Document, SipError>;

    async fn get_document(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
    ) -> Result<Option<Document>, SipError>;

    async fn update_document_processing_status(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
        status: crate::entity::document::ProcessingStatus,
        error: Option<String>,
    ) -> Result<Document, SipError>;

    async fn list_documents(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<Document>, SipError>;

    async fn archive(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
        archived_by_id: UserId,
        reason: &str,
    ) -> Result<Document, SipError>;
}

#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn create_activity(
        &self,
        ctx: &TenantContext,
        activity: &Activity,
    ) -> Result<Activity, SipError>;

    async fn list_activities_by_entity(
        &self,
        ctx: &TenantContext,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<Vec<Activity>, SipError>;

    async fn list_activities(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<Activity>, SipError>;
}

#[async_trait]
pub trait AIConversationRepository: Send + Sync {
    async fn create_conversation(
        &self,
        ctx: &TenantContext,
        conversation: &AIConversation,
    ) -> Result<AIConversation, SipError>;

    async fn create_message(
        &self,
        ctx: &TenantContext,
        message: &AIMessage,
    ) -> Result<AIMessage, SipError>;

    async fn create_retrieval_trace(
        &self,
        ctx: &TenantContext,
        trace: &AIRetrievalTrace,
    ) -> Result<AIRetrievalTrace, SipError>;

    async fn list_conversations_by_user(
        &self,
        ctx: &TenantContext,
        user_id: UserId,
    ) -> Result<Vec<AIConversation>, SipError>;

    async fn get_messages_by_conversation(
        &self,
        ctx: &TenantContext,
        conversation_id: AIConversationId,
    ) -> Result<Vec<AIMessage>, SipError>;

    async fn list_traces_by_message(
        &self,
        ctx: &TenantContext,
        message_id: AIMessageId,
    ) -> Result<Vec<AIRetrievalTrace>, SipError>;

    async fn submit_feedback(
        &self,
        ctx: &TenantContext,
        message_id: AIMessageId,
        rating: i32,
        comment: Option<String>,
    ) -> Result<(), SipError>;
}

#[async_trait]
pub trait InspectionRepository {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: InspectionId,
    ) -> Result<Option<Inspection>, SipError>;

    async fn list(
        &self,
        ctx: &TenantContext,
        work_order_id: Option<WorkOrderId>,
    ) -> Result<Vec<Inspection>, SipError>;

    async fn update_items(
        &self,
        ctx: &TenantContext,
        inspection_id: InspectionId,
        items: Vec<InspectionChecklistItem>,
    ) -> Result<Inspection, SipError>;
}

#[async_trait]
pub trait PartRepository {
    async fn create(
        &self,
        ctx: &TenantContext,
        part: &Part,
    ) -> Result<Part, SipError>;

    async fn get(
        &self,
        ctx: &TenantContext,
        id: PartId,
    ) -> Result<Option<Part>, SipError>;

    async fn list(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<Part>, SipError>;

    async fn update(
        &self,
        ctx: &TenantContext,
        id: PartId,
        patch: serde_json::Value,
    ) -> Result<Part, SipError>;
}

#[async_trait]
pub trait EmbeddingRecordRepository: Send + Sync {
    async fn insert_embedding_record(
        &self,
        ctx: &TenantContext,
        record: &EmbeddingRecord,
    ) -> Result<EmbeddingRecord, SipError>;

    async fn delete_embeddings_by_source(
        &self,
        ctx: &TenantContext,
        source_type: crate::entity::embedding_record::EmbeddingSourceType,
        source_id: uuid::Uuid,
    ) -> Result<(), SipError>;
}

#[async_trait]
pub trait ScheduleRepository {
    async fn create(&self, ctx: &TenantContext, schedule: &Schedule) -> Result<Schedule, SipError>;
    async fn get(&self, ctx: &TenantContext, id: ScheduleId) -> Result<Option<Schedule>, SipError>;
    async fn list(&self, ctx: &TenantContext) -> Result<Vec<Schedule>, SipError>;
    async fn update(&self, ctx: &TenantContext, id: ScheduleId, patch: serde_json::Value) -> Result<Schedule, SipError>;
    async fn archive(&self, ctx: &TenantContext, id: ScheduleId, archived_by: UserId) -> Result<Schedule, SipError>;
    async fn list_active(&self, ctx: &TenantContext) -> Result<Vec<Schedule>, SipError>;
}
