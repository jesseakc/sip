# SIP

**Service Intelligence Platform**

SIP is open-source software for teams that maintain physical equipment. It brings
assets, work orders, parts, and service history together, with APIs and optional
AI tools to help people find and use that knowledge.

It is built for field service, robotics, facilities, fleet maintenance, and other
teams responsible for keeping equipment running.

## What you can do

- **Keep equipment records together.** Track assets, locations, parts, and repair history.
- **Manage maintenance work.** Create work orders, assign technicians, schedule recurring work, and record inspections.
- **Bring your existing data.** Import CSV and JSON records through Migration Studio.
- **Ask about service history.** Connect an AI provider to query operational records with source references.
- **Build on the platform.** Use the REST API and plugins for integrations, automation, and new interfaces.

## Run locally

You need Git, Docker, and Docker Compose v2.

```bash
git clone https://github.com/jesseakc/sip.git
cd sip
cp .env.example .env
docker compose up -d --build
```

Open [localhost:3000](http://localhost:3000). Fresh installations have no demo
accounts. Follow the [Quick Start](QUICKSTART.md#create-your-first-administrator)
to create your first administrator and sign in.

AI is disabled by default. The [Quick Start](QUICKSTART.md) covers setup,
importing data, and troubleshooting.

## Documentation

- [Documentation home](docs/README.md): guides and technical references.
- [Architecture](docs/architecture.md): how SIP works and current implementation gaps.
- [API reference](docs/openapi.yaml): endpoint definitions.
- [Plugin development](docs/plugins/overview.md): extend SIP and build integrations.
- [Product vision](docs/product-vision.md): the ideas behind SIP and future direction.
- [Development](docs/development.md): source layout, configuration, and checks.

## Project status

SIP is under active development. Some capabilities are still being connected or
expanded. See the [architecture guide](docs/architecture.md) for implementation
details and the [progress tracker](docs/PROGRESS.md) for development status.

## License

[GNU Affero General Public License v3.0](LICENSE).
