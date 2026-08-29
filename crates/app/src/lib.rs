//! Profile Pulse application shell.
mod routes;
#[cfg(not(target_arch = "wasm32"))]
mod sync_prompt;
mod state;
mod views;

use dioxus::prelude::*;
use routes::Route;
use state::{ActiveProfile, AppState};
use profile_pulse_storage::StorageBackend;
#[cfg(not(target_arch = "wasm32"))]
use sync_prompt::SyncPromptState;

pub fn launch() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let active_profile = use_signal(|| None::<profile_pulse_core::ProfileId>);
    use_context_provider(|| ActiveProfile(active_profile));
    use_context_provider(AppState::initialize);
    #[cfg(not(target_arch = "wasm32"))]
    let sync_pending =
        use_signal(|| None::<(profile_pulse_core::ProfileId, Vec<profile_pulse_sync::TargetRemoteChanges>)>);
    #[cfg(not(target_arch = "wasm32"))]
    let sync_conflicts = use_signal(Vec::<profile_pulse_sync::PullConflict>::new);
    #[cfg(not(target_arch = "wasm32"))]
    use_context_provider(|| SyncPromptState {
        pending: sync_pending,
        conflicts: sync_conflicts,
    });
    let active_profile = use_context::<ActiveProfile>();
    let state = use_context::<AppState>();
    #[cfg(not(target_arch = "wasm32"))]
    let sync_prompt = use_context::<SyncPromptState>();
    #[cfg(not(target_arch = "wasm32"))]
    {
        let state_for_backups = state.clone();
        use_effect(move || {
            let state = state_for_backups.clone();
            spawn(async move {
                let _ = state.contact_service.run_scheduled_backups().await;
            });
        });
        let state_for_poll = state.clone();
        use_effect(move || {
            let state = state_for_poll.clone();
            let mut sync_prompt = sync_prompt;
            spawn(async move {
            loop {
                let Some(profile_id) = active_profile.id() else {
                    tokio::time::sleep(std::time::Duration::from_secs(15 * 60)).await;
                    continue;
                };
                if let Ok(mut profile) = state.storage.load_profile(profile_id).await {
                    match state.sync_service.poll_remote_changes(&profile).await {
                        Ok(changes) if !changes.is_empty() => {
                            sync_prompt.set_pending(profile_id, changes);
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                    profile.settings.last_remote_sync_check = Some(chrono::Utc::now());
                    let _ = state
                        .contact_service
                        .update_profile_settings(profile)
                        .await;
                }
                tokio::time::sleep(std::time::Duration::from_secs(15 * 60)).await;
            }
        });
        });
    }
    #[cfg(target_arch = "wasm32")]
    use_effect(move || {
        let state = state.clone();
        spawn(async move {
            let _ = state.contact_service.run_scheduled_backups().await;
        });
    });
    rsx! {
        document::Stylesheet { href: asset!("/assets/styles.css") }
        Router::<Route> {}
    }
}

#[component]
pub fn AppShell() -> Element {
    let active_profile = use_context::<ActiveProfile>();
    rsx! {
        div {
            class: "app-root",
            header {
                class: "app-header",
                SyncPromptBanner {}
                h1 {
                    "Profile Pulse"
                }
                nav {
                    class: "header-nav",
                    Link {
                        to: Route::Profiles {},
                        "Profiles"
                    }
                    Link {
                        to: Route::PicSourcesSettings {},
                        "Pic sources"
                    }
                    Link {
                        to: Route::SyncSettings {},
                        "Sync"
                    }
                    Link {
                        to: Route::BackupsSettings {},
                        "Backups"
                    }
                }
                if let Some(profile_id) = active_profile.id() {
                    span {
                        class: "active-profile",
                        "Profile: {profile_id}"
                    }
                }
            }
            main {
                class: "app-main",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn SyncPromptBanner() -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        return rsx! {};
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let sync_prompt = use_context::<SyncPromptState>();
        let nav = navigator();
        let Some((profile_id, changes)) = sync_prompt.pending_snapshot() else {
            return rsx! {};
        };
        let summary = changes
            .iter()
            .map(|target| format!("{} ({})", target.target_kind, target.changes.len()))
            .collect::<Vec<_>>()
            .join(", ");
        return rsx! {
            div {
                class: "sync-prompt-banner",
                span {
                    "Remote changes detected: {summary}"
                }
                button {
                    class: "link-button",
                    onclick: move |_| {
                        let _ = nav.push(Route::SyncSettings {});
                        let _ = profile_id;
                    },
                    "Review in Sync settings"
                }
                button {
                    class: "link-button",
                    onclick: {
                        let mut sync_prompt = sync_prompt;
                        move |_| sync_prompt.clear_pending()
                    },
                    "Dismiss"
                }
            }
        };
    }
}
