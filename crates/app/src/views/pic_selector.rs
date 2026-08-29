use crate::state::AppState;
use dioxus::prelude::*;
use profile_pulse_core::{Contact, ContactId, PicSourcePluginId, ProfileId};
use profile_pulse_pic_source_plugin_api::{ContactContext, ProfilePicCandidate};
use profile_pulse_pic_source_plugin_host::{
    PicSourcePluginRegistry, github_candidate_for_username, gitlab_candidate_for_username,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct PicCandidateRow {
    plugin_id: PicSourcePluginId,
    candidate: ProfilePicCandidate,
}

#[component]
pub fn PicSelector(
    profile_id: ProfileId,
    contact_id: ContactId,
    contact: Contact,
    on_applied: EventHandler<Contact>,
) -> Element {
    let state = use_context::<AppState>();
    let mut candidates = use_signal(Vec::<PicCandidateRow>::new);
    let mut error = use_signal(|| None::<String>);
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);
    let mut github_username = use_signal(String::new);
    let mut gitlab_username = use_signal(String::new);
    let contact_for_effect = contact.clone();
    let state_for_discover = state.clone();
    use_effect(move || {
        let state = state_for_discover.clone();
        let contact = contact_for_effect.clone();
        spawn(async move {
            let ctx = ContactContext::from_contact(&contact);
            let plugins = state.plugin_registry.read().unwrap().enabled_plugins();
            match PicSourcePluginRegistry::discover_plugins(&plugins, &ctx).await {
                Ok(list) => {
                    error.set(None);
                    candidates.set(
                        list.into_iter()
                            .map(|(plugin_id, candidate)| PicCandidateRow {
                                plugin_id,
                                candidate,
                            })
                            .collect(),
                    );
                }
                Err(err) => error.set(Some(err.to_string())),
            }
        });
    });
    rsx! {
        div {
            class: "pic-selector",
            p {
                class: "hint",
                "Discover profile pictures from built-in sources. Select a candidate to fetch and apply."
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
            if contact.photo_content_hash.is_some() {
                p {
                    class: "hint",
                    "This contact already has a profile picture hash stored."
                }
            }
            div {
                class: "pic-convenience",
                h3 {
                    "Try a username"
                }
                p {
                    class: "hint",
                    "Look up GitHub or GitLab avatars without adding a website link first."
                }
                div {
                    class: "create-profile",
                    input {
                        r#type: "text",
                        placeholder: "GitHub username",
                        value: "{github_username}",
                        oninput: move |event| github_username.set(event.value()),
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            move |_| {
                                if let Some(candidate) = github_candidate_for_username(&github_username()) {
                                    let mut list = candidates.write().clone();
                                    let plugin_id =
                                        PicSourcePluginId("profile-pulse.builtin.github-pic-source".into(),);
                                    if !list.iter().any(|row| {
                                        row.plugin_id == plugin_id && row.candidate.source_key == candidate.source_key
                                    }) {
                                        list.push(PicCandidateRow {
                                            plugin_id,
                                            candidate,
                                        });
                                        candidates.set(list);
                                    }
                                } else {
                                    error.set(Some(
                                        "Enter a GitHub username or profile URL (e.g. octocat or https://github.com/octocat)".into(),
                                    ));
                                }
                            }
                        },
                        "Add GitHub candidate"
                    }
                    input {
                        r#type: "text",
                        placeholder: "GitLab username",
                        value: "{gitlab_username}",
                        oninput: move |event| gitlab_username.set(event.value()),
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            move |_| {
                                if let Some(candidate) = gitlab_candidate_for_username(&gitlab_username()) {
                                    let mut list = candidates.write().clone();
                                    let plugin_id =
                                        PicSourcePluginId("profile-pulse.builtin.gitlab-pic-source".into(),);
                                    if !list.iter().any(|row| {
                                        row.plugin_id == plugin_id && row.candidate.source_key == candidate.source_key
                                    }) {
                                        list.push(PicCandidateRow {
                                            plugin_id,
                                            candidate,
                                        });
                                        candidates.set(list);
                                    }
                                } else {
                                    error.set(Some(
                                        "Enter a GitLab username or profile URL (e.g. gitlab or https://gitlab.com/gitlab)".into(),
                                    ));
                                }
                            }
                        },
                        "Add GitLab candidate"
                    }
                }
            }
            if candidates.read().is_empty() {
                p {
                    class: "hint",
                    "No candidates yet. Add emails for Gravatar, or GitHub/GitLab website links on the contact."
                }
            }
            ul {
                class: "pic-candidates",
                for row in candidates.read().iter() {
                    li {
                        class: "pic-candidate",
                        div {
                            class: "pic-candidate-main",
                            if let Some(url) = &row.candidate.preview_url {
                                img {
                                    class: "pic-preview",
                                    src: "{url}",
                                    alt: "{row.candidate.label}",
                                }
                            }
                            div {
                                strong {
                                    "{row.candidate.label}"
                                }
                                p {
                                    class: "hint",
                                    "Source: {row.plugin_id}"
                                }
                            }
                        }
                        button {
                            disabled: busy(),
                            onclick: {
                                let plugin_id = row.plugin_id.clone();
                                let candidate = row.candidate.clone();
                                let state = state.clone();
                                move |_| {
                                    let plugin_id = plugin_id.clone();
                                    let candidate = candidate.clone();
                                    let label = candidate.label.clone();
                                    busy.set(true);
                                    error.set(None);
                                    status.set(Some(format!("Fetching {label}…")));
                                    let state = state.clone();
                                    spawn(async move {
                                        let plugin = match state.plugin_registry.read().unwrap().get(&plugin_id) {
                                            Ok(plugin) => plugin,
                                            Err(err) => {
                                                error.set(Some(err.to_string()));
                                                busy.set(false);
                                                return;
                                            }
                                        };
                                        let fetch_result = plugin.fetch_pic(&candidate).await;
                                        match fetch_result {
                                            Ok(pic) => {
                                                match state
                                                    .contact_service
                                                    .apply_profile_pic(profile_id, contact_id, pic,)
                                                    .await {
                                                    Ok(updated) => {
                                                        status.set(
                                                            Some(format!("Applied profile picture from {label}"))
                                                        );
                                                        on_applied.call(updated);
                                                    },
                                                    Err(err) => {
                                                        error.set(Some(err.to_string()))
                                                    },
                                                }
                                            },
                                            Err(err) => error.set(Some(err.to_string())),
                                        }
                                        busy.set(false);
                                    });
                                }
                            },
                            "Apply"
                        }
                    }
                }
            }
            if ! contact.websites.is_empty() {
                div {
                    class: "pic-convenience",
                    h3 {
                        "Website links"
                    }
                    ul {
                        for site in contact.websites.iter() {
                            li {
                                a {
                                    href: "{site.url}",
                                    target: "_blank",
                                    "{site.label}: {site.url}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
