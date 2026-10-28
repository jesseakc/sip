//! Regression guard for the plugin route contract.
//!
//! `PluginManifest::public_view()` used to drop `ui.routes` when building
//! `PublicUiInfo`. Clients could therefore see a plugin in the sidebar (via
//! `navigation`) but had no way to learn which URL paths it owns or where its
//! application lives, which is why no plugin UI could ever be hosted.
//!
//! These tests pin the contract: routes survive the public view, and a route's
//! `mount` survives with them.

use sip_plugins::registry::load_manifest_from_str;

const MANIFEST: &str = r#"
id = "sip-test-bench"
name = "SIP Test Bench"
version = "0.1.0"
description = "Test harness UI plugin"
plugin_type = "ui"
enabled_by_default = true

[ui]
kind = "web"
framework = "react"
entrypoint = "/test-bench"
dev_url = "http://localhost:5173"
production_mount = "/api/v1/ui/plugins/sip-test-bench/app/index.html"

[ui.compatibility]
level = "standalone"
requires_shell = false
uses_sip_components = false
uses_theme_tokens = true
allows_global_css = true

[[ui.routes]]
id = "test-bench"
path = "/test-bench"
mount = "/api/v1/ui/plugins/sip-test-bench/app/index.html"
layout = "standalone"
title = "SIP Test Bench"
breadcrumb = "Test Bench"
required_permissions = ["migration:read"]

[[ui.actions]]
id = "test-bench.load-fixture"
label = "Load SMRT fixture"
icon = "flask-conical"
route = "/test-bench#/datasets"
placement = ["command_palette"]
required_permissions = ["migration:execute"]

[[navigation]]
id = "test-bench"
label = "Test Bench"
path = "/test-bench"
icon = "flask-conical"
order = 200
"#;

#[test]
fn ui_routes_survive_the_public_view() {
    let manifest = load_manifest_from_str(MANIFEST).expect("manifest should parse");
    let public = manifest.public_view();

    let ui = public.ui.as_ref().expect("ui section should be present");

    assert_eq!(
        ui.routes.len(),
        1,
        "the public view must expose the plugin's route table"
    );
    let route = &ui.routes[0];
    assert_eq!(route.id, "test-bench");
    assert_eq!(route.path, "/test-bench");
    assert_eq!(route.layout.as_deref(), Some("standalone"));
    assert_eq!(
        route.mount.as_deref(),
        Some("/api/v1/ui/plugins/sip-test-bench/app/index.html"),
        "the mount point is what tells the shell where to load the plugin's app"
    );
    assert_eq!(
        route.required_permissions,
        vec!["migration:read".to_string()]
    );
}

#[test]
fn ui_actions_survive_the_public_view() {
    let manifest = load_manifest_from_str(MANIFEST).expect("manifest should parse");
    let public = manifest.public_view();
    let ui = public.ui.as_ref().expect("ui section should be present");

    assert_eq!(ui.actions.len(), 1);
    assert_eq!(ui.actions[0].id, "test-bench.load-fixture");
    assert_eq!(ui.actions[0].placement, vec!["command_palette".to_string()]);
}

#[test]
fn compatibility_and_mount_fallback_are_exposed() {
    let manifest = load_manifest_from_str(MANIFEST).expect("manifest should parse");
    let public = manifest.public_view();
    let ui = public.ui.as_ref().expect("ui section should be present");

    let compatibility = ui
        .compatibility
        .as_ref()
        .expect("compatibility block should be present");
    assert_eq!(
        compatibility.level,
        Some(sip_plugins::manifest::UiCompatibilityLevel::Standalone)
    );
    assert!(!compatibility.requires_shell);
    assert_eq!(
        ui.production_mount.as_deref(),
        Some("/api/v1/ui/plugins/sip-test-bench/app/index.html"),
        "production_mount is the fallback when a route declares no mount"
    );
}

#[test]
fn a_route_without_a_mount_still_parses() {
    // Legacy shape: a declarative route whose component lives in the first-party
    // tree. It must keep loading, with mount absent rather than fabricated.
    let toml = r#"
id = "legacy-plugin"
name = "Legacy"
version = "1.0.0"
plugin_type = "ui"

[ui]
kind = "web"

[[ui.routes]]
id = "legacy-home"
path = "/legacy"
component = "LegacyDashboard"
"#;
    let manifest = load_manifest_from_str(toml).expect("manifest should parse");
    let public = manifest.public_view();
    let route = &public.ui.as_ref().expect("ui present").routes[0];
    assert_eq!(route.mount, None);
    assert_eq!(route.component.as_deref(), Some("LegacyDashboard"));
}
