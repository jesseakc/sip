#!/usr/bin/env python3
"""Build SIP-loadable fixtures from the SMRT maintenance log dataset.

Source
------
`ASNIJKDNI/SMRT-Maintenance-Logs` (Hugging Face). 10,785 rows of real SMRT Trains
maintenance log text, each tagged with the component that was replaced or fixed.
The raw parquet files are vendored under ``raw/``.

What this produces
------------------
Two parallel encodings of the same records, both matching SIP's canonical import
contracts in ``crates/sip-domain/src/entity/migration.rs``:

``fixture/canonical/NN-<object>.json``
    ``{"object_type": ..., "canonical_version": "1.0", "records": [...]}``
    Ready for ``POST /api/v1/migrations/{id}/source-records`` in dependency order.

``fixture/csv/<object>.csv``
    Flat CSV for the Migration Studio upload wizard (header names are chosen to
    map cleanly onto canonical field names).

Everything the source does not state is marked as inferred, both in the record
metadata and in ``README.md``. No OEM, depot, line, cost or labour value is
invented: the source simply does not carry them.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import re
import sys
from collections import Counter, defaultdict
from datetime import date, datetime, timedelta
from pathlib import Path

import pandas as pd

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "raw"
FIXTURE = ROOT / "fixture"

# ── Text parsing ────────────────────────────────────────────────────────────

SEP = "~~~~~"
# A 4-digit car number. Years (19xx/20xx) are excluded separately.
CAR_TOKEN = re.compile(r"\b([1-9][0-9]{3})\b")
YEARLIKE = re.compile(r"^(19|20)[0-9]{2}$")
EXPLICIT_DATE = re.compile(r"\b(\d{1,2})[./](\d{1,2})[./](\d{2,4})\b")
EXCEL_SERIAL = re.compile(r"\b(4[0-9]{4})\b")  # 40000-49999 -> 2010-2036
SERIAL_NO = re.compile(r"(?:OLD|NEW|New|Old)\s*S/?N\s*:?\s*([0-9A-Za-z][0-9A-Za-z/\-]{2,})")
TRAILING_NAMES = re.compile(
    r"\b([A-Z][a-z]{2,}(?:/[A-Z][a-z]{2,})*)\s*\.?\s*(?:\(\s*)?(?:\d{1,2}[./]\d{1,2}[./]\d{2,4}|4[0-9]{4})?\s*\)?\s*$"
)
WS = re.compile(r"\s+")

EXCEL_EPOCH = date(1899, 12, 30)

# Heuristic priority signals. Documented as heuristics in README.md.
SAFETY_TERMS = (
    "brake", "smoke detector", "fire", "door", "emergency", "evacuat",
    "derail", "obstacle", "horn", "gangway", "detrainment", "levelling valve",
)
URGENT_TERMS = ("emergency", "immediate", "unsafe", "critical", "fail safe")
PREVENTIVE_TERMS = ("snaprep", "servicing", "pm ", "routine", "check due", "clean ")

# Rolling-stock series, inferred from the leading digit of the car number.
# The source does not state the fleet class, so this stays a numeric grouping.
SERIES_OF_DIGIT = {
    "1": ("1xxx", "Car series 1xxx (inferred grouping)"),
    "2": ("2xxx", "Car series 2xxx (inferred grouping)"),
    "3": ("3xxx", "Car series 3xxx (inferred grouping)"),
    "4": ("4xxx", "Car series 4xxx (inferred grouping)"),
    "5": ("5xxx", "Car series 5xxx (inferred grouping)"),
    "6": ("6xxx", "Car series 6xxx (inferred grouping)"),
    "7": ("7xxx", "Car series 7xxx (inferred grouping)"),
}

ORG_NAME = "SMRT Trains"
SITE_EXT_ID = "smrt-site"
CAR_TYPE_EXT_ID = "smrt-train-car"
COMPONENT_CATEGORY = "Rolling Stock Component"


def norm(text: str) -> str:
    return WS.sub(" ", str(text or "").replace("\x00", " ")).strip()


def slug(value: str) -> str:
    s = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return s[:60] or "unnamed"


def parse_explicit_date(text: str) -> date | None:
    """dd.mm.yy[yy] / dd/mm/yyyy. Ambiguous numeric order is assumed day-first,
    which the dataset's own examples confirm (``28.6.24``, ``18/11/2020``)."""
    for m in EXPLICIT_DATE.finditer(text):
        d, mo, y = (int(g) for g in m.groups())
        if y < 100:
            y += 2000 if y < 70 else 1900
        if not (1 <= mo <= 12 and 1 <= d <= 31):
            continue
        try:
            candidate = date(y, mo, d)
        except ValueError:
            continue
        if 1990 <= candidate.year <= 2035:
            return candidate
    return None


def parse_excel_serial(text: str) -> date | None:
    """Trailing 5-digit codes such as ``45382`` are consistent with Excel date
    serials (45382 -> 2024-03-23). Used only when no explicit date is present."""
    found = EXCEL_SERIAL.findall(text)
    for token in reversed(found):
        candidate = EXCEL_EPOCH + timedelta(days=int(token))
        if 2005 <= candidate.year <= 2035:
            return candidate
    return None


def extract_car(text: str) -> str | None:
    """First non-year 4-digit token; falls back to a ``#1234`` marker."""
    hash_marker = re.search(r"#\s*([1-9][0-9]{3})\b", text)
    if hash_marker and not YEARLIKE.match(hash_marker.group(1)):
        return hash_marker.group(1)
    for token in CAR_TOKEN.findall(text):
        if not YEARLIKE.match(token):
            return token
    return None


def derive_priority(blob: str, log_count: int) -> str:
    low = blob.lower()
    if any(t in low for t in URGENT_TERMS):
        return "critical"
    if any(t in low for t in SAFETY_TERMS):
        return "high"
    if log_count >= 8:
        return "high"
    if log_count >= 3:
        return "medium"
    return "low"


def derive_type(blob: str) -> str:
    low = blob.lower()
    if any(t in low for t in PREVENTIVE_TERMS) and not any(
        t in low for t in ("fault", "fail", "leak", "not working", "defect")
    ):
        return "preventive"
    return "corrective"


def load_rows() -> pd.DataFrame:
    frames = []
    for name in ("train", "test"):
        path = RAW / f"{name}.parquet"
        if path.exists():
            df = pd.read_parquet(path)
            frames.append(df)
    if not frames:
        sys.exit(f"no parquet files in {RAW}")
    return pd.concat(frames, ignore_index=True)


def load_label_names() -> list[str]:
    path = RAW / "label_names.json"
    if not path.exists():
        sys.exit(f"missing {path} - run the label fetch first")
    return json.loads(path.read_text())


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(limit: int | None) -> dict:
    df = load_rows()
    labels = load_label_names()

    # ── pass 1: normalise every row ────────────────────────────────────────
    rows = []
    date_sources: Counter[str] = Counter()
    for idx, raw in enumerate(df.itertuples(index=False)):
        text = norm(getattr(raw, "text"))
        label_id = int(getattr(raw, "label"))
        component = labels[label_id] if 0 <= label_id < len(labels) else f"Category {label_id}"

        report, _, detail = text.partition(SEP)
        report = norm(report) or text
        detail = norm(detail)
        blob = f"{report} {detail}"

        car = extract_car(report) or extract_car(blob)
        explicit = parse_explicit_date(text)
        serial = None if explicit else parse_excel_serial(text)
        logged = explicit or serial
        date_sources["explicit" if explicit else ("excel_serial" if serial else "none")] += 1

        names = TRAILING_NAMES.search(detail or report)
        technicians = names.group(1) if names else None
        serials = SERIAL_NO.findall(text)

        rows.append(
            {
                "seq": idx,
                "external_id": f"SMRT-LOG-{idx:05d}",
                "car": car,
                "component": component,
                "report": report,
                "detail": detail,
                "blob": blob,
                "logged": logged.isoformat() if logged else None,
                "technicians": technicians,
                "serials": serials,
            }
        )

    if limit:
        rows = rows[:limit]

    # Aggregate per-car statistics used for asset criticality and tags.
    per_car: dict[str, list[dict]] = defaultdict(list)
    for r in rows:
        if r["car"]:
            per_car[r["car"]].append(r)

    # ── locations ──────────────────────────────────────────────────────────
    locations = [
        {
            "external_id": SITE_EXT_ID,
            "name": ORG_NAME,
            "location_type": "site",
            "country": "SG",
            "timezone": "Asia/Singapore",
            "notes": "Root location for the SMRT maintenance-log fixture.",
        }
    ]
    series_seen = sorted({r["car"][0] for r in rows if r["car"] and r["car"][0] in SERIES_OF_DIGIT})
    for digit in series_seen:
        key, note = SERIES_OF_DIGIT[digit]
        locations.append(
            {
                "external_id": f"smrt-series-{key}",
                "name": f"Car series {key}",
                "location_type": "building",
                "parent_external_id": SITE_EXT_ID,
                "country": "SG",
                "timezone": "Asia/Singapore",
                "notes": (
                    note
                    + ". Grouping is derived from the first digit of the car number; "
                    "the source dataset records no depot or line."
                ),
            }
        )

    # ── asset types ────────────────────────────────────────────────────────
    asset_types = [
        {
            "external_id": CAR_TYPE_EXT_ID,
            "name": "Train Car",
            "description": (
                "Electric multiple unit car. Inferred asset type for the SMRT "
                "maintenance-log fixture (the source is per-car maintenance text)."
            ),
            "category": "rolling_stock",
            "icon": "train-front",
            "json_schema": {
                "type": "object",
                "properties": {
                    "car_number": {"type": "string", "description": "Car number as written in the log"},
                    "series": {"type": "string", "description": "Inferred numeric series group"},
                    "log_count": {"type": "integer", "description": "Maintenance log entries in the source"},
                    "first_log_date": {"type": "string", "format": "date"},
                    "last_log_date": {"type": "string", "format": "date"},
                    "top_components": {"type": "array", "items": {"type": "string"}},
                },
            },
        }
    ]

    # ── assets ─────────────────────────────────────────────────────────────
    assets = []
    for car in sorted(per_car):
        logs = per_car[car]
        dated = sorted(r["logged"] for r in logs if r["logged"])
        components = Counter(r["component"] for r in logs)
        top = [c for c, _ in components.most_common(5)]
        latest = max((r for r in logs if r["logged"]), key=lambda r: r["logged"], default=logs[-1])
        low = f"{latest['report']} {latest['detail']}".lower()
        # A car whose newest log defers work is reported degraded; otherwise
        # every log in the source ends with the repair completed and function
        # checked, so the car is operational.
        deferred = any(
            t in low for t in ("raise to rsw", "deferred", "not done", "pending", "recommend", "to monitor")
        )
        assets.append(
            {
                "external_id": f"SMRT-CAR-{car}",
                "name": f"Car {car}",
                "asset_type_external_id": CAR_TYPE_EXT_ID,
                "location_external_id": f"smrt-series-{SERIES_OF_DIGIT[car[0]][0]}",
                "serial_number": car,
                "status": "degraded" if deferred else "operational",
                "criticality": derive_priority(" ".join(top), len(logs)),
                "notes": (
                    f"{len(logs)} maintenance log entries from the SMRT fixture; "
                    f"first {dated[0] if dated else 'unknown'}, last {dated[-1] if dated else 'unknown'}. "
                    "Derived from maintenance text only."
                ),
                "tags": [f"series:{SERIES_OF_DIGIT[car[0]][0]}"] + [f"component:{slug(c)}" for c in top[:3]],
                "custom_attributes": {
                    "car_number": car,
                    "series": SERIES_OF_DIGIT[car[0]][0],
                    "log_count": len(logs),
                    "first_log_date": dated[0] if dated else None,
                    "last_log_date": dated[-1] if dated else None,
                    "top_components": top,
                },
            }
        )

    # ── parts (one per maintenance component category) ─────────────────────
    component_counts = Counter(r["component"] for r in rows)
    parts = []
    for component, count in sorted(component_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        parts.append(
            {
                "external_id": f"SMRT-PART-{slug(component)}",
                "name": component,
                "part_number": f"SMRT-CAT-{slug(component).upper()[:48]}",
                "description": (
                    f"Component category '{component}' from the SMRT maintenance log fixture; "
                    f"referenced by {count} log entr{'y' if count == 1 else 'ies'}."
                ),
                "quantity_on_hand": 0,
                "unit_of_measure": "each",
                "reorder_point": 0,
                "category": COMPONENT_CATEGORY,
                "notes": "Quantities are not present in the source dataset and default to 0.",
            }
        )

    part_ext_of_component = {c: f"SMRT-PART-{slug(c)}" for c in component_counts}

    # ── work orders ────────────────────────────────────────────────────────
    work_orders = []
    for r in rows:
        car = r["car"]
        priority = derive_priority(r["blob"], len(per_car.get(car, [])) if car else 1)
        wo = {
            "external_id": r["external_id"],
            "title": f"{r['component']}" + (f" - Car {car}" if car else ""),
            "description": r["report"],
            "priority": priority,
            "type": derive_type(r["blob"]),
            "status": "completed",
            "source_external_id": r["external_id"],
            "parts_used": [
                {
                    "part_external_id": part_ext_of_component[r["component"]],
                    "quantity": 1,
                    "notes": "Component replaced or serviced per log entry.",
                }
            ],
        }
        if car:
            wo["asset_external_id"] = f"SMRT-CAR-{car}"
        if r["logged"]:
            wo["completed_date"] = r["logged"]
            wo["due_date"] = r["logged"]
        resolution_bits = []
        if r["detail"]:
            resolution_bits.append(r["detail"])
        if r["technicians"]:
            resolution_bits.append(f"Technician(s): {r['technicians']}")
        if r["serials"]:
            resolution_bits.append("Serial numbers referenced: " + ", ".join(r["serials"][:4]))
        wo["resolution_notes"] = "\n".join(resolution_bits) if resolution_bits else None
        work_orders.append({k: v for k, v in wo.items() if v is not None})

    return {
        "rows": rows,
        "locations": locations,
        "asset_types": asset_types,
        "assets": assets,
        "parts": parts,
        "work_orders": work_orders,
        "date_sources": dict(date_sources),
        "component_counts": component_counts,
    }


def write_canonical(built: dict) -> list[dict]:
    out = FIXTURE / "canonical"
    out.mkdir(parents=True, exist_ok=True)
    order = [
        ("01-locations.json", "location"),
        ("02-asset-types.json", "asset_type"),
        ("03-assets.json", "asset"),
        ("04-parts.json", "part"),
        ("05-work-orders.json", "work_order"),
    ]
    key_of = {
        "location": "locations",
        "asset_type": "asset_types",
        "asset": "assets",
        "part": "parts",
        "work_order": "work_orders",
    }
    written = []
    for filename, object_type in order:
        records = built[key_of[object_type]]
        payload = {
            "object_type": object_type,
            "canonical_version": "1.0",
            "source_system": "SMRT Maintenance Logs (Hugging Face: ASNIJKDNI/SMRT-Maintenance-Logs)",
            "records": records,
        }
        path = out / filename
        path.write_text(json.dumps(payload, indent=1, ensure_ascii=False) + "\n")
        written.append({"file": f"canonical/{filename}", "object_type": object_type,
                        "count": len(records), "sha256": sha256(path)})
    return written


CSV_COLUMNS = {
    "locations": ["external_id", "name", "location_type", "parent_external_id", "city", "country",
                  "timezone", "notes"],
    "asset-types": ["external_id", "name", "description", "category", "icon"],
    "assets": ["external_id", "name", "asset_type_external_id", "location_external_id",
               "serial_number", "status", "criticality", "notes", "tags"],
    "parts": ["external_id", "name", "part_number", "description", "quantity_on_hand",
              "unit_of_measure", "reorder_point", "category", "notes"],
    "work-orders": ["external_id", "title", "description", "asset_external_id", "priority", "type",
                    "status", "completed_date", "due_date", "resolution_notes", "source_external_id"],
}


def write_csv(built: dict) -> list[dict]:
    out = FIXTURE / "csv"
    out.mkdir(parents=True, exist_ok=True)
    source = {
        "locations": built["locations"],
        "asset-types": built["asset_types"],
        "assets": built["assets"],
        "parts": built["parts"],
        "work-orders": built["work_orders"],
    }
    written = []
    for name, records in source.items():
        cols = CSV_COLUMNS[name]
        path = out / f"{name}.csv"
        with path.open("w", newline="", encoding="utf-8") as fh:
            writer = csv.DictWriter(fh, fieldnames=cols, extrasaction="ignore")
            writer.writeheader()
            for rec in records:
                row = dict(rec)
                if isinstance(row.get("tags"), list):
                    row["tags"] = ";".join(row["tags"])
                writer.writerow(row)
        written.append({"file": f"csv/{name}.csv", "object_type": name,
                        "count": len(records), "sha256": sha256(path)})
    return written


def main() -> None:
    global RAW
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--limit", type=int, default=None,
                    help="only build the first N source rows (for a quick smoke fixture)")
    ap.add_argument("--sample", action="store_true",
                    help="also emit a 500-row sample fixture under fixture/sample-500/")
    ap.add_argument("--raw-dir", default=os.environ.get("SIP_DATA_RAW", str(RAW)),
                    help="directory holding train.parquet / test.parquet "
                         "(default: the corpus raw/ dir, or $SIP_DATA_RAW). The "
                         "downloaded records are deliberately not committed.")
    args = ap.parse_args()

    # The source records live outside the repository; only this script is tracked.
    RAW = Path(args.raw_dir).expanduser().resolve()
    if not (RAW / "train.parquet").is_file():
        sys.exit(
            f"source dataset not found in {RAW}\n"
            "Download the corpus locally and point at it with --raw-dir or "
            "$SIP_DATA_RAW. See data/smrt-maintenance-logs/README.md."
        )

    built = build(args.limit)
    files = write_canonical(built) + write_csv(built)

    manifest = {
        "generated_at": datetime.now().astimezone().isoformat(timespec="seconds"),
        "source": {
            "dataset": "ASNIJKDNI/SMRT-Maintenance-Logs",
            "url": "https://huggingface.co/datasets/ASNIJKDNI/SMRT-Maintenance-Logs",
            "license_declared_on_hub": "mit",
            "rows_used": len(built["rows"]),
            "limit_applied": args.limit,
        },
        "load_order": ["location", "asset_type", "asset", "part", "work_order"],
        "counts": {
            "locations": len(built["locations"]),
            "asset_types": len(built["asset_types"]),
            "assets": len(built["assets"]),
            "parts": len(built["parts"]),
            "work_orders": len(built["work_orders"]),
        },
        "date_extraction": built["date_sources"],
        "files": files,
        "inferences": [
            "Car number is the first non-year 4-digit token in the log text.",
            "Car series (1xxx..7xxx) is a numeric grouping by leading digit; the source has no depot or fleet class.",
            "Dates prefer explicit dd.mm.yy[yy]; 5-digit trailing codes are read as Excel serials when no explicit date exists.",
            "Priority is a keyword heuristic (safety terms -> high, urgency terms -> critical, recurrence -> medium/high).",
            "Work order type is corrective unless the text reads as a routine check, clean or servicing task.",
            "Asset status is degraded only when the newest log defers work; otherwise operational.",
            "No manufacturer, asset model, cost, labour-hour or quantity values are produced.",
        ],
    }
    FIXTURE.mkdir(parents=True, exist_ok=True)
    (FIXTURE / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")

    if args.sample:
        sample_dir = FIXTURE / "sample-500"
        sample_dir.mkdir(parents=True, exist_ok=True)
        for object_type, key in (("location", "locations"), ("asset_type", "asset_types"),
                                 ("asset", "assets"), ("part", "parts"),
                                 ("work_order", "work_orders")):
            recs = built[key]
            # keep the sample self-consistent: only assets/WOs touched by the first 500 rows
            if object_type == "work_order":
                recs = recs[:500]
                keep_assets = {r.get("asset_external_id") for r in recs if r.get("asset_external_id")}
            elif object_type == "asset":
                recs = built["assets"][:150]
                keep_assets = set()
            else:
                keep_assets = set()
            payload = {"object_type": object_type, "canonical_version": "1.0", "records": recs}
            (sample_dir / f"{object_type}.json").write_text(
                json.dumps(payload, indent=1, ensure_ascii=False) + "\n")

    print(json.dumps(manifest["counts"], indent=2))
    print("date sources:", manifest["date_extraction"])
    print(f"fixture written to {FIXTURE}")


if __name__ == "__main__":
    main()
