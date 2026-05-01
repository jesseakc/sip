use serde::{Deserialize, Serialize};

use crate::id::{DocumentId, DocumentLinkId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentLink {
    pub id: DocumentLinkId,
    pub organization_id: OrganizationId,
    pub document_id: DocumentId,
    pub linked_entity_type: String,
    pub linked_entity_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
