# Migration API Reference

All endpoints require authentication and appropriate permissions (gated behind `#[cfg(feature = "plugins")]`).

## Base URL

```
/api/v1/migrations
```

## Endpoints

### Jobs

| Method | Endpoint | Permission | Description |
|--------|----------|-----------|-------------|
| `GET` | `/migrations` | `migration:read` | List all migration jobs |
| `POST` | `/migrations` | `migration:create` | Create a new migration job |
| `GET` | `/migrations/{job_id}` | `migration:read` | Get job details |
| `POST` | `/migrations/{job_id}/cancel` | `migration:delete` | Cancel a job |

### Source Records

| Method | Endpoint | Permission | Description |
|--------|----------|-----------|-------------|
| `GET` | `/migrations/{job_id}/source-records` | `migration:read` | List source records |
| `POST` | `/migrations/{job_id}/source-records` | `migration:create` | Upload source records |

### Field Mappings

| Method | Endpoint | Permission | Description |
|--------|----------|-----------|-------------|
| `GET` | `/migrations/{job_id}/mappings` | `migration:read` | Get field mappings |
| `POST` | `/migrations/{job_id}/mappings` | `migration:map` | Save field mappings |

### Pipeline

| Method | Endpoint | Permission | Description |
|--------|----------|-----------|-------------|
| `POST` | `/migrations/{job_id}/validate` | `migration:validate` | Validate staged records |
| `POST` | `/migrations/{job_id}/dry-run` | `migration:dry_run` | Dry-run import |
| `POST` | `/migrations/{job_id}/import` | `migration:execute` | Execute import |
| `POST` | `/migrations/{job_id}/rollback` | `migration:rollback` | Rollback import |

### Auditing

| Method | Endpoint | Permission | Description |
|--------|----------|-----------|-------------|
| `GET` | `/migrations/{job_id}/staged-records` | `migration:read` | List staged records |
| `GET` | `/migrations/{job_id}/validation-issues` | `migration:read` | List validation issues |
| `GET` | `/migrations/{job_id}/duplicates` | `migration:read` | List duplicate candidates |
| `GET` | `/migrations/{job_id}/external-id-maps` | `migration:read` | List external ID mappings |
| `GET` | `/migrations/{job_id}/report` | `migration:read` | Get migration report |

## Example Request: Create Job

```bash
curl -X POST http://localhost:8000/api/v1/migrations \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Maximo Asset Import",
    "source_system": "IBM Maximo",
    "source_object_type": "asset",
    "description": "Importing assets from legacy Maximo instance"
  }'
```

## Example Request: Add Source Records

```bash
curl -X POST http://localhost:8000/api/v1/migrations/{job_id}/source-records \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "records": [
      {
        "external_id": "MX-1001",
        "source_object_type": "asset",
        "name": "Chiller Unit 3",
        "serial_number": "SN-12345",
        "status": "OPERATIONAL"
      }
    ]
  }'
```
