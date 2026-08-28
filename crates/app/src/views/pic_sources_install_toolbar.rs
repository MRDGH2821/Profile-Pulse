use dioxus::prelude::*;
use profile_pulse_pic_source_plugin_host::{
    PACKAGE_EXTENSION, PicSourcePluginManifest, preview_package,
};

#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn PicSourceInstallToolbar(
    busy: bool,
    mut error: Signal<Option<String>>,
    mut pending_manifest: Signal<Option<PicSourcePluginManifest>>,
    mut pending_path: Signal<Option<String>>,
    mut approve_network: Signal<bool>,
    mut approve_secrets: Signal<bool>,
) -> Element {
    rsx! {
        div {
            class: "toolbar",
            button {
                disabled: busy,
                onclick: move |_| {
                    let Some(path) =
                        rfd::FileDialog::new()
                            .add_filter("Profile pic source plugin", &[PACKAGE_EXTENSION, "zip"])
                            .pick_file()
                    else {
                        return;
                    };
                    match preview_package(&path) {
                        Ok(manifest) => {
                            error.set(None);
                            pending_manifest.set(Some(manifest));
                            pending_path.set(Some(path.display().to_string()));
                            approve_network.set(false);
                            approve_secrets.set(false);
                        },
                        Err(err) => error.set(Some(err.to_string())),
                    }
                },
                "Install from file…"
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn PicSourceInstallToolbar(
    _busy: bool,
    _error: Signal<Option<String>>,
    _pending_manifest: Signal<Option<PicSourcePluginManifest>>,
    _pending_path: Signal<Option<String>>,
    _approve_network: Signal<bool>,
    _approve_secrets: Signal<bool>,
) -> Element {
    rsx! {
        div {
            class: "toolbar",
            p {
                class: "hint",
                "Built-in pic sources are available on web. User plugin install arrives in a later web build."
            }
        }
    }
}
