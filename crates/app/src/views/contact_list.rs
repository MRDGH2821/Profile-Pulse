use crate::routes::Route;
use crate::state::{ActiveProfile, AppState};
use dioxus::prelude::*;
use profile_pulse_core::{ContactId, ProfileId};
use profile_pulse_storage::{ContactIndex, StorageBackend};

#[component]
pub fn ContactList(profile_id: String) -> Element {
    let state = use_context::<AppState>();
    let mut active_profile = use_context::<ActiveProfile>();
    let nav = navigator();
    let profile_uuid = match uuid::Uuid::parse_str(&profile_id) {
        Ok(id) => ProfileId(id),
        Err(_) => {
            return rsx! {
                p { class: "error", "Invalid profile id" }
            };
        }
    };

    active_profile.set(profile_uuid);

    let mut query = use_signal(String::new);
    let mut contacts = use_signal(Vec::<ContactSummary>::new);
    let mut error = use_signal(|| None::<String>);
    let mut profile_name = use_signal(|| profile_id.clone());

    let state_for_profile = state.clone();
    use_effect(move || {
        let state = state_for_profile.clone();
        spawn(async move {
            if let Ok(profile) = state.storage.load_profile(profile_uuid).await {
                profile_name.set(profile.name);
            }
        });
    });

    let state_for_search = state.clone();
    use_effect(move || {
        let state = state_for_search.clone();
        let search_query = query();
        spawn(async move {
            let result = load_contacts(state, profile_uuid, search_query).await;
            match result {
                Ok(list) => {
                    error.set(None);
                    contacts.set(list);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    });

    rsx! {
        section { class: "panel",
            div { class: "toolbar",
                button {
                    class: "link-button",
                    onclick: move |_| {
                        let _ = nav.push(Route::Profiles {});
                    },
                    "← Profiles"
                }
                h2 { "Contacts — {profile_name}" }
            }

            input {
                class: "search-input",
                r#type: "search",
                placeholder: "Search contacts",
                value: "{query}",
                oninput: move |event| query.set(event.value()),
            }

            if let Some(message) = error() {
                p { class: "error", "{message}" }
            }

            if contacts.read().is_empty() {
                p { class: "hint", "No contacts yet. Import and editing arrive in later phases." }
            }

            ul { class: "contact-list",
                for contact in contacts.read().iter() {
                    li {
                        button {
                            class: "list-button",
                            onclick: {
                                let contact_id = contact.id.0.to_string();
                                let profile_id = profile_id.clone();
                                move |_| {
                                    let _ = nav.push(Route::ContactDetail {
                                        profile_id: profile_id.clone(),
                                        contact_id: contact_id.clone(),
                                    });
                                }
                            },
                            "{contact.display_name}"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ContactSummary {
    id: ContactId,
    display_name: String,
}

async fn load_contacts(
    state: AppState,
    profile_id: ProfileId,
    query: String,
) -> Result<Vec<ContactSummary>, String> {
    let ids = if query.trim().is_empty() {
        state
            .storage
            .list_contact_ids(profile_id)
            .await
            .map_err(|e| e.to_string())?
    } else {
        state
            .index
            .search(profile_id, query.trim(), 100)
            .await
            .map_err(|e| e.to_string())?
    };

    let mut summaries = Vec::with_capacity(ids.len());
    for id in ids {
        let contact = state
            .storage
            .load_contact(profile_id, id)
            .await
            .map_err(|e| e.to_string())?;
        summaries.push(ContactSummary {
            id: contact.id,
            display_name: contact.display_name,
        });
    }
    summaries.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    Ok(summaries)
}
