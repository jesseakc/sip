# SIP Quick Start

## Prerequisites
- Docker and Docker Compose v2+
- 8 GB RAM (16 GB recommended for local AI)

## 1. Clone and Configure
```bash
git clone https://github.com/jesseakc/sip.git
cd sip
cp .env.example .env
```

## 2. Start (Minimal Mode — no AI)

```bash
docker compose up --build
```

This starts: API (port 8000), Frontend (port 3000), PostgreSQL, Redis, MinIO.
Ollama is skipped by default. AI features are disabled.

## 3. Start (With Local AI)

```bash
docker compose --profile ollama up --build
```

Pulls llama3.1:8b automatically on first run. Takes ~2 minutes.

## 4. Start (With Hosted AI)

Edit `.env`:
```
SIP_AI_ENABLED=true
SIP_AI_PROVIDER=openai
SIP_AI_OPENAI_API_KEY=sk-your-key-here
```
Then: `docker compose up --build`

## 5. Login

| Role | Email | Password |
|------|-------|----------|
| Admin | admin@acme.local | password |
| Manager | manager@acme.local | password |
| Technician | tech1@acme.local | password |

Open http://localhost:3000

## 6. Services

| Service | URL |
|---------|-----|
| Frontend | http://localhost:3000 |
| API | http://localhost:8000 |
| API Health | http://localhost:8000/api/v1/health |
| MinIO Console | http://localhost:9001 |

## 7. Test Migration Studio

1. Login as admin
2. Click "Migration Studio" in the sidebar
3. Click "New Migration Job"
4. Name it "Test Import", select "CSV File", "Asset / Equipment"
5. Upload the sample file: `examples/migration/assets.csv`
6. Click Create
7. On the job page, go to the Mapping tab
8. Map source fields to SIP fields (at minimum: name → name, serial_number → serial_number)
9. Click Save & Validate
10. Go to Dry Run tab, click Run Dry Run
11. Go to Report tab, click Execute Import
12. Check Assets page — imported records should appear

## 8. Reset Test Data

```bash
docker compose down -v   # destroys all containers AND volumes
docker compose up --build  # fresh start with seed data
```

## Troubleshooting

### "Port already in use"
```bash
docker compose down
```

### "Ollama model not found"
The model pulls on first start. Wait for the ollama container to finish pulling before logging in.
Check: curl http://localhost:11434/api/tags

### "401 Unauthorized"
Token expired. Log out and log in again.

### "Database connection refused"
Wait for db-migrate to complete before sip-api starts. Check: `docker compose logs db-migrate`
