use std::fs;
use std::path::{Path, PathBuf};

use profile_pulse_pic_source_plugin_api::PicSourceCapability;
use tokio::fs as async_fs;

use crate::error::HostError;
use crate::manifest::{
    PicSourcePluginManifest, PluginInstallState, INSTALL_FILE, MANIFEST_FILE, WASM_FILE,
};

pub fn plugins_root(data_root: &Path) -> PathBuf {
    data_root.join("pic-source-plugins")
}

pub fn plugin_install_dir(data_root: &Path, plugin_id: &str) -> PathBuf {
    plugins_root(data_root).join(sanitize_plugin_id(plugin_id))
}

fn sanitize_plugin_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn read_manifest_from_dir(dir: &Path) -> Result<PicSourcePluginManifest, HostError> {
    let bytes = fs::read(dir.join(MANIFEST_FILE))?;
    let manifest = PicSourcePluginManifest::parse(&bytes)?;
    manifest.validate()?;
    Ok(manifest)
}

pub fn read_install_state(dir: &Path) -> Result<PluginInstallState, HostError> {
    let path = dir.join(INSTALL_FILE);
    if !path.exists() {
        return Ok(PluginInstallState::default());
    }
    let text = fs::read_to_string(path)?;
    toml::from_str(&text).map_err(|e| HostError::Manifest(e.to_string()))
}

pub fn write_install_state(dir: &Path, state: &PluginInstallState) -> Result<(), HostError> {
    let text = toml::to_string_pretty(state).map_err(|e| HostError::Manifest(e.to_string()))?;
    fs::write(dir.join(INSTALL_FILE), text)?;
    Ok(())
}

pub fn approved_capabilities_from_install(
    manifest: &PicSourcePluginManifest,
    state: &PluginInstallState,
) -> Vec<PicSourceCapability> {
    let requested = manifest.requested_capabilities();
    if state.approved_capabilities.is_empty() {
        return requested;
    }
    requested
        .into_iter()
        .filter(|cap| {
            state
                .approved_capabilities
                .iter()
                .any(|name| capability_matches(name, *cap))
        })
        .collect()
}

fn capability_matches(name: &str, cap: PicSourceCapability) -> bool {
    name == crate::manifest::capability_name(cap)
}

/// Install a `.pp-pic-source-plugin` directory or zip archive into the data directory.
pub async fn install_package(
    data_root: &Path,
    source: &Path,
    approved: &[PicSourceCapability],
) -> Result<PathBuf, HostError> {
    let staging = tempfile::tempdir()?;
    let staged_dir = if source.is_dir() {
        copy_dir_recursive(source, staging.path())?;
        staging.path().to_path_buf()
    } else if source
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pp-pic-source-plugin") || ext == "zip")
    {
        extract_zip(source, staging.path())?;
        find_package_root(staging.path())?
    } else {
        return Err(HostError::InvalidPackage(
            "expected a plugin directory or .pp-pic-source-plugin zip".into(),
        ));
    };

    let manifest = read_manifest_from_dir(&staged_dir)?;
    manifest.validate()?;
    if !staged_dir.join(WASM_FILE).exists() {
        return Err(HostError::InvalidPackage(format!(
            "package missing {WASM_FILE}"
        )));
    }

    let dest = plugin_install_dir(data_root, &manifest.id);
    if dest.exists() {
        async_fs::remove_dir_all(&dest).await?;
    }
    copy_dir_recursive(&staged_dir, &dest)?;

    let install_state = PluginInstallState {
        enabled: true,
        approved_capabilities: approved
            .iter()
            .map(|cap| crate::manifest::capability_name(*cap).to_string())
            .collect(),
    };
    write_install_state(&dest, &install_state)?;
    Ok(dest)
}

/// Read and validate a package manifest without installing (install preview).
pub fn preview_package(source: &Path) -> Result<PicSourcePluginManifest, HostError> {
    if source.is_dir() {
        return read_manifest_from_dir(source);
    }
    let staging = tempfile::tempdir()?;
    extract_zip(source, staging.path())?;
    let root = find_package_root(staging.path())?;
    read_manifest_from_dir(&root)
}

fn find_package_root(staging: &Path) -> Result<PathBuf, HostError> {
    if staging.join(MANIFEST_FILE).exists() {
        return Ok(staging.to_path_buf());
    }
    for entry in fs::read_dir(staging)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join(MANIFEST_FILE).exists() {
            return Ok(path);
        }
    }
    Err(HostError::InvalidPackage(
        "archive does not contain manifest.toml".into(),
    ))
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), HostError> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| HostError::InvalidPackage(e.to_string()))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| HostError::InvalidPackage(e.to_string()))?;
        let outpath = dest.join(entry.name());
        if entry.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
            continue;
        }
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut outfile = fs::File::create(&outpath)?;
        std::io::copy(&mut entry, &mut outfile)
            .map_err(|e| HostError::InvalidPackage(e.to_string()))?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), HostError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
