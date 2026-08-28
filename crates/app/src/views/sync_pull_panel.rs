#[cfg(not(target_arch = "wasm32"))]
use crate::routes::Route;
#[cfg(not(target_arch = "wasm32"))]
use crate::state::AppState;
#[cfg(not(target_arch = "wasm32"))]
use crate::sync_prompt::SyncPromptState;
#[cfg(not(target_arch = "wasm32"))]
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use profile_pulse_core::{ContactId, ProfileId};
#[cfg(not(target_arch = "wasm32"))]
use profile_pulse_storage::StorageBackend;
#[cfg(not(target_arch = "wasm32"))]
use profile_pulse_sync::PullPrepareResult;

#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn SyncPullPanel(
    selected_profile: ProfileId,
    mut busy: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut status: Signal<Option<String>>,
) -> Element {
    let state = use_context::<AppState>();
    let sync_prompt = use_context::<SyncPromptState>();
    let mut sync_prompt = sync_prompt;
    let mut sections = Vec::new();

    if let Some((prompt_profile_id, changes)) = sync_prompt.pending_snapshot() {
        if selected_profile == prompt_profile_id {
            sections.push(rsx! {
                div {
                    class: "sync-prompt-panel",
                    h3 {
                        "Remote changes ready to pull"
                    }
                    ul {
                        for target in changes.iter() {
                            li {
                                "{target.target_kind}: {target.changes.len()} contact(s)"
                            }
                        }
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            let mut sync_prompt = sync_prompt;
                            move |_| {
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                let changes = sync_prompt
                                    .pending_snapshot()
                                    .map(|(_, c)| c)
                                    .unwrap_or_default();
                                spawn(async move {
                                    let mut sync_prompt = sync_prompt;
                                    let Ok(profile) = state.storage.load_profile(prompt_profile_id).await
                                    else {
                                        busy.set(false);
                                        return;
                                    };
                                    let mut applied = 0u32;
                                    let mut conflict_count = 0u32;
                                    for target in &changes {
                                        for change in &target.changes {
                                            let contact_id = match state.sync_service.links().find_contact_by_remote_id(
                                                prompt_profile_id,
                                                &target.target_kind,
                                                &change.remote_id,
                                            ) {
                                                Ok(Some(id)) => id,
                                                Ok(None) => ContactId(uuid::Uuid::new_v4()),
                                                Err(err) => {
                                                    error.set(Some(err.to_string()));
                                                    continue;
                                                }
                                            };
                                            let local = state
                                                .storage
                                                .load_contact(prompt_profile_id, contact_id)
                                                .await
                                                .ok();
                                            match state
                                                .sync_service
                                                .prepare_pull_item(
                                                    &profile,
                                                    &target.target_kind,
                                                    change,
                                                    contact_id,
                                                    local.as_ref(),
                                                )
                                                .await
                                            {
                                                Ok(PullPrepareResult::Apply(remote)) => {
                                                    if let Ok(contact) = state
                                                        .sync_service
                                                        .apply_pull_item(
                                                            &profile,
                                                            &target.target_kind,
                                                            &change.remote_id,
                                                            remote,
                                                        )
                                                        .await
                                                    {
                                                        if let Err(err) = state
                                                            .contact_service
                                                            .update_contact(contact)
                                                            .await
                                                        {
                                                            error.set(Some(err.to_string()));
                                                        } else {
                                                            applied += 1;
                                                        }
                                                    }
                                                }
                                                Ok(PullPrepareResult::Conflict(conflict)) => {
                                                    sync_prompt.add_conflict(conflict);
                                                    conflict_count += 1;
                                                }
                                                Err(err) => error.set(Some(err.to_string())),
                                            }
                                        }
                                    }
                                    sync_prompt.clear_pending();
                                    status.set(Some(format!(
                                        "Pulled {applied} contact(s); {conflict_count} conflict(s) need review"
                                    )));
                                    busy.set(false);
                                });
                            }
                        },
                        "Pull remote changes"
                    }
                }
            });
        }
    }

    if !sync_prompt.conflicts_snapshot().is_empty() {
        sections.push(rsx! {
            div {
                class: "sync-conflicts-panel",
                h3 {
                    "Unresolved pull conflicts"
                }
                ul {
                    for conflict in sync_prompt.conflicts_snapshot().iter() {
                        li {
                            "{conflict.local.display_name} ({conflict.target_kind})"
                            Link {
                                to: Route::ContactDetail {
                                    profile_id: conflict.local.profile_id.0.to_string(),
                                    contact_id: conflict.contact_id.0.to_string(),
                                },
                                " Review"
                            }
                        }
                    }
                }
            }
        });
    }

    rsx! {
        {sections.into_iter()}
    }
}

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn SyncPullPanel(
    _selected_profile: profile_pulse_core::ProfileId,
    _busy: Signal<bool>,
    _error: Signal<Option<String>>,
    _status: Signal<Option<String>>,
) -> Element {
    rsx! {}
}
