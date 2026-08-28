use crate::routes::Route;
use crate::state::AppState;
use crate::views::pic_sources_install_toolbar::PicSourceInstallToolbar;
use dioxus::prelude::*;
use profile_pulse_pic_source_plugin_api::PicSourceCapability;
use profile_pulse_pic_source_plugin_host::{
    PACKAGE_EXTENSION, PicSourcePluginManifest, PluginRuntimeKind, capability_name,
};

#[component]
pub fn PicSourcesSettings() -> Element {
    let state = use_context::<AppState>();
    let nav = navigator();
    let mut entries = use_signal(Vec::new);
    let mut error = use_signal(|| None::<String>);
    let mut status = use_signal(|| None::<String>);
    let mut pending_manifest = use_signal(|| None::<PicSourcePluginManifest>);
    let mut pending_path = use_signal(|| None::<String>);
    let mut approve_network = use_signal(|| false);
    let mut approve_secrets = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut reload = {
        let state = state.clone();
        move || {
            let list = state.plugin_registry.read().unwrap().list_entries();
            entries.set(list);
        }
    };
    use_effect(move || {
        reload();
    });
    rsx! {
        section {
            class: "panel",
            div {
                class: "toolbar",
                button {
                    class: "link-button",
                    onclick: move | _ | {
                        let _ = nav.push(Route::Profiles {});
                    },
                    "← Profiles"
                }
                h2 {
                    "Profile pic sources"
                }
            }
            p {
                class: "hint",
                "Manage built-in and WASM profile pic source plugins. Install packages with extension `.{PACKAGE_EXTENSION}`."
            }
            if let Some(message) = error() {
                p {
                    class: "error",
                    "{message}"
                }
            }
            if let Some(message) = status() {
                p {
                    class: "hint",
                    "{message}"
                }
            }
            PicSourceInstallToolbar {
                busy: busy(),
                error: error,
                pending_manifest: pending_manifest,
                pending_path: pending_path,
                approve_network: approve_network,
                approve_secrets: approve_secrets,
            }
            if let Some(manifest) = pending_manifest() {
                div {
                    class: "pic-convenience",
                    h3 {
                        "Approve install — {manifest.name}"
                    }
                    p {
                        class: "hint",
                        "ID: {manifest.id} · v{manifest.version}"
                    }
                    if manifest.capabilities.is_empty() {
                        p {
                            class: "hint",
                            "This plugin requests no special capabilities."
                        }
                    }
                    else {
                        p {
                            "Requested capabilities:"
                        }
                        ul {
                            for cap in manifest.capabilities.iter() {
                                li {
                                    "{cap}"
                                }
                            }
                        }
                        label {
                            input {
                                r#type: "checkbox",
                                checked: approve_network(),
                                onchange: move |event| approve_network.set(event.checked()),
                            }
                            " Allow network access"
                        }
                        label {
                            input {
                                r#type: "checkbox",
                                checked: approve_secrets(),
                                onchange: move |event| approve_secrets.set(event.checked()),
                            }
                            " Allow reading plugin secrets"
                        }
                    }
                    div {
                        class: "toolbar",
                        button {
                            disabled: busy(),
                            onclick: move | _ | {
                                pending_manifest.set(None);
                                pending_path.set(None);
                            },
                            "Cancel"
                        }
                        button {
                            disabled: busy(),
                            onclick: {
                                let state = state.clone();
                                let manifest = manifest.clone();
                                move |_| {
                                    let Some(path) = pending_path() else {
                                        return
                                    };
                                    let mut approved = Vec::new();
                                    if approve_network() {
                                        approved.push(PicSourceCapability::Network);
                                    }
                                    if approve_secrets() {
                                        approved.push(PicSourceCapability::ReadSecrets);
                                    }
                                    for cap in manifest.requested_capabilities() {
                                        if !approved.contains(&cap) {
                                            error.set(
                                                Some(
                                                    format!(
                                                        "Approve all requested capabilities before installing ({})",
                                                        capability_name(cap)
                                                    )
                                                )
                                            );
                                            return;
                                        }
                                    }
                                    busy.set(true);
                                    error.set(None);
                                    let state = state.clone();
                                    spawn(async move {
                                        let result = {
                                            let mut registry = state.plugin_registry.write().unwrap();
                                            registry.install_package(std::path::Path::new(&path), &approved).await
                                        };
                                        match result {
                                            Ok(id) => {
                                                status.set(Some(format!("Installed {id}")));
                                                pending_manifest.set(None);
                                                pending_path.set(None);
                                                let list = state.plugin_registry.read().unwrap().list_entries();
                                                entries.set(list);
                                            },
                                            Err(err) => error.set(Some(err.to_string())),
                                        }
                                        busy.set(false);
                                    });
                                }
                            },
                            "Install"
                        }
                    }
                }
            }
            ul {
                class: "profile-list",
                for entry in entries.read().iter() {
                    li {
                        class: "pic-candidate",
                        div {
                            strong {
                                "{entry.metadata.name}"
                            }
                            p {
                                class: "hint",
                                "{entry.metadata.id} · " if entry.runtime == PluginRuntimeKind:: Builtin {
                                    "built-in"
                                }
                                else {
                                    "wasm"
                                }
                            }
                        }
                        label {
                            input {
                                r#type: "checkbox",
                                checked: entry.enabled,
                                onchange: {
                                    let plugin_id = entry.metadata.id.0.clone();
                                    let state = state.clone();
                                    move |event| {
                                        let enabled = event.checked();
                                        let result =
                                            state.plugin_registry.write().unwrap().set_enabled(&plugin_id, enabled,);
                                        if let Err(err) = result {
                                            error.set(Some(err.to_string()));
                                        } else {
                                            let list = state.plugin_registry.read().unwrap().list_entries();
                                            entries.set(list);
                                        }
                                    }
                                },
                            }
                            " Enabled"
                        }
                    }
                }
            }
        }
    }
}
