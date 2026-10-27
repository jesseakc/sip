# Fleet maintenance intelligence and inventory forecasting

## Goal
Turn installed configuration, lifecycle status and spare stock into actionable fleet maintenance plans.

## Requirements
- Tenant-scoped queries for due/overdue components across customers, sites, systems, part numbers, serial ranges and 90/180/365-day windows.
- Demand forecasts derived from approved lifecycle policies, installation populations and stock-on-hand; distinguish confirmed obligations from statistical predictions.
- Suggest combining maintenance tasks with planned service visits, respecting site access, skill requirements, safety constraints and inventory availability.
- Generate reviewable proposed work orders and spare-part reservations; never silently schedule or order parts without approved policy.
- Natural-language agent tools for fleet questions with evidence-backed responses and transparent data freshness.
- Optional future adapters for InvenTree/Snipe-IT; keep external integration contracts decoupled from native Rust domain.
- Dashboards for due, overdue, uncertain, forecast demand and configuration/recall impact.

## Acceptance criteria
- Forecast replacement quantities across a date window from synthetic fleet data.
- Show sources, assumptions, missing telemetry and stock shortfalls.
- Prevent cross-tenant aggregation and double-counting of installed components.
- Generate proposed, non-committed work orders and auditable approvals.
- Benchmark larger fleet queries and verify graceful degradation with incomplete records.

## Dependencies
Inventory, engineering BOM, and lifecycle policy modules.

## Delivery
Implement after core modules with tests, UI and API docs.