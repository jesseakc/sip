# Product vision

[Documentation home](README.md)

SIP stands for Service Intelligence Platform. Its goal is to make equipment
history useful to the next person, application, or AI agent that needs it.

This page describes the product direction. For current behavior and unfinished
work, read the [architecture guide](architecture.md) and [progress tracker](PROGRESS.md).

## The problem SIP addresses

An asset can accumulate years of service knowledge across work orders,
technician notes, spreadsheets, manuals, and conversations. Finding what failed,
what changed, or what fixed it last time can require tracking down the person
who remembers the repair.

SIP aims to connect that history to the equipment itself, with structured
records that people and software can query.

## Who it is for

- Field service teams maintaining equipment across customer sites.
- Robotics companies supporting deployed machines and service fleets.
- Hospital biomedical and clinical engineering teams.
- Facilities, plant maintenance, and infrastructure teams.
- Fleet, vehicle, and heavy-equipment service organizations.
- Developers building service integrations, plugins, and AI workflows.
- Organizations moving service records from another maintenance system.

## How to think about SIP

| Idea | Plain-language meaning |
|------|------------------------|
| Maintenance management, or CMMS | Track equipment, work orders, inspections, parts, and preventive maintenance. |
| Asset CRM | Give each piece of equipment a connected service history, much as a customer record collects interactions. |
| AI-ready service data | Make records available through structured APIs, so AI tools can retrieve specific evidence. |
| Agent harness | Provide the permissions, workflow rules, and activity records around tools an agent can use. |

These ideas describe SIP's direction. They do not imply that every planned
integration, agent action, or verification step is available today.

## Design principles

**Start with equipment and its history.** Assets, locations, work orders, parts,
and service notes should connect through a common model. Configurable asset
types should support different kinds of equipment without a separate product
for each industry.

**Make the platform accessible through APIs.** The web interface, plugins, and
external clients should work through documented interfaces. The goal is to let
teams build new workflows on the same service records.

**Carry permissions into AI workflows.** Retrieval and tool use should respect
organization boundaries and user permissions. Proposed actions should follow
the same workflow rules as actions taken by people.

**Treat answers as claims that need evidence.** The SIPmem design combines exact
records, semantic search, relationships, and history. It aims to trace claims
to their sources, identify conflicts, and distinguish current facts from stale
information. This is a development goal, not a guarantee that generated answers
are correct. See the [SIPmem overview](sipmem/overview.md).

**Support self-hosting and portability.** SIP is licensed under
[AGPLv3](../LICENSE). Open code, documented APIs, and export tools are intended
to give operators control over their deployments and data.

## Questions SIP should help answer

- What work has been done on this asset?
- Have we seen this symptom on similar equipment?
- Which parts were replaced, and when?
- What changed before the failure started?
- Which records support this proposed explanation?

## Future direction

The longer-term plans include per-organization AI configuration, improved
retrieval and ranking, notifications, agent-assisted actions, predictive
maintenance, compliance support, mobile applications, and enterprise sign-in.
These are planning topics rather than release commitments. See the
[requirements](PRD-v2.md) and [progress tracker](PROGRESS.md) for more detail.

## Future website content

A separate website can carry the longer product story, industry examples,
comparison research, and educational articles. The existing
[website content strategy](PRD-website.md) provides that outline. Comparisons and
market figures should be checked against current sources before publication.
