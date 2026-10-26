//! Validate every plugin manifest shipped in the repository.
//!
//! The registry loads `plugins/*/plugin.toml` at startup and logs a warning when
//! one fails to parse or validate, so a broken manifest degrades silently: the
//! plugin simply disappears from the sidebar. These tests walk the real
//! `plugins/` directory next to the crate and assert each manifest loads,
//! validates, and keeps its `id` in sync with its directory name.

use std::path::{Path, PathBuf};

use sip_plugins::registry::load_manifest_from_file;
use sip_plugins::validation::validate_manifest;

fn plugins_dir() -> PathBuf {
    // crates/sip-plugins -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate should live at <repo>/crates/sip-plugins")
        .join("plugins")
}

fn manifest_paths() -> Vec<PathBuf> {
    let dir = plugins_dir();
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("plugin.toml"))
        .filter(|p| p.is_file())
        .collect();
    found.sort();
    found
}

#[test]
fn every_shipped_manifest_parses_and_validates() {
    let paths = manifest_paths();
    assert!(
        !paths.is_empty(),
        "expected at least one plugin manifest under {}",
        plugins_dir().display()
    );

    for path in &paths {
        let manifest = load_manifest_from_file(path)
            .unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()));

        // The directory name is the plugin id; the registry keys off both, so a
        // mismatch means the plugin is unreachable by its documented id.
        let dir_name = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|n| n.to_str())
            .expect("plugin directory should have a name");
        assert_eq!(
            manifest.id,
            dir_name,
            "{} declares id '{}' but lives in directory '{}'",
            path.display(),
            manifest.id,
            dir_name
        );

        let report = validate_manifest(&manifest);
        assert!(
            report.valid,
            "{} failed validation: {:?}",
            path.display(),
            report.errors
        );
    }
}

#[test]
fn test_bench_declares_a_mountable_route() {
    let path = plugins_dir().join("sip-test-bench/plugin.toml");
    assert!(
        path.is_file(),
        "sip-test-bench manifest missing at {}",
        path.display()
    );

    let manifest = load_manifest_from_file(&path).expect("manifest should parse");
    let ui = manifest.ui.as_ref().expect("sip-test-bench is a UI plugin");
    let public = manifest.public_view();
    let public_ui = public.ui.as_ref().expect("ui survives the public view");

    // The route must be visible to clients *and* carry a mount, otherwise the
    // shell discovers the path but has nothing to load for it.
    assert_eq!(
        public_ui.routes.len(),
        ui.routes.len(),
        "no route should be dropped by the public view"
    );
    let route = public_ui
        .routes
        .iter()
        .find(|r| r.path == "/test-bench")
        .expect("a /test-bench route should be declared");
    assert!(
        route.mount.is_some(),
        "/test-bench must declare a mount so the shell can host the plugin app"
    );

    // And the sidebar link must point at the same path, or the nav item 404s.
    let nav = public
        .navigation
        .iter()
        .find(|n| n.path == "/test-bench")
        .expect("the plugin should contribute a /test-bench navigation item");
    assert_eq!(nav.label, "Test Bench");
}
