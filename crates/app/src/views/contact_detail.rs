use crate::routes::Route;
use crate::state::AppState;
use dioxus::prelude::*;
use profile_pulse_core::{ContactId, ProfileId};
use profile_pulse_storage::StorageBackend;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ContactTab {
    Details,
    Editor,
    PicSelector,
}

#[component]
pub fn ContactDetail(profile_id: String, contact_id: String) -> Element {
    let state = use_context::<AppState>();
    let nav = navigator();
    let mut tab = use_signal(|| ContactTab::Details);
    let mut contact = use_signal(|| None::<profile_pulse_core::Contact>);
    let mut error = use_signal(|| None::<String>);

    let profile_uuid = match uuid::Uuid::parse_str(&profile_id) {
        Ok(id) => ProfileId(id),
        Err(_) => {
            return rsx! { p { class: "error", "Invalid profile id" } };
        }
    };
    let contact_uuid = match uuid::Uuid::parse_str(&contact_id) {
        Ok(id) => ContactId(id),
        Err(_) => {
            return rsx! { p { class: "error", "Invalid contact id" } };
        }
    };

    use_effect(move || {
        let state = state.clone();
        spawn(async move {
            match state
                .storage
                .load_contact(profile_uuid, contact_uuid)
                .await
            {
                Ok(loaded) => {
                    error.set(None);
                    contact.set(Some(loaded));
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        });
    });

    rsx! {
        section { class: "panel",
            div { class: "toolbar",
                button {
                    class: "link-button",
                    onclick: move |_| {
                        let _ = nav.push(Route::ContactList {
                            profile_id: profile_id.clone(),
                        });
                    },
                    "← Contacts"
                }
                h2 {
                    if let Some(c) = contact() {
                        "{c.display_name}"
                    } else {
                        "Contact"
                    }
                }
            }

            nav { class: "tabs",
                button {
                    class: if tab() == ContactTab::Details { "tab active" } else { "tab" },
                    onclick: move |_| tab.set(ContactTab::Details),
                    "Details"
                }
                button {
                    class: if tab() == ContactTab::Editor { "tab active" } else { "tab" },
                    onclick: move |_| tab.set(ContactTab::Editor),
                    "Editor"
                }
                button {
                    class: if tab() == ContactTab::PicSelector { "tab active" } else { "tab" },
                    onclick: move |_| tab.set(ContactTab::PicSelector),
                    "Profile pic selector"
                }
            }

            if let Some(message) = error() {
                p { class: "error", "{message}" }
            }

            match tab() {
                ContactTab::Details => rsx! {
                    if let Some(c) = contact() {
                        dl { class: "details",
                            dt { "Display name" }
                            dd { "{c.display_name}" }
                            dt { "Emails" }
                            dd {
                                if c.emails.is_empty() {
                                    span { class: "hint", "None" }
                                } else {
                                    ul {
                                        for email in c.emails.iter() {
                                            li { "{email.label}: {email.address}" }
                                        }
                                    }
                                }
                            }
                            dt { "Phones" }
                            dd {
                                if c.phones.is_empty() {
                                    span { class: "hint", "None" }
                                } else {
                                    ul {
                                        for phone in c.phones.iter() {
                                            li { "{phone.label}: {phone.number}" }
                                        }
                                    }
                                }
                            }
                            dt { "Websites" }
                            dd {
                                if c.websites.is_empty() {
                                    span { class: "hint", "None" }
                                } else {
                                    ul {
                                        for site in c.websites.iter() {
                                            li { "{site.label}: {site.url}" }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        p { class: "hint", "Loading contact…" }
                    }
                },
                ContactTab::Editor => rsx! {
                    p { class: "placeholder", "Contact editor — Phase 4" }
                },
                ContactTab::PicSelector => rsx! {
                    p { class: "placeholder", "Profile pic selector — Phase 2" }
                },
            }
        }
    }
}
