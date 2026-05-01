# Transform Function Reference

The Migration Studio transform engine provides 14 built-in transform functions that normalize,
clean, and reshape source data before it is imported into SIP.

## Usage in Mapping Templates

Each mapping field can reference a transform function via the `transform` key:

```toml
[[mappings.fields]]
source = "email"
target = "email"
transform = "normalize_email"
```

Some transforms accept configuration via an optional `config` key:

```toml
[[mappings.fields]]
source = "priority"
target = "priority"
transform = "normalize_priority"
config = { lookup = { "1" = "critical", "2" = "high", "3" = "medium", "4" = "low" } }
```

---

## 1. `rename_field`

**What it does:**
Rename a source field name to a target field name. This is implicit in every mapping (the
engine always reads from `source` and writes to `target`), but `rename_field` can be used
explicitly when the transform needs to be recorded or when the field name itself is dynamic.

**Input format:** `string` — the source field value

**Output format:** `string` — the value unchanged, mapped to the target field

**Configuration options:** None

**Example:**
```
Source:  FIELD_OLD_NAME → "Chiller Unit 3"
Target:  name = "Chiller Unit 3"
```

**In template:**
```toml
transform = "rename_field"
```

---

## 2. `merge_fields`

**What it does:**
Combines multiple source field values into a single target field. Commonly used to concatenate
`first_name` and `last_name` into a full name, or join address components.

**Input format:** `string` — multiple field references (e.g., `field1` + `" "` + `field2`)

**Output format:** `string` — the merged result

**Configuration options:**
| Option      | Type   | Description                          | Default |
|-------------|--------|--------------------------------------|---------|
| `fields`    | array  | Source field names to merge          | —       |
| `separator` | string | Delimiter between joined values      | `" "`   |
| `order`     | array  | Order of fields in the output        | —       |

**Example:**
```
Input:  first_name = "John", last_name = "Smith"
Config: { fields = ["first_name", "last_name"], separator = " " }
Output: "John Smith"
```

**In template:**
```toml
transform = "merge_fields"
config = { fields = ["first_name", "last_name"], separator = " " }
```

---

## 3. `split_field`

**What it does:**
Splits a single source field value into an array on a delimiter. Commonly used for
comma-separated tags, skill lists, or multi-value attributes.

**Input format:** `string` — a delimited string

**Output format:** `Vec<string>` — an array of trimmed strings

**Configuration options:**
| Option      | Type   | Description                    | Default |
|-------------|--------|--------------------------------|---------|
| `delimiter` | string | Character to split on          | `","`   |
| `max_items` | int    | Maximum items to produce       | —       |
| `trim`      | bool   | Trim whitespace from each item | `true`  |

**Example:**
```
Input:  "hvac, critical, backup"
Config: { delimiter = "," }
Output: ["hvac", "critical", "backup"]
```

**In template:**
```toml
transform = "split_field"
config = { delimiter = ",", max_items = 10 }
```

---

## 4. `normalize_date`

**What it does:**
Converts various date/datetime formats into ISO 8601 (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SSZ`).
Recognizes common formats: `MM/DD/YYYY`, `DD/MM/YYYY`, `Mon DD, YYYY`, `YYYYMMDD`, Unix timestamps,
and Salesforce/ServiceNow datetime strings.

**Input format:** `string` — a date in any recognized format

**Output format:** `string` — ISO 8601 date string (`YYYY-MM-DD`)

**Configuration options:**
| Option         | Type   | Description                           | Default    |
|----------------|--------|---------------------------------------|------------|
| `format_hint`  | string | Expected source format (if known)     | —          |
| `timezone`     | string | IANA timezone for conversion          | `"UTC"`    |
| `on_invalid`   | string | Behavior: `"skip"`, `"null"`, `"error"` | `"null"`  |

**Example:**
```
Input:  "12/25/2025"
Output: "2025-12-25"

Input:  "2025-05-15T14:30:00.000+0000"
Output: "2025-05-15"

Input:  "Jan 1, 2026"
Output: "2026-01-01"
```

**In template:**
```toml
transform = "normalize_date"
config = { format_hint = "%m/%d/%Y", timezone = "America/Chicago" }
```

---

## 5. `normalize_email`

**What it does:**
Lowercases the email address, trims whitespace, and validates basic email format
(must contain `@` and a domain part). Returns `null` for invalid emails unless
`on_invalid` is configured otherwise.

**Input format:** `string` — a raw email address

**Output format:** `string` or `null` — a normalized email address

**Configuration options:**
| Option       | Type   | Description                           | Default    |
|--------------|--------|---------------------------------------|------------|
| `on_invalid` | string | Behavior: `"skip"`, `"null"`, `"error"` | `"null"`  |

**Example:**
```
Input:  "  John.Doe@Example.COM  "
Output: "john.doe@example.com"

Input:  "not-an-email"
Output: null (when on_invalid = "null")
```

**In template:**
```toml
transform = "normalize_email"
```

---

## 6. `normalize_phone`

**What it does:**
Strips all non-digit characters from a phone number and standardizes to E.164-ish format
(country code + digits). Defaults to US (+1) if no country code is present and the number
is 10 digits.

**Input format:** `string` — a raw phone number

**Output format:** `string` or `null` — a normalized phone number

**Configuration options:**
| Option          | Type   | Description                    | Default |
|-----------------|--------|--------------------------------|---------|
| `default_cc`    | string | Default country code           | `"1"`   |
| `strip_ext`     | bool   | Remove extensions (x123)       | `true`  |
| `min_digits`    | int    | Minimum digits for valid phone | `7`     |

**Example:**
```
Input:  "(555) 010-0123"
Output: "+15550100123"

Input:  "555.010.0123"
Output: "+15550100123"

Input:  "+44 20 7946 0958"
Output: "+442079460958"

Input:  "555-0100 x1234"
Output: "+15550100123"  (extension stripped)
```

**In template:**
```toml
transform = "normalize_phone"
config = { default_cc = "1", strip_ext = true }
```

---

## 7. `normalize_status`

**What it does:**
Maps source-specific status values to SIP canonical status values using a lookup table.
If no lookup table is provided, it falls back to lowercase/trim. Common canonical values
include: `draft`, `open`, `in_progress`, `on_hold`, `completed`, `closed`, `cancelled`,
`operational`, `degraded`, `down`, `maintenance`, `retired`.

**Input format:** `string` — a source status value

**Output format:** `string` — a canonical SIP status value

**Configuration options:**
| Option     | Type   | Description                       | Default |
|------------|--------|-----------------------------------|---------|
| `lookup`   | map    | Source → canonical value mapping  | —       |
| `fallback` | string | Value to use if no match found    | —       |
| `target`   | string | Target entity type for defaults   | —       |

**Example:**
```
Input:  "In Progress"
Config: { lookup = { "New" = "draft", "In Progress" = "open", "Closed" = "completed" } }
Output: "open"

Input:  "OPERATING"
Config: { lookup = { "OPERATING" = "operational", "NOTREADY" = "down" } }
Output: "operational"
```

**In template:**
```toml
transform = "normalize_status"
config = { lookup = { "New" = "draft", "In Progress" = "open", "Closed" = "completed" } }
```

---

## 8. `normalize_priority`

**What it does:**
Maps priority values from source systems to SIP canonical priority enum values.
Canonical values are: `critical`, `high`, `medium`, `low`. Falls back to case-insensitive
matching if no lookup table is configured.

**Input format:** `string` or `int` — a source priority value

**Output format:** `string` — a canonical priority value

**Configuration options:**
| Option     | Type   | Description                       | Default |
|------------|--------|-----------------------------------|---------|
| `lookup`   | map    | Source → canonical value mapping  | —       |
| `fallback` | string | Value to use if no match found    | `"medium"` |

**Example:**
```
Input:  "1"
Config: { lookup = { "1" = "critical", "2" = "high", "3" = "medium", "4" = "low" } }
Output: "critical"

Input:  "High"
Output: "high"

Input:  "CRITICAL"
Output: "critical"

Input:  "unknown-priority"
Config: { fallback = "low" }
Output: "low"
```

**In template:**
```toml
transform = "normalize_priority"
config = { lookup = { "1" = "critical", "2" = "high", "3" = "medium", "4" = "low" } }
```

---

## 9. `map_enum_value`

**What it does:**
A generic enum/value mapper. Maps any source value to any target value using a
configurable lookup table. Unlike `normalize_status` and `normalize_priority`, this
function does not have built-in canonical values — it is fully config-driven.

**Input format:** `string` — any source value

**Output format:** `string` — the mapped target value

**Configuration options:**
| Option       | Type   | Description                           | Default  |
|--------------|--------|---------------------------------------|----------|
| `lookup`     | map    | Source → target value mapping         | —        |
| `fallback`   | string | Value to use if no match found        | —        |
| `on_no_match` | string | Behavior: `"keep"`, `"null"`, `"fallback"`, `"error"` | `"keep"` |

**Example:**
```
Input:  "PM"
Config: { lookup = { "PM" = "preventive", "CM" = "corrective", "EM" = "emergency" } }
Output: "preventive"

Input:  "WAPPR"
Config: { lookup = { "WAPPR" = "draft", "APPR" = "open", "COMP" = "completed" } }
Output: "draft"
```

**In template:**
```toml
transform = "map_enum_value"
config = { lookup = { "PM" = "preventive", "CM" = "corrective" }, fallback = "corrective" }
```

---

## 10. `create_location_hierarchy`

**What it does:**
Parses a hierarchical location path string (delimited by `/`, `\`, `>`, or `→`) and creates
a Location entity or returns the leaf node external ID. The hierarchy is preserved so parent
locations can be created automatically during import.

**Input format:** `string` — a path like `"Site / Building / Floor / Room"`

**Output format:** `string` — the external ID of the leaf location; intermediate locations are created as needed

**Configuration options:**
| Option       | Type   | Description                       | Default |
|--------------|--------|-----------------------------------|---------|
| `delimiter`  | string | Path delimiter                    | `"/"`   |
| `create_missing` | bool | Auto-create parent locations      | `true`  |
| `max_depth`  | int    | Maximum hierarchy depth           | `10`    |

**Example:**
```
Input:  "Building A / Floor 3 / Room 301"
Output: "Building A Floor 3 Room 301"  (external_id of leaf)

Input:  "Plant-1 > Assembly > Station-4"
Config: { delimiter = ">" }
Output: "Plant-1 Assembly Station-4"
```

**In template:**
```toml
transform = "create_location_hierarchy"
config = { delimiter = "/", create_missing = true, max_depth = 10 }
```

---

## 11. `create_asset_hierarchy`

**What it does:**
Parses a parent-child asset path and creates asset hierarchy relationships. Links to
parent assets via `parent_external_id` so the import engine can resolve nested asset
structures.

**Input format:** `string` — a path like `"Site/Building/Floor/Room"` or `"Parent > Child"`

**Output format:** `string` — the external ID of the immediate parent asset

**Configuration options:**
| Option           | Type   | Description                       | Default |
|------------------|--------|-----------------------------------|---------|
| `delimiter`      | string | Path delimiter                    | `"/"`   |
| `create_missing` | bool   | Auto-create parent assets         | `true`  |
| `max_depth`      | int    | Maximum hierarchy depth           | `10`    |

**Example:**
```
Input:  "Chiller Plant/Chiller Unit 3/Compressor A"
Output: "Chiller Unit 3"  (parent_external_id of Compressor A)

Input:  "Assembly Line > Conveyor > Section C"
Config: { delimiter = ">" }
Output: "Conveyor"
```

**In template:**
```toml
transform = "create_asset_hierarchy"
config = { delimiter = "/", create_missing = true }
```

---

## 12. `trim_clean_text`

**What it does:**
Trims leading and trailing whitespace, normalizes internal whitespace (collapses multiple
spaces/tabs into a single space), and removes non-printable control characters (except
newlines and tabs, which are converted to spaces). Does NOT strip Unicode — it preserves
international characters.

**Input format:** `string` — raw text

**Output format:** `string` — cleaned text

**Configuration options:**
| Option            | Type | Description                        | Default |
|-------------------|------|------------------------------------|---------|
| `preserve_newlines` | bool | Keep line breaks as-is             | `false` |
| `strip_unicode`   | bool | Remove non-ASCII characters        | `false` |
| `max_length`      | int  | Truncate to this length (0 = none) | `0`     |

**Example:**
```
Input:  "  HVAC   Unit \t A-1  \n  "
Output: "HVAC Unit A-1"

Input:  "Chiller\u0000 Unit 3"
Output: "Chiller Unit 3"

Input:  "   Some very long string that exceeds the max...   "
Config: { max_length = 20 }
Output: "Some very long stri"
```

**In template:**
```toml
transform = "trim_clean_text"
```

---

## 13. `convert_blank_to_null`

**What it does:**
Converts commonly used "empty" placeholder values to `null`. Recognizes: empty string (`""`),
whitespace-only strings, `"N/A"`, `"n/a"`, `"null"`, `"NULL"`, `"None"`, `"-"`, `"--"`,
`"TBD"`, `"NONE"`, `"."`.

**Input format:** `string` — any value

**Output format:** `string` or `null` — the value unchanged, or null if it matches a blank placeholder

**Configuration options:**
| Option       | Type   | Description                           | Default |
|--------------|--------|---------------------------------------|---------|
| `extra_blanks` | array | Additional placeholder strings to treat as blank | `[]` |
| `case_sensitive` | bool | Match placeholders case-sensitively   | `false` |

**Example:**
```
Input:  ""
Output: null

Input:  "N/A"
Output: null

Input:  "   "
Output: null

Input:  "   "
Output: null

Input:  "Install replacement pump"
Output: "Install replacement pump"

Input:  "n/a"
Output: null

Input:  "TBD"
Output: null

Input:  "UNKNOWN"
Config: { extra_blanks = ["UNKNOWN"] }
Output: null
```

**In template:**
```toml
transform = "convert_blank_to_null"
```

---

## 14. `parse_json_field`

**What it does:**
Parses a JSON-encoded string field into a structured value (object, array, etc.). Used when
source systems export structured data as JSON strings in flat-file formats (e.g., custom
attributes, specifications, metadata).

**Input format:** `string` — a JSON-encoded string

**Output format:** `serde_json::Value` — the parsed JSON value, or `null` on parse failure

**Configuration options:**
| Option         | Type   | Description                           | Default  |
|----------------|--------|---------------------------------------|----------|
| `on_error`     | string | Behavior: `"null"`, `"string"`, `"error"` | `"null"` |
| `flatten`      | bool   | Flatten parsed object to top-level fields | `false` |
| `flatten_prefix` | string | Prefix for flattened keys             | `""`     |

**Example:**
```
Input:  "{\"color\": \"red\", \"weight_kg\": 450}"
Output: { "color": "red", "weight_kg": 450 }

Input:  "[\"tag1\", \"tag2\", \"tag3\"]"
Output: ["tag1", "tag2", "tag3"]

Input:  "{\"specs\": {\"rpm\": 1800, \"voltage\": 480}}"
Config: { flatten = true, flatten_prefix = "spec_" }
Output: { "spec_rpm": 1800, "spec_voltage": 480 }

Input:  "not valid json"
Output: null
```

**In template:**
```toml
transform = "parse_json_field"
config = { on_error = "null", flatten = true, flatten_prefix = "custom_" }
```

---

## Transform Chaining

Transforms can be chained by separating transform names with a pipe (`|`). They are
applied left to right:

```toml
[[mappings.fields]]
source = "description"
target = "notes"
transform = "trim_clean_text | convert_blank_to_null"
```

The above will first trim/clean the text, then convert blank placeholders to `null`.
