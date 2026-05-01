# Connector Authoring Guide

SIP Migration Studio supports adding new import connectors. This guide walks through adding a new source system.

## Quick Start: CSV Connector

CSV is already supported. Use the sample files in `examples/` as templates.

## Adding a New Connector: Salesforce

### 1. Create Mapping Template

Create a mapping template file describing how Salesforce objects map to SIP entities:

**`mappings/salesforce-case-to-work-order.json`:**
```json
{
  "id": "salesforce-case-to-work-order",
  "name": "Salesforce Case → SIP Work Order",
  "source_system": "salesforce",
  "source_object_type": "case",
  "target_entity_type": "work_order",
  "field_mappings": [
    { "source": "CaseNumber", "target": "title" },
    { "source": "Description", "target": "description" },
    { "source": "Priority", "target": "priority", "transform": "normalize_priority" },
    { "source": "Status", "target": "status", "transform": "normalize_status" },
    { "source": "Subject", "target": "title" },
    { "source": "Asset.Name", "target": "asset_name" },
    { "source": "CreatedDate", "target": "due_date", "transform": "normalize_date" },
    { "source": "ClosedDate", "target": "completed_date", "transform": "normalize_date" }
  ]
}
```

### 2. Define Transform Functions

Each transform is a named function that the mapping engine applies:

```typescript
// plugin/transforms.ts

export function normalizePriority(value: string): string {
  const mapping: Record<string, string> = {
    'Critical': 'critical',
    'High': 'high',
    'Medium': 'medium',
    'Low': 'low',
  };
  return mapping[value] || value.toLowerCase();
}

export function normalizeStatus(value: string): string {
  const mapping: Record<string, string> = {
    'New': 'draft',
    'In Progress': 'open',
    'On Hold': 'on_hold',
    'Closed': 'completed',
    'Resolved': 'closed',
  };
  return mapping[value] || value.toLowerCase().replace(/ /g, '_');
}

export function normalizeDate(value: string): string {
  if (!value) return '';
  const d = new Date(value);
  return d.toISOString();
}
```

### 3. Register the Connector

Update `plugin.toml`:

```toml
[backend]
api_routes = ["/migration/connectors/salesforce"]

[[backend.jobs]]
id = "salesforce-export-poller"
name = "Salesforce Export Poller"
description = "Polls Salesforce for new Case exports and queues them for import."
schedule = "0 */6 * * *"
```

### 4. Add Sample Data

Add a representative sample file to `examples/salesforce-service-cloud/cases.json`.

### 5. Write Tests

Create tests for:
- Salesforce field mapping transforms
- Date normalization for Salesforce format (`2024-01-15T10:30:00.000Z`)
- Priority/status mapping for all Salesforce values
- Edge cases (null values, empty strings, multi-select picklists)

## Connector Checklist

Use this checklist when building a new connector:

- [ ] **Sample file** in `examples/{system}/` with realistic data
- [ ] **Mapping template** describing field mappings and transforms
- [ ] **Transform functions** for date normalization, enum mapping, string cleaning
- [ ] **Validation rules** for required fields specific to the source system
- [ ] **Duplicate detection** strategy for that system's ID format
- [ ] **Test data** covers edge cases (nulls, special characters, long strings)
- [ ] **Documentation** explaining the connector's assumptions and limitations
- [ ] **Manifest updated** with backend routes, jobs, and extension points
