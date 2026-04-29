use serde::{Deserialize, Serialize};

use crate::id::{InspectionChecklistItemId, InspectionId, WorkOrderId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inspection {
    pub id: InspectionId,
    pub work_order_id: WorkOrderId,
    pub template_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecklistResponseType {
    PassFail,
    Numeric,
    Text,
    Photo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecklistResult {
    Pass,
    Fail,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionChecklistItem {
    pub id: InspectionChecklistItemId,
    pub inspection_id: InspectionId,
    pub ordinal: i32,
    pub question: String,
    pub response_type: ChecklistResponseType,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
    pub result: Option<ChecklistResult>,
    pub finding: Option<String>,
    pub photo_url: Option<String>,
}
