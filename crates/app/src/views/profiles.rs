use crate::routes::Route;
use crate::state::{ActiveProfile, AppState};
use dioxus::prelude::*;
use profile_pulse_core::Profile;
use profile_pulse_core::SyncTargetConfig;

#[component]
pub fn Profiles() -> Element {
    let state = use_context::<AppState>();
    let active_profile = use_context::<ActiveProfile>();
    let mut profiles = use_signal(Vec::<Profile>::new);
    let mut new_name = use_signal(String::new);
    let mut enable_google = use_signal(|| false);
    let mut enable_carddav = use_signal(|| false);
    let mut carddav_url = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let nav = navigator();

    let state_for_load = state.clone();
    use_effect(move || {
        let state = state_for_load.clone();
        spawn(async move {
            match state.list_profiles().await {
                Ok(list) => profiles.set(list),
                Err(message) => error.set(Some(message)),
            }
        });
    });

    rsx! {
        section { class: "panel",
            h2 { "Profiles" }
            p { class: "hint", "Choose a profile or create a new local contact book." }

            if let Some(message) = error() {
                p { class: "error", "{message}" }
            }

            ul { class: "profile-list",
                for profile in profiles.read().iter() {
                    li {
                        button {
                            class: "list-button",
                            onclick: {
                                let profile_id = profile.id;
                                let mut active_profile = active_profile;
                                move |_| {
                                    active_profile.set(profile_id);
                                    let _ = nav.push(Route::ContactList {
                                        profile_id: profile_id.0.to_string(),
                                    });
                                }
                            },
                            "{profile.name}"
                        }
                    }
                }
            }

            div { class: "create-profile",
                h3 { "Create profile" }
                input {
                    r#type: "text",
                    placeholder: "Profile name",
                    value: "{new_name}",
                    oninput: move |event| new_name.set(event.value()),
                }
                label {
                    input {
                        r#type: "checkbox",
                        checked: enable_google(),
                        onchange: move |event| enable_google.set(event.checked()),
                    }
                    " Link Google Contacts"
                }
                label {
                    input {
                        r#type: "checkbox",
                        checked: enable_carddav(),
                        onchange: move |event| enable_carddav.set(event.checked()),
                    }
                    " Link CardDAV"
                }
                if enable_carddav() {
                    input {
                        r#type: "url",
                        placeholder: "CardDAV URL",
                        value: "{carddav_url}",
                        oninput: move |event| carddav_url.set(event.value()),
                    }
                }
                button {
                    onclick: {
                        let state = state.clone();
                        move |_| {
                            let name = new_name();
                            if name.trim().is_empty() {
                                error.set(Some("Enter a profile name".into()));
                                return;
                            }
                            let state = state.clone();
                            let mut active_profile = active_profile;
                            spawn(async move {
                                let mut sync_targets = Vec::new();
                                if enable_google() {
                                    sync_targets.push(SyncTargetConfig::Google { enabled: true });
                                }
                                if enable_carddav() {
                                    sync_targets.push(SyncTargetConfig::CardDav {
                                        enabled: true,
                                        url: carddav_url().trim().to_string(),
                                    });
                                }
                                match state.create_profile(name.trim(), sync_targets).await {
                                    Ok(profile) => {
                                        error.set(None);
                                        active_profile.set(profile.id);
                                        profiles.write().push(profile.clone());
                                        new_name.set(String::new());
                                        let _ = nav.push(Route::ContactList {
                                            profile_id: profile.id.0.to_string(),
                                        });
                                    }
                                    Err(message) => error.set(Some(message)),
                                }
                            });
                        }
                    },
                    "Create"
                }
            }
        }
    }
}
