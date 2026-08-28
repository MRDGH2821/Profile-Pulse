#[cfg(not(target_arch = "wasm32"))]
use crate::state::AppState;
#[cfg(not(target_arch = "wasm32"))]
use crate::sync_prompt::SyncPromptState;
#[cfg(not(target_arch = "wasm32"))]
use dioxus::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use profile_pulse_core::{ContactId, ProfileId, PullConflictResolution};
#[cfg(not(target_arch = "wasm32"))]
use profile_pulse_storage::StorageBackend;
#[cfg(not(target_arch = "wasm32"))]
use profile_pulse_sync::PullApplyResult;

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::redundant_locals)]
#[component]
pub fn ContactPullConflictPanel(
    profile_uuid: ProfileId,
    contact_uuid: ContactId,
    mut contact: Signal<Option<profile_pulse_core::Contact>>,
    on_open_editor: EventHandler<()>,
    mut error: Signal<Option<String>>,
    mut conflict_status: Signal<Option<String>>,
) -> Element {
    let app_state = use_context::<AppState>();
    let sync_prompt = use_context::<SyncPromptState>();
    let Some(conflict) = sync_prompt.conflict_for(contact_uuid) else {
        return rsx! {};
    };
    rsx! {
        div {
            class: "sync-conflict-panel",
            h3 {
                "Pull conflict"
            }
            p {
                class: "hint",
                "Local and remote versions both changed since the last sync link. Choose how to resolve."
            }
            div {
                class: "toolbar",
                button {
                    onclick: {
                        let state = app_state.clone();
                        let conflict = conflict.clone();
                        move |_| {
                            let state = state.clone();
                            let conflict = conflict.clone();
                            let mut sync_prompt = sync_prompt;
                            spawn(async move {
                                let Ok(profile) = state.storage.load_profile(profile_uuid).await else {
                                    return;
                                };
                                match state
                                    .sync_service
                                    .pull_with_resolution(
                                        &profile,
                                        &conflict,
                                        PullConflictResolution::KeepLocal,
                                    )
                                    .await
                                {
                                    Ok((_, PullApplyResult::KeptLocal)) => {
                                        sync_prompt.remove_conflict(contact_uuid);
                                        conflict_status.set(Some("Kept local version".into()));
                                    }
                                    Ok(_) => conflict_status.set(Some("Resolution applied".into())),
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                            });
                        }
                    },
                    "Keep local"
                }
                button {
                    onclick: {
                        let state = app_state.clone();
                        let conflict = conflict.clone();
                        move |_| {
                            let state = state.clone();
                            let conflict = conflict.clone();
                            let mut sync_prompt = sync_prompt;
                            spawn(async move {
                                let Ok(profile) = state.storage.load_profile(profile_uuid).await else {
                                    return;
                                };
                                match state
                                    .sync_service
                                    .pull_with_resolution(
                                        &profile,
                                        &conflict,
                                        PullConflictResolution::TakeRemote,
                                    )
                                    .await
                                {
                                    Ok((updated, PullApplyResult::Applied)) => {
                                        if let Err(err) = state
                                            .contact_service
                                            .update_contact(updated.clone())
                                            .await
                                        {
                                            error.set(Some(err.to_string()));
                                        } else {
                                            contact.set(Some(updated));
                                            sync_prompt.remove_conflict(contact_uuid);
                                            conflict_status
                                                .set(Some("Applied remote version".into()));
                                        }
                                    }
                                    Ok(_) => conflict_status.set(Some("Resolution applied".into())),
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                            });
                        }
                    },
                    "Take remote"
                }
                button {
                    onclick: move |_| {
                        contact.set(Some(conflict.remote.clone()));
                        on_open_editor.call(());
                        conflict_status.set(Some(
                            "Remote version loaded in editor for manual review".into(),
                        ));
                    },
                    "Review in editor"
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
#[component]
pub fn ContactPullConflictPanel(
    _profile_uuid: profile_pulse_core::ProfileId,
    _contact_uuid: profile_pulse_core::ContactId,
    _contact: Signal<Option<profile_pulse_core::Contact>>,
    _on_open_editor: EventHandler<()>,
    _error: Signal<Option<String>>,
    _conflict_status: Signal<Option<String>>,
) -> Element {
    rsx! {}
}
