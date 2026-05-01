# SIP Migration Studio

A hybrid plugin for the [SIP Service Intelligence Platform](https://github.com/jesseakc/sip) that imports customer data from external CRMs and CMMS systems.

## Overview

SIP Migration Studio is the **reference implementation** for SIP's Migration Core Framework. It demonstrates how a hybrid plugin can:

1. Accept CSV and JSON file uploads
2. Detect source schemas automatically
3. Map source fields to SIP's canonical import DTOs
4. Validate mapped records against business rules
5. Run dry-run simulations with zero side effects
6. Execute imports through SIP's core API layer (preserving RBAC, tenant isolation, audit logs)
7. Preserve external ID mappings for permanent traceability
8. Rollback imports when needed

## Quick Start

### Prerequisites

- SIP API running (see [main SIP README](../../README.md#quick-start))
- The `plugins` Cargo feature must be enabled (it is in the default `full` build)

### Enable the Plugin

Add the plugin directory:
```
plugins/sip-migration-studio/plugin.toml
```

The plugin registry automatically discovers `plugin.toml` files in the `plugins/` directory on startup. Restart the SIP API:

```bash
docker compose restart sip-api
```

Verify it's registered:
```bash
curl http://localhost:8000/api/v1/plugins/sip-migration-studio | jq .
```

### UI Access

The Migration Studio appears as a sidebar navigation item. Log in to the SIP web UI and click **Migration Studio** in the sidebar.

## Workflow

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ 1. Upload    │───▶│ 2. Map       │───▶│ 3. Validate  │
│    CSV/JSON  │    │    Fields    │    │    Records   │
└──────────────┘    └──────────────┘    └──────────────┘
                                                │
                    ┌──────────────┐    ┌──────────────┐
                    │ 6. Report    │◀───│ 5. Execute   │◀───│ 4. Dry Run  │
                    │    Export    │    │    Import     │    │    Preview  │
                    └──────────────┘    └──────────────┘    └──────────────┘
```

1. **Upload**: Drag & drop a CSV or JSON file. The plugin parses headers and shows a preview.
2. **Map Fields**: Map each source column to a canonical SIP field (name, serial_number, status, etc.).
3. **Validate**: Run validation to check for missing required fields, invalid enums, duplicate detection.
4. **Dry Run**: Simulate the import — no permanent changes. See what will be created, updated, or skipped.
5. **Execute Import**: Run the actual import. Records flow through SIP's core validation and are created via the standard service layer.
6. **Report**: View results — created records, skipped records, errors, external ID mappings.

## Architecture

```
plugins/sip-migration-studio/
├── plugin.toml              # Plugin manifest
├── README.md                # This file
├── docs/                    # Detailed documentation
│   ├── architecture.md
│   ├── migration-lifecycle.md
│   ├── mapping-template-reference.md
│   ├── transform-functions.md
│   ├── validation-rules.md
│   ├── connector-authoring-guide.md
│   ├── security-and-tenant-isolation.md
│   ├── testing-guide.md
│   └── troubleshooting.md
└── examples/                # Sample import files
    ├── generic-crm/
    │   └── contacts.csv
    ├── generic-cmms/
    │   └── assets.csv
    ├── salesforce-service-cloud/
    │   └── cases.json
    ├── servicenow-csm/
    │   └── incidents.json
    └── maximo-eam/
        └── assets.json
```

The frontend UI lives in `frontend/app/migration-studio/` (see [SIP plugin architecture docs](../../docs/plugins/overview.md)).

## API Endpoints

All migration endpoints are available under `{API_BASE}/migrations/`:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/jobs` | List all migration jobs |
| POST | `/jobs` | Create new migration job |
| GET | `/jobs/:id` | Get job details |
| POST | `/jobs/:id/source-records` | Upload source records |
| POST | `/jobs/:id/field-mappings` | Save field mappings |
| POST | `/jobs/:id/validate` | Trigger validation |
| POST | `/jobs/:id/dry-run` | Run dry run |
| POST | `/jobs/:id/execute` | Execute import |
| POST | `/jobs/:id/cancel` | Cancel job |
| POST | `/jobs/:id/rollback` | Rollback import |
| GET | `/jobs/:id/validation-issues` | List validation issues |
| GET | `/jobs/:id/duplicates` | List duplicate candidates |
| GET | `/jobs/:id/external-id-maps` | List external ID mappings |
| GET | `/jobs/:id/report` | Get import report |
| GET | `/jobs/:id/source-records` | List source records |
| GET | `/jobs/:id/staged-records` | List staged records |

See the [SIP Migration API docs](../../docs/migrations/api-reference.md) for full request/response schemas.

## Security & Permissions

The plugin requires these migration permissions (see manifest):
- `migration:create`, `migration:read`, `migration:map`, `migration:validate`
- `migration:dry_run`, `migration:execute`, `migration:rollback`, `migration:delete`

Default role assignments:
- **Admin**: All migration permissions
- **Manager**: Read, map, validate, dry run
- **Technician, Viewer, Vendor, Auditor**: None by default

## Building Your Own Connector

SIP Migration Studio is designed to be extended. See [connector-authoring-guide.md](docs/connector-authoring-guide.md) for how to add support for Salesforce, ServiceNow, Maximo, or custom APIs.

## License

AGPL-3.0-or-later — same as SIP core.
