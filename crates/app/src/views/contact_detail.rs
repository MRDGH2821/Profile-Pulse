use crate::routes::Route;
use crate::state::AppState;
use crate::views::contact_pull_conflict::ContactPullConflictPanel;
use crate::views::{ContactAvatar, ContactAvatarSize, ContactEditor, PicSelector};
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
    let app_state = state.clone();
    let nav = navigator();
    let mut tab = use_signal(|| {
        if contact_id == "new" {
            ContactTab::Editor
        } else {
            ContactTab::Details
        }
    });
    let mut contact = use_signal(|| None::<profile_pulse_core::Contact>);
    let mut error = use_signal(|| None::<String>);
    let mut sync_status = use_signal(|| None::<String>);
    let conflict_status = use_signal(|| None::<String>);
    let profile_uuid = match uuid::Uuid::parse_str(&profile_id) {
        Ok(id) => ProfileId(id),
        Err(_) => {
            return rsx! {
                p {
                    class: "error",
                    "Invalid profile id"
                }
            };
        }
    };
    let is_new = contact_id == "new";
    let contact_uuid = if is_new {
        ContactId(uuid::Uuid::new_v4())
    } else {
        match uuid::Uuid::parse_str(&contact_id) {
            Ok(id) => ContactId(id),
            Err(_) => {
                return rsx! {
                    p {
                        class: "error",
                        "Invalid contact id"
                    }
                };
            }
        }
    };
    use_effect({
        let state = app_state.clone();
        move || {
            if is_new {
                return;
            }
            let state = state.clone();
            spawn(async move {
                match state.storage.load_contact(profile_uuid, contact_uuid).await {
                    Ok(loaded) => {
                        error.set(None);
                        contact.set(Some(loaded));
                    }
                    Err(err) => error.set(Some(err.to_string())),
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
                        let _ = nav.push(Route::ContactList { profile_id: profile_id.clone(), });
                    },
                    "← Contacts"
                }
                h2 {
                    if is_new {
                        "New contact"
                    } else if let Some(c) = contact() {
                        "{c.display_name}"
                    } else {
                        "Contact"
                    }
                }
            }
            if ! is_new {
                nav {
                    class: "tabs",
                    button {
                        class: if tab() == ContactTab:: Details {
                            "tab active"
                        }
                        else {
                            "tab"
                        },
                        onclick: move | _ | tab.set(ContactTab::Details),
                        "Details"
                    }
                    button {
                        class: if tab() == ContactTab:: Editor {
                            "tab active"
                        }
                        else {
                            "tab"
                        },
                        onclick: move | _ | tab.set(ContactTab::Editor),
                        "Editor"
                    }
                    button {
                        class: if tab() == ContactTab:: PicSelector {
                            "tab active"
                        }
                        else {
                            "tab"
                        },
                        onclick: move | _ | tab.set(ContactTab::PicSelector),
                        "Profile pic selector"
                    }
                }
            }
            if let Some(message) = error() {
                p {
                    class: "error",
                    "{message}"
                }
            }
            if let Some(message) = sync_status() {
                p {
                    class: "hint",
                    "{message}"
                }
            }
            if let Some(message) = conflict_status() {
                p {
                    class: "hint",
                    "{message}"
                }
            }
            ContactPullConflictPanel {
                profile_uuid: profile_uuid,
                contact_uuid: contact_uuid,
                contact: contact,
                on_open_editor: move |_| tab.set(ContactTab::Editor),
                error: error,
                conflict_status: conflict_status,
            }
            match tab() {
                ContactTab::Details => rsx!{
                    if let Some(c) = contact() {
                        div {
                            class: "contact-detail-header",
                            ContactAvatar {
                                photo_hash: c.photo_content_hash.clone(),
                                display_name: c.display_name.clone(),
                                size: ContactAvatarSize::Lg,
                            }
                            h3 {
                                "{c.display_name}"
                            }
                        }
                        dl {
                            class: "details",
                            dt {
                                "Display name"
                            }
                            dd {
                                "{c.display_name}"
                            }
                            dt {
                                "Emails"
                            }
                            dd {
                                if c.emails.is_empty() {
                                    span {
                                        class: "hint",
                                        "None"
                                    }
                                }
                                else {
                                    ul {
                                        for email in c.emails.iter() {
                                            li {
                                                "{email.label}: {email.address}"
                                            }
                                        }
                                    }
                                }
                            }
                            dt {
                                "Phones"
                            }
                            dd {
                                if c.phones.is_empty() {
                                    span {
                                        class: "hint",
                                        "None"
                                    }
                                }
                                else {
                                    ul {
                                        for phone in c.phones.iter() {
                                            li {
                                                "{phone.label}: {phone.number}"
                                            }
                                        }
                                    }
                                }
                            }
                            dt {
                                "Websites"
                            }
                            dd {
                                if c.websites.is_empty() {
                                    span {
                                        class: "hint",
                                        "None"
                                    }
                                }
                                else {
                                    ul {
                                        for site in c.websites.iter() {
                                            li {
                                                "{site.label}: {site.url}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "toolbar",
                            button {
                                onclick: {
                                    let state = app_state.clone();
                                    let c = c.clone();
                                    move |_| {
                                        let state = state.clone();
                                        let c = c.clone();
                                        spawn(async move {
                                            match state.storage.load_profile(profile_uuid).await {
                                                Ok(profile) => {
                                                    match state.sync_service.push_contact(&profile, &c).await {
                                                        Ok(results) => {
                                                            sync_status.set(
                                                                Some(
                                                                    format!(
                                                                        "Pushed to {} sync target(s)",
                                                                        results.len()
                                                                    )
                                                                )
                                                            );
                                                            error.set(None);
                                                        },
                                                        Err(err) => error.set(Some(err.to_string())),
                                                    }
                                                },
                                                Err(err) => error.set(Some(err.to_string())),
                                            }
                                        });
                                    }
                                },
                                "Sync contact"
                            }
                        }
                    }
                    else if ! is_new {
                        p {
                            class: "hint",
                            "Loading contact…"
                        }
                    }
                },
                ContactTab::Editor => rsx!{
                    ContactEditor {
                        profile_id: profile_uuid,
                        contact_id: contact_uuid,
                        initial: contact(),
                        on_saved: {
                            let profile_id_for_save = profile_id.clone();
                            move |saved: profile_pulse_core::Contact| {
                                contact.set(Some(saved.clone()));
                                error.set(None);
                                if is_new {
                                    let _ = nav.push(Route::ContactDetail {
                                        profile_id: profile_id_for_save.clone(),
                                        contact_id: saved.id.0.to_string(),
                                    });
                                }
                            }
                        },
                        on_deleted: {
                            let profile_id_for_delete = profile_id.clone();
                            move |_| {
                                let _ = nav.push(Route::ContactList { profile_id: profile_id_for_delete.clone(), });
                            }
                        },
                    }
                },
                ContactTab::PicSelector => rsx!{
                    if let Some(c) = contact() {
                        PicSelector {
                            profile_id: profile_uuid,
                            contact_id: contact_uuid,
                            contact: c.clone(),
                            on_applied: move |updated| contact.set(Some(updated)),
                        }
                    }
                    else {
                        p {
                            class: "hint",
                            "Loading contact…"
                        }
                    }
                },
            }
        }
    }
}
