use std::path::PathBuf;
use std::sync::Arc;

use profile_pulse_pic_source_plugin_api::{ContactContext, ProfilePicSourcePlugin};
use profile_pulse_pic_source_plugin_host::{
    install_package, PicSourcePluginRegistry, WasmPicSourcePlugin,
};

fn sample_plugin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../pic-source-plugins/sample-hello-pic-source")
}

fn sample_wasm_path() -> PathBuf {
    sample_plugin_dir().join("pic_source_plugin.wasm")
}

#[test]
fn parses_valid_manifest() {
    let text = include_str!("../../../pic-source-plugins/sample-hello-pic-source/manifest.toml");
    let manifest = profile_pulse_pic_source_plugin_host::PicSourcePluginManifest::parse(text.as_bytes())
        .expect("parse manifest");
    manifest.validate().expect("validate");
    assert_eq!(manifest.id, "community.sample-hello-pic-source");
}

#[tokio::test]
async fn sample_wasm_plugin_discovers_and_fetches() {
    if !sample_wasm_path().exists() {
        eprintln!("skip: build sample wasm with `./scripts/build-sample-pic-source.sh`");
        return;
    }
    let host = Arc::new(profile_pulse_pic_source_plugin_host::DesktopHostApi::new(
        tempfile::tempdir().unwrap().path().join("cache"),
    ));
    let plugin = WasmPicSourcePlugin::from_install_dir(&sample_plugin_dir(), host, vec![])
        .expect("load wasm plugin");
    let ctx = ContactContext {
        emails: vec!["ada@example.com".into()],
        websites: vec![],
        existing_photo_hash: None,
    };
    let candidates = plugin.discover_sources(&ctx).await.expect("discover");
    assert!(!candidates.is_empty());
    let pic = plugin.fetch_pic(&candidates[0]).await.expect("fetch");
    assert!(!pic.bytes.is_empty());
}

#[tokio::test]
async fn installs_sample_package_into_data_dir() {
    if !sample_wasm_path().exists() {
        eprintln!("skip: build sample wasm first");
        return;
    }
    let data_root = tempfile::tempdir().unwrap();
    install_package(data_root.path(), &sample_plugin_dir(), &[])
        .await
        .expect("install");
    let host = Arc::new(profile_pulse_pic_source_plugin_host::DesktopHostApi::new(
        data_root.path().join("cache"),
    ));
    let mut registry = PicSourcePluginRegistry::new(data_root.path().to_path_buf(), host);
    registry.load_installed_wasm_plugins().expect("load");
    let entries = registry.list_entries();
    assert!(entries.iter().any(|e| e.metadata.id.0.contains("sample-hello")));
}
