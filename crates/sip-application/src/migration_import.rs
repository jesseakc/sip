//! Canonical import handlers for the Migration Core Framework.
//!
//! `MigrationService::execute_import` used to understand only `asset`, mapping
//! every other canonical object type to a "no import handler" skip. That made a
//! migration job report success while quietly importing nothing but assets. This
//! module supplies real handlers for every object type the canonical contract
//! defines and that SIP can persist:
//!
//! `location`, `asset_type`, `manufacturer`, `asset_model`, `asset`, `part`,
//! `work_order` (with nested part usage).
//!
//! Two rules govern the handlers:
//!
//! 1. **Cross-references resolve through the migration external-ID map.** A work
//!    order finds its asset, an asset finds its location and type, by looking up
//!    the external ID recorded when the referenced object was imported. Objects
//!    therefore have to be imported in dependency order. The required order is
//!    `location -> asset_type -> manufacturer -> asset_model -> asset -> part ->
//!    work_order`, which is also the order the `data/` fixtures emit.
//!
//! 2. **Historical imports do not mutate live state.** The assets and work orders
//!    being imported are records of things that already happened. Part usage is
//!    therefore inserted as history rather than routed through
//!    `PartUsageRepository::create_part_usage`, which decrements
//!    `parts.quantity_on_hand` and rejects the write when stock is insufficient.
//!    Back-filling a 2020 repair must not drive today's stock negative.

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use sip_domain::{
    entity::{
        asset::{Asset, AssetStatus, Criticality},
        asset_model::{AssetModel, LifecycleStatus},
        asset_type::AssetType,
        location::{Location, LocationType},
        manufacturer::Manufacturer,
        migration::{
            CanonicalAssetImport, CanonicalAssetModelImport, CanonicalAssetTypeImport,
            CanonicalLocationImport, CanonicalManufacturerImport, CanonicalPartImport,
            CanonicalWorkOrderImport,
        },
        part::Part,
        work_order::{
            WorkOrder, WorkOrderPriority, WorkOrderSourceType, WorkOrderStatus, WorkOrderType,
        },
    },
    error::SipError,
    id::{
        AssetId, AssetModelId, AssetTypeId, LocationId, ManufacturerId, PartId, PartUsageId,
        UserId, WorkOrderId,
    },
    repository::{AssetRepository, PartRepository, WorkOrderRepository},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::{
    PgAssetModelRepository, PgAssetRepository, PgAssetTypeRepository, PgLocationRepository,
    PgManufacturerRepository, PgPartRepository, PgWorkOrderRepository,
};
use sip_tenancy::set_rls_org_pool;
use sqlx::PgPool;
use uuid::Uuid;

/// `source_system` recorded in the external-ID map for imported objects.
///
/// Kept identical to the value the original asset handler wrote, so maps created
/// by earlier versions of the importer still resolve.
pub const IMPORT_SOURCE_SYSTEM: &str = "import";

/// Result of importing one staged record.
pub enum ImportOutcome {
    /// An entity was created; carries its new SIP id.
    Created(Uuid),
    /// No handler exists for this object type. The caller records a skip.
    Unhandled,
}

/// Resolve a `*_external_id` reference to the SIP id of a previously imported object.
async fn resolve_ref(
    pool: &PgPool,
    ctx: &TenantContext,
    object_type: &str,
    external_id: &str,
) -> Result<Uuid, SipError> {
    let repo = sip_infrastructure::repositories::PgMigrationRepository::new(pool.clone());
    repo.lookup_external_id(ctx, IMPORT_SOURCE_SYSTEM, object_type, external_id)
        .await?
        .ok_or_else(|| {
            SipError::Validation(format!(
                "unresolved reference: no imported {object_type} with external id '{external_id}'. \
                 Import objects in dependency order ({})",
                "location -> asset_type -> manufacturer -> asset_model -> asset -> part -> work_order"
            ))
        })
}

/// Resolve an optional reference, treating empty strings as absent.
async fn resolve_opt_ref(
    pool: &PgPool,
    ctx: &TenantContext,
    object_type: &str,
    external_id: &Option<String>,
) -> Result<Option<Uuid>, SipError> {
    match external_id.as_deref() {
        None | Some("") => Ok(None),
        Some(id) => Ok(Some(resolve_ref(pool, ctx, object_type, id).await?)),
    }
}

// ── scalar parsing ──────────────────────────────────────────────────────────
//
// validate_job() only ever warns about unknown enum strings, so the handlers
// have to be defensive: anything unrecognised falls back rather than failing an
// otherwise importable record.

fn parse_status(v: Option<&str>) -> AssetStatus {
    match v.unwrap_or("").to_ascii_lowercase().as_str() {
        "operational" => AssetStatus::Operational,
        "degraded" => AssetStatus::Degraded,
        "down" => AssetStatus::Down,
        "maintenance" => AssetStatus::Maintenance,
        "retired" => AssetStatus::Retired,
        _ => AssetStatus::Operational,
    }
}

fn parse_criticality(v: Option<&str>) -> Criticality {
    match v.unwrap_or("").to_ascii_lowercase().as_str() {
        "critical" => Criticality::Critical,
        "high" => Criticality::High,
        "low" => Criticality::Low,
        _ => Criticality::Medium,
    }
}

fn parse_priority(v: Option<&str>) -> WorkOrderPriority {
    match v.unwrap_or("").to_ascii_lowercase().as_str() {
        "critical" => WorkOrderPriority::Critical,
        "high" => WorkOrderPriority::High,
        "low" | "routine" => WorkOrderPriority::Low,
        _ => WorkOrderPriority::Medium,
    }
}

fn parse_wo_status(v: Option<&str>) -> WorkOrderStatus {
    match v.unwrap_or("").to_ascii_lowercase().as_str() {
        "draft" => WorkOrderStatus::Draft,
        "open" => WorkOrderStatus::Open,
        "assigned" => WorkOrderStatus::Assigned,
        "accepted" => WorkOrderStatus::Accepted,
        "in_progress" => WorkOrderStatus::InProgress,
        "on_hold" => WorkOrderStatus::OnHold,
        "reviewed" => WorkOrderStatus::Reviewed,
        "closed" => WorkOrderStatus::Closed,
        "cancelled" | "canceled" => WorkOrderStatus::Cancelled,
        _ => WorkOrderStatus::Completed,
    }
}

fn parse_wo_type(v: Option<&str>) -> WorkOrderType {
    match v.unwrap_or("").to_ascii_lowercase().as_str() {
        "preventive" => WorkOrderType::Preventive,
        "inspection" => WorkOrderType::Inspection,
        "emergency" => WorkOrderType::Emergency,
        _ => WorkOrderType::Corrective,
    }
}

fn parse_location_type(v: Option<&str>) -> LocationType {
    match v.unwrap_or("").to_ascii_lowercase().as_str() {
        "site" => LocationType::Site,
        "building" => LocationType::Building,
        "floor" => LocationType::Floor,
        "room" => LocationType::Room,
        "area" => LocationType::Area,
        _ => LocationType::Site,
    }
}

/// Accept either RFC 3339 or a bare `%Y-%m-%d` date, which is what validate_job()
/// accepts. Anything else is dropped rather than failing the record.
fn parse_date(v: Option<&str>) -> Option<NaiveDate> {
    let raw = v?.trim();
    if raw.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").ok().or_else(|| {
        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|dt| dt.date_naive())
    })
}

/// Same as [`parse_date`] but as a UTC timestamp, for columns typed TIMESTAMPTZ.
fn parse_dt(v: Option<&str>) -> Option<DateTime<Utc>> {
    let raw = v?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|ndt| Utc.from_utc_datetime(&ndt))
}

fn dec(v: Option<f64>) -> Decimal {
    Decimal::from_f64_retain(v.unwrap_or(0.0)).unwrap_or(Decimal::ZERO)
}

/// Allocates `WO-<year>-<n>` display numbers for one import run.
///
/// The runtime path derives each number from `MAX(display_number)` with a retry
/// loop on collision. Doing that per record for a 10k-row import would mean 10k
/// extra queries and a large number of collisions, so the ceiling is read once
/// and numbers are handed out sequentially.
pub struct DisplayNumberAllocator {
    year: i32,
    next: i64,
}

impl DisplayNumberAllocator {
    pub async fn new(pool: &PgPool, ctx: &TenantContext) -> Result<Self, SipError> {
        let repo = PgWorkOrderRepository::new(pool.clone());
        let year = chrono::Datelike::year(&Utc::now());
        let max = repo.get_max_display_number_for_year(ctx, year).await?;
        let next = max
            .and_then(|n| n.rsplit('-').next().map(str::to_string))
            .and_then(|suffix| suffix.parse::<i64>().ok())
            .unwrap_or(0)
            + 1;
        Ok(Self { year, next })
    }

    fn take(&mut self) -> String {
        let n = self.next;
        self.next += 1;
        format!("WO-{}-{:05}", self.year, n)
    }
}

// ── handlers ────────────────────────────────────────────────────────────────

async fn import_location(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalLocationImport,
) -> Result<Uuid, SipError> {
    let parent = resolve_opt_ref(pool, ctx, "location", &dto.parent_external_id).await?;
    let now = Utc::now();
    let location = Location {
        id: LocationId::new(),
        organization_id: ctx.organization_id,
        parent_id: parent.map(LocationId::from),
        name: dto.name.clone(),
        location_type: parse_location_type(dto.location_type.as_deref()),
        geo_latitude: dto.latitude,
        geo_longitude: dto.longitude,
        metadata: Some(serde_json::json!({
            "imported": true,
            "source": IMPORT_SOURCE_SYSTEM,
            "address_line1": dto.address_line1,
            "address_line2": dto.address_line2,
            "city": dto.city,
            "state": dto.state,
            "postal_code": dto.postal_code,
            "country": dto.country,
            "timezone": dto.timezone,
            "notes": dto.notes,
            "active": dto.active,
        })),
        created_at: now,
        updated_at: now,
    };
    let created = PgLocationRepository::new(pool.clone())
        .create(ctx, &location)
        .await?;
    Ok(Uuid::from(created.id))
}

async fn import_asset_type(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalAssetTypeImport,
) -> Result<Uuid, SipError> {
    let now = Utc::now();
    let asset_type = AssetType {
        id: AssetTypeId::new(),
        organization_id: Some(ctx.organization_id),
        name: dto.name.clone(),
        category: dto
            .category
            .clone()
            .unwrap_or_else(|| "general".to_string()),
        description: dto.description.clone(),
        schema: dto.json_schema.clone(),
        default_pm_schedules: None,
        default_inspection_template: None,
        icon: dto.icon.clone(),
        is_system: false,
        created_at: now,
        updated_at: now,
    };
    let created = PgAssetTypeRepository::new(pool.clone())
        .create(ctx, &asset_type)
        .await?;
    Ok(Uuid::from(created.id))
}

async fn import_manufacturer(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalManufacturerImport,
) -> Result<Uuid, SipError> {
    let now = Utc::now();
    let manufacturer = Manufacturer {
        id: ManufacturerId::new(),
        organization_id: Some(ctx.organization_id),
        name: dto.name.clone(),
        website: dto.website.clone(),
        support_url: dto
            .support_email
            .clone()
            .or_else(|| dto.support_phone.clone()),
        created_at: now,
        updated_at: now,
    };
    let created = PgManufacturerRepository::new(pool.clone())
        .create(ctx, &manufacturer)
        .await?;
    Ok(Uuid::from(created.id))
}

async fn import_asset_model(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalAssetModelImport,
) -> Result<Uuid, SipError> {
    let manufacturer_id = resolve_opt_ref(pool, ctx, "manufacturer", &dto.manufacturer_external_id)
        .await?
        .ok_or_else(|| {
            SipError::Validation(format!(
                "asset model '{}' has no manufacturer_external_id; asset models require one",
                dto.name
            ))
        })?;
    let asset_type_id = resolve_opt_ref(pool, ctx, "asset_type", &dto.asset_type_external_id)
        .await?
        .ok_or_else(|| {
            SipError::Validation(format!(
                "asset model '{}' has no asset_type_external_id; asset models require one",
                dto.name
            ))
        })?;
    let now = Utc::now();
    let model = AssetModel {
        id: AssetModelId::new(),
        organization_id: Some(ctx.organization_id),
        manufacturer_id: ManufacturerId::from(manufacturer_id),
        name: dto.name.clone(),
        model_number: dto.name.clone(),
        revision: None,
        lifecycle_status: LifecycleStatus::Active,
        asset_type_id: AssetTypeId::from(asset_type_id),
        documentation_url: None,
        default_attributes: dto.specifications.clone(),
        metadata: None,
        created_at: now,
        updated_at: now,
    };
    let created = PgAssetModelRepository::new(pool.clone())
        .create(ctx, &model)
        .await?;
    Ok(Uuid::from(created.id))
}

async fn import_asset(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalAssetImport,
) -> Result<Uuid, SipError> {
    // The previous handler left location, type and model empty and randomised the
    // asset type. Resolve them for real, and fail loudly when a declared
    // reference cannot be resolved rather than silently dropping the link.
    let location_id = resolve_opt_ref(pool, ctx, "location", &dto.location_external_id).await?;
    let model_id = resolve_opt_ref(pool, ctx, "asset_model", &dto.model_external_id).await?;
    let parent_id = resolve_opt_ref(pool, ctx, "asset", &dto.parent_external_id).await?;

    let asset_type_id =
        match resolve_opt_ref(pool, ctx, "asset_type", &dto.asset_type_external_id).await? {
            Some(id) => id,
            None => {
                // A single-type import is a common shape; fall back to a
                // default type instead of failing, matching prior behaviour.
                tracing::warn!(
                "asset '{}' has no resolvable asset_type_external_id; using a generated type id",
                dto.name
            );
                Uuid::new_v4()
            }
        };

    let now = Utc::now();
    let asset = Asset {
        id: AssetId::new(),
        organization_id: ctx.organization_id,
        location_id: location_id.map(LocationId::from),
        parent_id: parent_id.map(AssetId::from),
        asset_type_id: AssetTypeId::from(asset_type_id),
        model_id: model_id.map(AssetModelId::from),
        name: dto.name.clone(),
        description: dto.notes.clone(),
        serial_number: dto.serial_number.clone(),
        firmware_version: None,
        software_version: None,
        hardware_revision: None,
        status: parse_status(dto.status.as_deref()),
        version: 1,
        criticality: parse_criticality(dto.criticality.as_deref()),
        installed_date: parse_date(dto.purchase_date.as_deref()),
        warranty_expiry: parse_date(dto.warranty_expiry.as_deref()),
        attributes: dto.custom_attributes.clone(),
        tags: dto.tags.clone().unwrap_or_default(),
        metadata: None,
        created_at: now,
        updated_at: now,
    };
    let created = PgAssetRepository::new(pool.clone())
        .create_asset(ctx, &asset)
        .await?;
    Ok(Uuid::from(created.id))
}

async fn import_part(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalPartImport,
) -> Result<Uuid, SipError> {
    let manufacturer_id =
        resolve_opt_ref(pool, ctx, "manufacturer", &dto.manufacturer_external_id).await?;
    let mut description = dto.description.clone().unwrap_or_default();
    if let Some(category) = dto.category.as_deref() {
        if !category.is_empty() {
            description = format!("{description} (category: {category})")
                .trim()
                .to_string();
        }
    }
    if let Some(mfr) = manufacturer_id {
        description = format!("{description} (manufacturer: {mfr})")
            .trim()
            .to_string();
    }
    if let Some(notes) = dto.notes.as_deref() {
        description = format!("{description} {notes}").trim().to_string();
    }
    let now = Utc::now();
    let part = Part {
        id: PartId::new(),
        organization_id: ctx.organization_id,
        name: dto.name.clone(),
        part_number: dto.part_number.clone(),
        description: if description.is_empty() {
            None
        } else {
            Some(description)
        },
        quantity_on_hand: dec(dto.quantity_on_hand),
        quantity_minimum: dto.reorder_point.map(|v| dec(Some(v))),
        unit: dto
            .unit_of_measure
            .clone()
            .unwrap_or_else(|| "each".to_string()),
        unit_cost: dto.unit_cost.map(|v| dec(Some(v))),
        storage_location: dto.location.clone(),
        created_at: now,
        updated_at: now,
    };
    let created = PgPartRepository::new(pool.clone())
        .create(ctx, &part)
        .await?;
    Ok(Uuid::from(created.id))
}

async fn import_work_order(
    pool: &PgPool,
    ctx: &TenantContext,
    dto: &CanonicalWorkOrderImport,
    display_numbers: &mut DisplayNumberAllocator,
) -> Result<Uuid, SipError> {
    let created_by: UserId = ctx.user_id.ok_or(SipError::PermissionDenied)?;

    let asset_external_id = dto.asset_external_id.as_deref().unwrap_or("");
    if asset_external_id.is_empty() {
        return Err(SipError::Validation(format!(
            "work order '{}' has no asset_external_id; work orders require an asset",
            dto.title
        )));
    }
    let asset_id = resolve_ref(pool, ctx, "asset", asset_external_id).await?;

    let completed = parse_dt(dto.completed_date.as_deref());
    let now = Utc::now();
    let work_order = WorkOrder {
        id: WorkOrderId::new(),
        organization_id: ctx.organization_id,
        asset_id: AssetId::from(asset_id),
        parent_id: None,
        schedule_id: None,
        work_order_type: parse_wo_type(dto.work_order_type.as_deref()),
        priority: parse_priority(dto.priority.as_deref()),
        status: parse_wo_status(dto.status.as_deref()),
        title: dto.title.clone(),
        display_number: display_numbers.take(),
        description: dto.description.clone().unwrap_or_default(),
        scheduled_start: None,
        scheduled_end: None,
        actual_start: None,
        actual_end: completed,
        due_at: parse_dt(dto.due_date.as_deref()),
        estimated_hours: dto.labor_hours.map(|h| dec(Some(h))),
        actual_hours: dto.labor_hours.map(|h| dec(Some(h))),
        resolution_notes: dto.resolution_notes.clone(),
        failure_code: None,
        root_cause: None,
        created_by_id: created_by,
        source_type: WorkOrderSourceType::Api,
        source_system: Some(IMPORT_SOURCE_SYSTEM.to_string()),
        external_id: dto.external_id.clone(),
        external_url: None,
        reopened_count: 0,
        last_reopened_at: None,
        last_reopened_by_id: None,
        version: 1,
        archived_at: None,
        archived_by_id: None,
        archive_reason: None,
        metadata: Some(serde_json::json!({
            "imported": true,
            "source_external_id": dto.source_external_id,
        })),
        created_at: now,
        updated_at: now,
    };
    let created = PgWorkOrderRepository::new(pool.clone())
        .create_work_order(ctx, &work_order)
        .await?;
    let work_order_id = Uuid::from(created.id);

    // Historical part usage: recorded as history, deliberately NOT routed through
    // create_part_usage, which decrements live stock and rejects insufficient
    // quantities. See the module docs.
    if let Some(usages) = dto.parts_used.as_ref() {
        for usage in usages {
            let Some(part_external_id) = usage.part_external_id.as_deref() else {
                continue;
            };
            if part_external_id.is_empty() {
                continue;
            }
            let part_id = match resolve_ref(pool, ctx, "part", part_external_id).await {
                Ok(id) => id,
                Err(e) => {
                    // A missing part should not lose the work order itself.
                    tracing::warn!("work order {}: skipping part usage, {}", work_order_id, e);
                    continue;
                }
            };
            set_rls_org_pool(pool, ctx.organization_id)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
            sqlx::query(
                "INSERT INTO part_usage (id, work_order_id, part_id, quantity, used_by_id)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(Uuid::from(PartUsageId::new()))
            .bind(work_order_id)
            .bind(part_id)
            .bind(usage.quantity)
            .bind(Uuid::from(created_by))
            .execute(pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
    }

    Ok(work_order_id)
}

/// Import a single staged record, dispatching on its target entity type.
pub async fn import_staged_record(
    pool: &PgPool,
    ctx: &TenantContext,
    object_type: &str,
    canonical_data: &serde_json::Value,
    display_numbers: &mut DisplayNumberAllocator,
) -> Result<ImportOutcome, SipError> {
    macro_rules! parse_dto {
        ($ty:ty) => {
            match serde_json::from_value::<$ty>(canonical_data.clone()) {
                Ok(dto) => dto,
                Err(e) => {
                    return Err(SipError::Validation(format!("invalid DTO: {}", e)));
                }
            }
        };
    }

    let id = match object_type {
        "location" => import_location(pool, ctx, &parse_dto!(CanonicalLocationImport)).await?,
        "asset_type" => import_asset_type(pool, ctx, &parse_dto!(CanonicalAssetTypeImport)).await?,
        "manufacturer" => {
            import_manufacturer(pool, ctx, &parse_dto!(CanonicalManufacturerImport)).await?
        }
        "asset_model" => {
            import_asset_model(pool, ctx, &parse_dto!(CanonicalAssetModelImport)).await?
        }
        "asset" => import_asset(pool, ctx, &parse_dto!(CanonicalAssetImport)).await?,
        "part" => import_part(pool, ctx, &parse_dto!(CanonicalPartImport)).await?,
        "work_order" => {
            import_work_order(
                pool,
                ctx,
                &parse_dto!(CanonicalWorkOrderImport),
                display_numbers,
            )
            .await?
        }
        _ => return Ok(ImportOutcome::Unhandled),
    };
    Ok(ImportOutcome::Created(id))
}

/// Undo one imported entity by archiving it where the domain supports archival.
///
/// Locations, asset types, manufacturers, models and parts have no archive or
/// delete path in SIP today, so rollback cannot remove them. Rather than pretend
/// otherwise, this returns a message the caller records as a rollback skip.
pub async fn rollback_entity(
    pool: &PgPool,
    ctx: &TenantContext,
    entity_type: &str,
    entity_id: Uuid,
) -> Result<Option<String>, SipError> {
    match entity_type {
        "asset" => {
            PgAssetRepository::new(pool.clone())
                .archive_asset(ctx, AssetId::from(entity_id))
                .await?;
            Ok(None)
        }
        "work_order" => {
            let by = ctx.user_id.ok_or(SipError::PermissionDenied)?;
            PgWorkOrderRepository::new(pool.clone())
                .archive_work_order(ctx, WorkOrderId::from(entity_id), by)
                .await?;
            Ok(None)
        }
        other => Ok(Some(format!(
            "{other} has no archive or delete operation in SIP; rolled back by leaving it in place"
        ))),
    }
}
