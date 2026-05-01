// test_templates.rs
//
// Tests for SIP Migration Studio mapping templates.
// Run: cargo test --test test_templates (when integrated into the crate)
// or:  rustc --test test_templates.rs && ./test_templates (standalone)

/// Parses a TOML template file and validates its structure.
/// Each template must have a [template] section with id, name, source_system, version.
/// Each [[mappings]] block must have target_entity and at least one field.
fn validate_template_structure(toml_content: &str, template_id: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // Check required sections exist
    if !toml_content.contains("[template]") {
        errors.push(format!("{}: missing [template] section", template_id));
    }
    if !toml_content.contains("[[mappings]]") {
        errors.push(format!("{}: missing [[mappings]] section(s)", template_id));
    }

    // Check required template fields
    for required in &["id =", "name =", "source_system =", "version ="] {
        if !toml_content.contains(required) {
            errors.push(format!("{}: missing required field '{}'", template_id, required));
        }
    }

    // Check mappings have required fields
    let mappings_sections: Vec<&str> = toml_content
        .split("[[mappings]]")
        .skip(1)
        .collect();

    if mappings_sections.is_empty() {
        errors.push(format!("{}: no [[mappings]] found", template_id));
    }

    for section in &mappings_sections {
        if !section.contains("target_entity =") {
            errors.push(format!("{}: mapping missing target_entity", template_id));
        }
        if !section.contains("[[mappings.fields]]") {
            errors.push(format!("{}: mapping missing [[mappings.fields]]", template_id));
        }
    }

    // Validate each field entry has source and target
    let field_entries: Vec<&str> = toml_content
        .split("[[mappings.fields]]")
        .skip(1)
        .collect();

    for field in &field_entries {
        if !field.contains("source =") {
            errors.push(format!("{}: field missing source", template_id));
        }
        if !field.contains("target =") {
            errors.push(format!("{}: field missing target", template_id));
        }
    }

    errors
}

/// Validates that all referenced transforms exist in the known set.
fn validate_transforms(toml_content: &str, template_id: &str) -> Vec<String> {
    let valid_transforms = vec![
        "rename_field",
        "merge_fields",
        "split_field",
        "normalize_date",
        "normalize_email",
        "normalize_phone",
        "normalize_status",
        "normalize_priority",
        "map_enum_value",
        "create_location_hierarchy",
        "create_asset_hierarchy",
        "trim_clean_text",
        "convert_blank_to_null",
        "parse_json_field",
    ];

    let mut errors = Vec::new();

    for line in toml_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("transform = ") {
            // Extract the transform name (handle quotes and chaining with |)
            let value = trimmed
                .strip_prefix("transform = ")
                .unwrap_or("")
                .trim_matches('"');

            if value.is_empty() {
                continue;
            }

            // Handle chaining: "normalize_email | convert_blank_to_null"
            let transforms: Vec<&str> = value.split('|').map(|s| s.trim().trim_matches('"')).collect();

            for t in &transforms {
                if !valid_transforms.contains(t) && !t.is_empty() {
                    errors.push(format!(
                        "{}: unknown transform '{}' in line: {}",
                        template_id, t, trimmed
                    ));
                }
            }
        }
    }

    errors
}

// ── Template-specific tests ────────────────────────────────────────────────────

#[test]
fn test_generic_crm_csv_template() {
    let content = include_str!("../templates/generic-crm-csv.toml");
    let errors = validate_template_structure(content, "generic-crm-csv");
    assert!(errors.is_empty(), "Validation errors: {:?}", errors);

    let transform_errors = validate_transforms(content, "generic-crm-csv");
    assert!(transform_errors.is_empty(), "Transform errors: {:?}", transform_errors);

    // Specific checks: customer name is required
    assert!(content.contains("target_entity = \"customer\""));
    assert!(content.contains("target_entity = \"contact\""));
    assert!(content.contains("target = \"name\""));
    assert!(content.contains("target = \"first_name\""));
    assert!(content.contains("target = \"last_name\""));
}

#[test]
fn test_generic_cmms_csv_template() {
    let content = include_str!("../templates/generic-cmms-csv.toml");
    let errors = validate_template_structure(content, "generic-cmms-csv");
    assert!(errors.is_empty(), "Validation errors: {:?}", errors);

    let transform_errors = validate_transforms(content, "generic-cmms-csv");
    assert!(transform_errors.is_empty(), "Transform errors: {:?}", transform_errors);

    // Specific checks: asset and work_order entities present
    assert!(content.contains("target_entity = \"asset\""));
    assert!(content.contains("target_entity = \"work_order\""));
    assert!(content.contains("transform = \"normalize_date\""));
    assert!(content.contains("transform = \"normalize_status\""));
}

#[test]
fn test_salesforce_service_cloud_template() {
    let content = include_str!("../templates/salesforce-service-cloud.toml");
    let errors = validate_template_structure(content, "salesforce-service-cloud");
    assert!(errors.is_empty(), "Validation errors: {:?}", errors);

    let transform_errors = validate_transforms(content, "salesforce-service-cloud");
    assert!(transform_errors.is_empty(), "Transform errors: {:?}", transform_errors);

    // Specific checks: CaseNumber, Priority, Status fields mapped
    assert!(content.contains("CaseNumber"));
    assert!(content.contains("Priority"));
    assert!(content.contains("Status"));
    assert!(content.contains("Asset.Name"));
    assert!(content.contains("Account.Name"));
    assert!(content.contains("CreatedDate"));
    assert!(content.contains("ClosedDate"));

    // Verify enum mappings for Priority
    assert!(content.contains("Critical = \"critical\""));
    assert!(content.contains("\"In Progress\" = \"open\""));
}

#[test]
fn test_servicenow_csm_template() {
    let content = include_str!("../templates/servicenow-csm.toml");
    let errors = validate_template_structure(content, "servicenow-csm");
    assert!(errors.is_empty(), "Validation errors: {:?}", errors);

    let transform_errors = validate_transforms(content, "servicenow-csm");
    assert!(transform_errors.is_empty(), "Transform errors: {:?}", transform_errors);

    // Specific checks: ServiceNow field names
    assert!(content.contains("number"));
    assert!(content.contains("short_description"));
    assert!(content.contains("cmdb_ci.name"));
    assert!(content.contains("caller.email"));
    assert!(content.contains("opened_at"));
    assert!(content.contains("closed_at"));

    // Verify numeric enum mappings
    assert!(content.contains("\"1\" = \"critical\""));
    assert!(content.contains("\"2\" = \"high\""));
    assert!(content.contains("\"3\" = \"medium\""));
    assert!(content.contains("\"4\" = \"low\""));
    assert!(content.contains("\"6\" = \"completed\""));
}

#[test]
fn test_maximo_eam_template() {
    let content = include_str!("../templates/maximo-eam.toml");
    let errors = validate_template_structure(content, "maximo-eam");
    assert!(errors.is_empty(), "Validation errors: {:?}", errors);

    let transform_errors = validate_transforms(content, "maximo-eam");
    assert!(transform_errors.is_empty(), "Transform errors: {:?}", transform_errors);

    // Specific checks: Maximo UPPERCASE field names
    assert!(content.contains("ASSETNUM"));
    assert!(content.contains("SERIALNUM"));
    assert!(content.contains("WONUM"));
    assert!(content.contains("REPORTDATE"));
    assert!(content.contains("ACTFINISH"));
    assert!(content.contains("WARRANTYEXPDATE"));

    // Verify Maximo-specific status enums
    assert!(content.contains("OPERATING = \"operational\""));
    assert!(content.contains("NOTREADY = \"down\""));
    assert!(content.contains("DECOMMISSIONED = \"retired\""));

    // Verify all three entity types
    assert!(content.contains("target_entity = \"asset\""));
    assert!(content.contains("target_entity = \"work_order\""));
    assert!(content.contains("target_entity = \"location\""));
}
