use crate::routes::Route;
use crate::state::{ActiveProfile, AppState};
use dioxus::prelude::*;
use profile_pulse_core::{Profile, ProfileId, SyncTargetConfig};
use profile_pulse_storage::StorageBackend;
use profile_pulse_sync::CardDavCredentials;

#[component]
pub fn SyncSettings() -> Element {
    let state = use_context::<AppState>();
    let active_profile = use_context::<ActiveProfile>();
    let nav = navigator();
    let mut profiles = use_signal(Vec::<Profile>::new);
    let mut selected_profile = use_signal(|| None::<ProfileId>);
    let mut google_enabled = use_signal(|| false);
    let mut carddav_enabled = use_signal(|| false);
    let mut carddav_url = use_signal(String::new);
    let mut carddav_username = use_signal(String::new);
    let mut carddav_password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    use_effect({
        let state = state.clone();
        let active_profile = active_profile;
        move || {
            let state = state.clone();
            spawn(async move {
                if let Ok(list) = state.list_profiles().await {
                    profiles.set(list.clone());
                    if selected_profile().is_none() {
                        selected_profile
                            .set(active_profile.id().or_else(|| list.first().map(|p| p.id)));
                    }
                }
            });
        }
    });
    use_effect({
        let state = state.clone();
        move || {
            let Some(profile_id) = selected_profile() else {
                return;
            };
            let state = state.clone();
            spawn(async move {
                if let Ok(profile) = state.storage.load_profile(profile_id).await {
                    google_enabled.set(
                        profile
                            .sync_targets
                            .iter()
                            .any(|t| matches!(t, SyncTargetConfig::Google { enabled: true })),
                    );
                    if let Some(SyncTargetConfig::CardDav { enabled, url }) = profile
                        .sync_targets
                        .iter()
                        .find(|t| matches!(t, SyncTargetConfig::CardDav { .. }))
                    {
                        carddav_enabled.set(*enabled);
                        carddav_url.set(url.clone());
                    } else {
                        carddav_enabled.set(false);
                        carddav_url.set(String::new());
                    }
                }
            });
        }
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
                    "Sync targets"
                }
            }
            p {
                class: "hint",
                "Push-only by default. Use the Sync button on a contact to push updates. Pull is available from sync settings when a remote link exists."
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
            label {
                "Profile" select {
                    class: "profile-select",
                    onchange: move | event | {
                        if let Ok(uuid) = uuid::Uuid::parse_str(&event.value()) {
                            selected_profile.set(Some(ProfileId(uuid)));
                        }
                    },
                    for profile in profiles.read().iter() {
                        option {
                            value: "{profile.id}",
                            selected: selected_profile() == Some(profile.id),
                            "{profile.name}"
                        }
                    }
                }
            }
            if let Some(profile_id) = selected_profile() {
                div {
                    class: "pic-convenience",
                    h3 {
                        "Google Contacts"
                    }
                    label {
                        input {
                            r#type: "checkbox",
                            checked: google_enabled(),
                            onchange: move |event| google_enabled.set(event.checked()),
                        }
                        " Enable Google Contacts sync"
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    match state.sync_service.link_google(profile_id).await {
                                        Ok(()) => {
                                            google_enabled.set(true);
                                            status.set(Some("Google account linked".into()));
                                        },
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Sign in with Google…"
                    }
                }
                div {
                    class: "pic-convenience",
                    h3 {
                        "CardDAV"
                    }
                    label {
                        input {
                            r#type: "checkbox",
                            checked: carddav_enabled(),
                            onchange: move |event| carddav_enabled.set(event.checked()),
                        }
                        " Enable CardDAV sync"
                    }
                    label {
                        "Server URL" input {
                            r#type: "url",
                            value: "{carddav_url}",
                            placeholder: "https://example.com/dav/contacts/",
                            oninput: move | event | carddav_url.set(event.value()),
                        }
                    }
                    label {
                        "Username" input {
                            r#type: "text",
                            value: "{carddav_username}",
                            oninput: move | event | carddav_username.set(event.value()),
                        }
                    }
                    label {
                        "App password" input {
                            r#type: "password",
                            value: "{carddav_password}",
                            oninput: move | event | carddav_password.set(event.value()),
                        }
                    }
                }
                button {
                    disabled: busy(),
                    onclick: {
                        let state = state.clone();
                        move |_| {
                            busy.set(true);
                            error.set(None);
                            let state = state.clone();
                            spawn(async move {
                                let mut profile = match state.storage.load_profile(profile_id).await {
                                    Ok(p) => p,
                                    Err(err) => {
                                        error.set(Some(err.to_string()));
                                        busy.set(false);
                                        return;
                                    },
                                };
                                profile.sync_targets.retain(|t| {
                                    !matches!(t, SyncTargetConfig::Google { .. } | SyncTargetConfig::CardDav { .. })
                                });
                                profile.sync_targets.push(SyncTargetConfig::Google { enabled: google_enabled(), });
                                profile.sync_targets.push(SyncTargetConfig::CardDav {
                                    enabled: carddav_enabled(),
                                    url: carddav_url().trim().to_string(),
                                });
                                if carddav_enabled() {
                                    let credentials = CardDavCredentials {
                                        username: carddav_username().trim().to_string(),
                                        password: carddav_password(),
                                    };
                                    if let Err(err) =
                                        state.sync_service.save_carddav_credentials(profile_id, &credentials) {
                                        error.set(Some(err.to_string()));
                                        busy.set(false);
                                        return;
                                    }
                                }
                                match state.contact_service.update_profile_settings(profile).await {
                                    Ok(_) => status.set(Some("Sync targets saved".into())),
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                                busy.set(false);
                            });
                        }
                    },
                    "Save sync targets"
                }
            }
        }
    }
}
