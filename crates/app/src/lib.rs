//! Profile Pulse application shell.

mod routes;
mod state;
mod views;

use routes::Route;
use state::{ActiveProfile, AppState};

use dioxus::prelude::*;

pub fn launch() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let active_profile = use_signal(|| None::<profile_pulse_core::ProfileId>);
    use_context_provider(|| ActiveProfile(active_profile));
    use_context_provider(AppState::initialize);
    let active_profile = use_context::<ActiveProfile>();
    let state = use_context::<AppState>();

    use_effect(move || {
        let state = state.clone();
        spawn(async move {
            let _ = state.contact_service.run_scheduled_backups().await;
        });
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/styles.css") }
        div { class: "app-root",
            header { class: "app-header",
                h1 { "Profile Pulse" }
                nav { class: "header-nav",
                    Link { to: Route::Profiles {}, "Profiles" }
                    Link { to: Route::PicSourcesSettings {}, "Pic sources" }
                    Link { to: Route::SyncSettings {}, "Sync" }
                    Link { to: Route::BackupsSettings {}, "Backups" }
                }
                if let Some(profile_id) = active_profile.id() {
                    span { class: "active-profile", "Profile: {profile_id}" }
                }
            }
            main { class: "app-main",
                Router::<Route> {}
            }
        }
    }
}
