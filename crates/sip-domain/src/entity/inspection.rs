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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checklist_response_type_serde_roundtrip() {
        for ty in [ChecklistResponseType::PassFail, ChecklistResponseType::Numeric, ChecklistResponseType::Text, ChecklistResponseType::Photo] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: ChecklistResponseType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }

    #[test]
    fn test_checklist_result_serde_roundtrip() {
        for result in [ChecklistResult::Pass, ChecklistResult::Fail, ChecklistResult::NotApplicable] {
            let json = serde_json::to_string(&result).unwrap();
            let back: ChecklistResult = serde_json::from_str(&json).unwrap();
            assert_eq!(back, result);
        }
    }
}
