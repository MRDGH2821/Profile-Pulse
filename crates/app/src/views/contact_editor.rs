use crate::state::AppState;
use chrono::Utc;
use dioxus::prelude::*;
use profile_pulse_core::{Contact, ContactId, EmailAddress, PhoneNumber, ProfileId, WebsiteLink};

#[component]
pub fn ContactEditor(
    profile_id: ProfileId,
    contact_id: ContactId,
    initial: Option<Contact>,
    on_saved: EventHandler<Contact>,
    on_deleted: EventHandler<()>,
) -> Element {
    let state = use_context::<AppState>();
    let mut display_name = use_signal(|| {
        initial
            .as_ref()
            .map(|c| c.display_name.clone())
            .unwrap_or_default()
    });
    let mut given_name = use_signal(|| {
        initial
            .as_ref()
            .and_then(|c| c.given_name.clone())
            .unwrap_or_default()
    });
    let mut family_name = use_signal(|| {
        initial
            .as_ref()
            .and_then(|c| c.family_name.clone())
            .unwrap_or_default()
    });
    let mut emails = use_signal(|| {
        initial
            .as_ref()
            .map(|c| c.emails.clone())
            .unwrap_or_default()
    });
    let mut phones = use_signal(|| {
        initial
            .as_ref()
            .map(|c| c.phones.clone())
            .unwrap_or_default()
    });
    let mut websites = use_signal(|| {
        initial
            .as_ref()
            .map(|c| c.websites.clone())
            .unwrap_or_default()
    });
    let mut error = use_signal(|| None::<String>);
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let is_new = initial.is_none();
    rsx! {
        div {
            class: "editor-form",
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
                "Display name" input {
                    r#type: "text",
                    value: "{display_name}",
                    oninput: move | event | display_name.set(event.value()),
                }
            }
            label {
                "Given name" input {
                    r#type: "text",
                    value: "{given_name}",
                    oninput: move | event | given_name.set(event.value()),
                }
            }
            label {
                "Family name" input {
                    r#type: "text",
                    value: "{family_name}",
                    oninput: move | event | family_name.set(event.value()),
                }
            }
            fieldset {
                class: "editor-fieldset",
                legend {
                    "Emails"
                }
                for(index, email) in emails.read().iter().enumerate() {
                    div {
                        class: "editor-row",
                        key: "{index}",
                        input {
                            r#type: "text",
                            placeholder: "Label",
                            value: "{email.label}",
                            oninput: move |event| {
                                let mut list = emails();
                                if let Some(entry) = list.get_mut(index) {
                                    entry.label = event.value();
                                    emails.set(list);
                                }
                            },
                        }
                        input {
                            r#type: "email",
                            placeholder: "Address",
                            value: "{email.address}",
                            oninput: move |event| {
                                let mut list = emails();
                                if let Some(entry) = list.get_mut(index) {
                                    entry.address = event.value();
                                    emails.set(list);
                                }
                            },
                        }
                        button {
                            r#type: "button",
                            onclick: move | _ | {
                                let mut list = emails();
                                list.remove(index);
                                emails.set(list);
                            },
                            "Remove"
                        }
                    }
                }
                button {
                    r#type: "button",
                    onclick: move | _ | {
                        let mut list = emails();
                        list.push(EmailAddress {
                            label: "home".into(),
                            address: String::new(),
                        });
                        emails.set(list);
                    },
                    "Add email"
                }
            }
            fieldset {
                class: "editor-fieldset",
                legend {
                    "Phones"
                }
                for(index, phone) in phones.read().iter().enumerate() {
                    div {
                        class: "editor-row",
                        key: "{index}",
                        input {
                            r#type: "text",
                            placeholder: "Label",
                            value: "{phone.label}",
                            oninput: move |event| {
                                let mut list = phones();
                                if let Some(entry) = list.get_mut(index) {
                                    entry.label = event.value();
                                    phones.set(list);
                                }
                            },
                        }
                        input {
                            r#type: "tel",
                            placeholder: "Number",
                            value: "{phone.number}",
                            oninput: move |event| {
                                let mut list = phones();
                                if let Some(entry) = list.get_mut(index) {
                                    entry.number = event.value();
                                    phones.set(list);
                                }
                            },
                        }
                        button {
                            r#type: "button",
                            onclick: move | _ | {
                                let mut list = phones();
                                list.remove(index);
                                phones.set(list);
                            },
                            "Remove"
                        }
                    }
                }
                button {
                    r#type: "button",
                    onclick: move | _ | {
                        let mut list = phones();
                        list.push(PhoneNumber {
                            label: "mobile".into(),
                            number: String::new(),
                        });
                        phones.set(list);
                    },
                    "Add phone"
                }
            }
            fieldset {
                class: "editor-fieldset",
                legend {
                    "Websites"
                }
                for(index, site) in websites.read().iter().enumerate() {
                    div {
                        class: "editor-row",
                        key: "{index}",
                        input {
                            r#type: "text",
                            placeholder: "Label",
                            value: "{site.label}",
                            oninput: move |event| {
                                let mut list = websites();
                                if let Some(entry) = list.get_mut(index) {
                                    entry.label = event.value();
                                    websites.set(list);
                                }
                            },
                        }
                        input {
                            r#type: "url",
                            placeholder: "URL",
                            value: "{site.url}",
                            oninput: move |event| {
                                let mut list = websites();
                                if let Some(entry) = list.get_mut(index) {
                                    entry.url = event.value();
                                    websites.set(list);
                                }
                            },
                        }
                        button {
                            r#type: "button",
                            onclick: move | _ | {
                                let mut list = websites();
                                list.remove(index);
                                websites.set(list);
                            },
                            "Remove"
                        }
                    }
                }
                button {
                    r#type: "button",
                    onclick: move | _ | {
                        let mut list = websites();
                        list.push(WebsiteLink {
                            label: "website".into(),
                            url: String::new(),
                        });
                        websites.set(list);
                    },
                    "Add website"
                }
            }
            div {
                class: "toolbar",
                button {
                    disabled: busy(),
                    onclick: {
                        let state = state.clone();
                        move |_| {
                            let name = display_name().trim().to_string();
                            if name.is_empty() {
                                error.set(Some("Display name is required".into()));
                                return;
                            }
                            busy.set(true);
                            error.set(None);
                            let given = {
                                let v = given_name().trim().to_string();
                                if v.is_empty() {
                                    None
                                } else {
                                    Some(v)
                                }
                            };
                            let family = {
                                let v = family_name().trim().to_string();
                                if v.is_empty() {
                                    None
                                } else {
                                    Some(v)
                                }
                            };
                            let contact = Contact {
                                id: contact_id,
                                profile_id,
                                display_name: name,
                                given_name: given,
                                family_name: family,
                                emails: emails().into_iter().filter(|e| !e.address.trim().is_empty()).collect(),
                                phones: phones().into_iter().filter(|p| !p.number.trim().is_empty()).collect(),
                                websites: websites().into_iter().filter(|w| !w.url.trim().is_empty()).collect(),
                                photo_content_hash: initial.as_ref().and_then(|c| c.photo_content_hash.clone()),
                                updated_at: Utc::now(),
                            };
                            let state = state.clone();
                            spawn(async move {
                                let result =
                                    state.contact_service.update_contact(contact.clone()).await.map(|_| contact);
                                match result {
                                    Ok(saved) => {
                                        status.set(Some("Contact saved".into()));
                                        on_saved.call(saved);
                                    },
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                                busy.set(false);
                            });
                        }
                    },
                    if is_new {
                        "Create contact"
                    }
                    else {
                        "Save changes"
                    }
                }
                if ! is_new {
                    button {
                        disabled: busy(),
                        class: "danger-button",
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    match state.contact_service.delete_contact(profile_id, contact_id).await {
                                        Ok(()) => on_deleted.call(()),
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Delete contact"
                    }
                }
            }
        }
    }
}
