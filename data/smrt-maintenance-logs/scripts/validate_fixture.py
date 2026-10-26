#!/usr/bin/env python3
"""Validate the generated fixture against SIP's own source of truth.

Rather than hard-coding the canonical import contract, this parses the real DTO
structs out of ``crates/sip-domain/src/entity/migration.rs`` so the check cannot
drift from the code. It then asserts, for every generated record:

* every key is a field of the corresponding canonical DTO
* required fields are present and non-empty
* enum-ish values are in the set the API's validate_job() accepts
* dates are ISO ``%Y-%m-%d``
* every cross-reference (asset_type, location, parent, asset, part) resolves

Plus one provenance check on the date inference: for source rows that carry both
an explicit dd.mm.yy date and a trailing 5-digit code, do the two agree?
"""
from __future__ import annotations

import json
import re
import sys
from datetime import date, timedelta
from pathlib import Path

import pandas as pd

ROOT = Path(__file__).resolve().parent.parent
REPO = ROOT.parent.parent
MIGRATION_RS = REPO / "crates/sip-domain/src/entity/migration.rs"
RAW = ROOT / "raw"
FIXTURE = ROOT / "fixture"

VALID_STATUS = {
    "operational", "degraded", "down", "maintenance", "retired",
    "draft", "open", "in_progress", "on_hold", "completed", "reviewed", "closed", "cancelled",
}
VALID_PRIORITY = {"critical", "high", "medium", "low", "routine"}
VALID_CRITICALITY = {"critical", "high", "medium", "low"}
VALID_WO_TYPE = {"preventive", "corrective", "inspection", "emergency"}

DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def dto_fields(struct_name: str) -> set[str]:
    """Field names of a `pub struct <name>Import { ... }` block."""
    src = MIGRATION_RS.read_text()
    match = re.search(rf"pub struct {struct_name} \{{(.*?)\n\}}", src, re.S)
    if not match:
        sys.exit(f"could not find struct {struct_name} in {MIGRATION_RS}")
    body = match.group(1)
    fields = set()
    for line in body.splitlines():
        line = line.strip()
        if line.startswith("pub ") and ":" in line:
            name = line[4:].split(":", 1)[0].strip()
            # ignore the serde rename attribute lines
            fields.add(name)
    return fields


DTOS = {
    "location": dto_fields("CanonicalLocationImport"),
    "asset_type": dto_fields("CanonicalAssetTypeImport"),
    "asset": dto_fields("CanonicalAssetImport"),
    "part": dto_fields("CanonicalPartImport"),
    "work_order": dto_fields("CanonicalWorkOrderImport"),
}

# `type` is renamed to `work_order_type` in Rust but serialises as "type".
ALIASES = {"work_order": {"type": "work_order_type"}}

errors: list[str] = []
warnings: list[str] = []
checks = 0


def check(cond: bool, msg: str, hard: bool = True) -> None:
    global checks
    checks += 1
    if not cond:
        (errors if hard else warnings).append(msg)


def main() -> None:
    print("canonical DTO fields discovered from Rust source:")
    for name, fields in DTOS.items():
        print(f"  {name:<12} ({len(fields):>2}) {sorted(fields)}")

    manifest = json.loads((FIXTURE / "manifest.json").read_text())
    loaded: dict[str, list[dict]] = {}
    for entry in manifest["files"]:
        if not entry["file"].startswith("canonical/"):
            continue
        payload = json.loads((FIXTURE / entry["file"]).read_text())
        object_type = payload["object_type"]
        records = payload["records"]
        loaded[object_type] = records
        check(len(records) == entry["count"],
              f"{entry['file']}: manifest says {entry['count']} records, file has {len(records)}")

        allowed = {f.lower() for f in DTOS[object_type]}
        allowed |= {alias for alias in ALIASES.get(object_type, {})}
        id_fields: set[str] = set()
        for key, value in payload.items():
            if key in ("object_type", "canonical_version", "source_system", "records"):
                continue
            errors.append(f"{entry['file']}: unexpected top-level key {key!r}")

        seen_ext: set[str] = set()
        for i, rec in enumerate(records):
            where = f"{entry['file']}[{i}]"
            for key in rec:
                check(key.lower() in allowed, f"{where}: unknown field {key!r} (not in canonical DTO)")
                if key.endswith("_external_id"):
                    id_fields.add(key)
            ext = rec.get("external_id")
            check(bool(ext), f"{where}: missing external_id")
            if ext:
                check(ext not in seen_ext, f"{where}: duplicate external_id {ext}")
                seen_ext.add(ext)

            if object_type in ("location", "asset", "asset_type", "part"):
                check(bool(rec.get("name")), f"{where}: name is required and empty")
            if object_type == "work_order":
                check(bool(rec.get("title")), f"{where}: title is required and empty")

            if object_type == "asset":
                check(rec.get("status") in VALID_STATUS,
                      f"{where}: status {rec.get('status')!r} not accepted by validate_job")
                check(rec.get("criticality") in VALID_CRITICALITY,
                      f"{where}: criticality {rec.get('criticality')!r} not accepted by validate_job")
            if object_type == "work_order":
                check(rec.get("status") in VALID_STATUS,
                      f"{where}: status {rec.get('status')!r} not accepted by validate_job")
                check(rec.get("priority") in VALID_PRIORITY,
                      f"{where}: priority {rec.get('priority')!r} not accepted by validate_job")
                check(rec.get("type") in VALID_WO_TYPE,
                      f"{where}: work order type {rec.get('type')!r} is not a WorkOrderType variant")
            if object_type == "part":
                check(bool(rec.get("part_number")), f"{where}: part_number is required by validate_job")

            for field in ("purchase_date", "warranty_expiry", "due_date", "completed_date"):
                value = rec.get(field)
                if value is not None:
                    check(isinstance(value, str) and bool(DATE_RE.match(value)),
                          f"{where}: {field}={value!r} is not ISO %Y-%m-%d")

    # ── cross-reference resolution ─────────────────────────────────────────
    asset_types = {r["external_id"] for r in loaded.get("asset_type", [])}
    locations = {r["external_id"] for r in loaded.get("location", [])}
    assets = {r["external_id"] for r in loaded.get("asset", [])}
    parts = {r["external_id"] for r in loaded.get("part", [])}

    for rec in loaded.get("asset", []):
        check(rec.get("asset_type_external_id") in asset_types,
              f"asset {rec['external_id']}: unknown asset_type {rec.get('asset_type_external_id')!r}")
        loc = rec.get("location_external_id")
        check(loc is None or loc in locations,
              f"asset {rec['external_id']}: unknown location {loc!r}")
    for rec in loaded.get("location", []):
        parent = rec.get("parent_external_id")
        check(parent is None or parent in locations,
              f"location {rec['external_id']}: unknown parent {parent!r}")

    wo_with_asset = 0
    for rec in loaded.get("work_order", []):
        asset = rec.get("asset_external_id")
        if asset:
            wo_with_asset += 1
            check(asset in assets, f"work_order {rec['external_id']}: unknown asset {asset!r}")
        for usage in rec.get("parts_used", []) or []:
            check(usage.get("part_external_id") in parts,
                  f"work_order {rec['external_id']}: unknown part {usage.get('part_external_id')!r}")

    print(f"\nwork orders resolving to an asset: {wo_with_asset} / {len(loaded.get('work_order', []))}")

    # ── date-inference provenance check ────────────────────────────────────
    df = pd.concat([pd.read_parquet(RAW / f"{s}.parquet") for s in ("train", "test")
                    if (RAW / f"{s}.parquet").exists()], ignore_index=True)
    explicit_re = re.compile(r"\b(\d{1,2})[./](\d{1,2})[./](\d{2,4})\b")
    serial_re = re.compile(r"\b(4[0-9]{4})\b")
    epoch = date(1899, 12, 30)
    agree = disagree = both = 0
    deltas: list[int] = []
    for text in df["text"].astype(str):
        m = explicit_re.search(text)
        s = serial_re.findall(text)
        if not (m and s):
            continue
        d, mo, y = (int(g) for g in m.groups())
        y += 2000 if y < 100 else 0
        try:
            explicit = date(y, mo, d)
        except ValueError:
            continue
        both += 1
        serial = epoch + timedelta(days=int(s[-1]))
        delta = abs((serial - explicit).days)
        deltas.append(delta)
        if delta <= 31:
            agree += 1
        else:
            disagree += 1
    print(f"\ntrailing 5-digit code treated as an Excel date serial:")
    print(f"  rows carrying both an explicit date and a 5-digit code: {both}")
    if both:
        print(f"  within 31 days of the explicit date: {agree} ({agree / both:.1%})")
        print(f"  median absolute difference: {sorted(deltas)[len(deltas) // 2]} days")
        if agree / both < 0.5:
            warnings.append(
                "Excel-serial date inference is weak: fewer than half of dual-dated rows agree. "
                "Consider dropping excel_serial as a date source."
            )

    print(f"\nchecks run: {checks}")
    print(f"errors  : {len(errors)}")
    print(f"warnings: {len(warnings)}")
    for e in errors[:25]:
        print("  ERROR  ", e)
    if len(errors) > 25:
        print(f"  ... and {len(errors) - 25} more errors")
    for w in warnings[:10]:
        print("  WARN   ", w)

    if errors:
        sys.exit(1)
    print("\nfixture validates against the canonical import contract.")


if __name__ == "__main__":
    main()
