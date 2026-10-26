#!/usr/bin/env python3
"""Exploratory analysis of the SMRT maintenance log dataset.

Answers the questions needed to design the SIP fixture mapping:
  - how many rows, how the text is structured
  - can we reliably extract car numbers, dates, part serials, technicians
  - what the label classes look like and how they are distributed
"""
from __future__ import annotations

import collections
import re
import sys
from pathlib import Path

import pandas as pd

RAW = Path(__file__).resolve().parent.parent / "raw"


def load() -> pd.DataFrame:
    frames = []
    for name in ("train", "test"):
        p = RAW / f"{name}.parquet"
        if p.exists():
            df = pd.read_parquet(p)
            df["_split"] = name
            frames.append(df)
    if not frames:
        sys.exit(f"no parquet files under {RAW}")
    return pd.concat(frames, ignore_index=True)


CAR_RE = re.compile(r"\b([1-9][0-9]{3})\b")           # 4-digit car numbers
SERIAL_RE = re.compile(r"(?:OLD|NEW)?\s*S/?N\s*:?\s*([0-9A-Za-z][0-9A-Za-z/\-]{3,})", re.I)
DATE_DOT_RE = re.compile(r"\b(\d{1,2})[./](\d{1,2})[./](\d{2,4})\b")
UNIT_RE = re.compile(r"#\s*([1-9][0-9]{3})\b")


def main() -> None:
    df = load()
    print(f"rows total      : {len(df)}")
    print(f"columns         : {list(df.columns)}")
    print(f"label dtype     : {df['label'].dtype}")
    print(f"null texts      : {df['text'].isna().sum()}")

    lens = df["text"].astype(str).str.len()
    print(f"text length     : min={lens.min()} p50={int(lens.median())} p95={int(lens.quantile(.95))} max={lens.max()} mean={lens.mean():.0f}")

    texts = df["text"].astype(str)

    # ---- structure of the text -------------------------------------------
    has_tilde = texts.str.contains("~~~~~", regex=False)
    print(f"\ncontains '~~~~~' separator: {has_tilde.sum()} / {len(texts)} ({has_tilde.mean():.1%})")

    # ---- car numbers ------------------------------------------------------
    cars: collections.Counter[str] = collections.Counter()
    no_car = 0
    for t in texts:
        found = CAR_RE.findall(t)
        if not found:
            no_car += 1
        cars.update(found)
    print(f"\ndistinct 4-digit tokens      : {len(cars)}")
    print(f"rows with no 4-digit token   : {no_car} ({no_car / len(texts):.1%})")
    print(f"top 15 tokens                : {cars.most_common(15)}")
    print(f"token count distribution     : {collections.Counter(cars.values()).most_common(6)}")

    # first token is likely the reporting car
    first_tokens = [CAR_RE.search(t).group(1) if CAR_RE.search(t) else None for t in texts]
    ft = collections.Counter(x for x in first_tokens if x)
    print(f"distinct leading cars        : {len(ft)}")
    print(f"leading-car top 15           : {ft.most_common(15)}")
    # which prefixes (rolling stock class is encoded in the first digit)
    print(f"leading-car first digit dist : {collections.Counter(c[0] for c in ft).most_common()}")

    # ---- labels -----------------------------------------------------------
    print(f"\ndistinct labels: {df['label'].nunique()}")
    try:
        names = df["label"].astype("category").cat.categories
        print(f"first 12 label names: {list(names[:12])}")
    except Exception:
        pass
    print("label value counts (top 15):")
    print(df["label"].value_counts().head(15).to_string())

    # ---- dates ------------------------------------------------------------
    dot_dates = sum(1 for t in texts if DATE_DOT_RE.search(t))
    print(f"\nrows with dd.mm.yy-ish date  : {dot_dates} ({dot_dates / len(texts):.1%})")

    # ---- part serials -----------------------------------------------------
    sn_rows = sum(1 for t in texts if SERIAL_RE.search(t))
    print(f"rows with an S/N             : {sn_rows} ({sn_rows / len(texts):.1%})")

    # ---- report vs resolution split --------------------------------------
    sample = texts[has_tilde].iloc[0]
    head, _, tail = sample.partition("~~~~~")
    print("\n--- example split ---")
    print(f"REPORT  : {head.strip()[:160]}")
    print(f"DETAIL  : {tail.strip()[:220]}")

    # crude technician extraction: trailing "Name" or "Name/Name" + optional date
    tech_re = re.compile(r"\b([A-Z][a-z]{2,}(?:/[A-Z][a-z]+)*)\s*\.?\s*(?:\d{1,2}[./]\d{1,2}[./]\d{2,4}|\d{5})?\s*$")
    techs = collections.Counter()
    for t in texts:
        m = tech_re.search(t.strip())
        if m:
            techs[m.group(1)] += 1
    print(f"\nrows w/ trailing name guess  : {sum(techs.values())}")
    print(f"top technician guesses       : {techs.most_common(15)}")


if __name__ == "__main__":
    main()
