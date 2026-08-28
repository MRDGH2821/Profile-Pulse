use crate::state::AppState;
use dioxus::prelude::*;
use profile_pulse_core::{Profile, ProfileId};

#[cfg(not(target_arch = "wasm32"))]
#[component]
pub fn BackupsImportExportToolbar(
    profile_id: ProfileId,
    busy: bool,
    mut busy_signal: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut status: Signal<Option<String>>,
    mut profiles: Signal<Vec<Profile>>,
) -> Element {
    let state = use_context::<AppState>();
    rsx! {
        div {
            class: "toolbar",
            button {
                disabled: busy,
                onclick: {
                    let state = state.clone();
                    move |_| {
                        let Some(path) =
                            rfd::FileDialog::new().add_filter("vCard", &["vcf"]).pick_file()
                        else {
                            return;
                        };
                        busy_signal.set(true);
                        error.set(None);
                        let state = state.clone();
                        spawn(async move {
                            match std::fs::read(&path) {
                                Ok(bytes) => {
                                    match state.contact_service.import_vcf(profile_id, &bytes).await {
                                        Ok(ids) => status.set(Some(format!(
                                            "Imported {} contact(s)",
                                            ids.len()
                                        ))),
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                },
                                Err(err) => error.set(Some(err.to_string())),
                            }
                            busy_signal.set(false);
                        });
                    }
                },
                "Import VCF…"
            }
            button {
                disabled: busy,
                onclick: {
                    let state = state.clone();
                    move |_| {
                        let Some(path) = rfd::FileDialog::new()
                            .add_filter("Profile Pulse bundle", &["pp-profile", "zip"])
                            .pick_file()
                        else {
                            return;
                        };
                        busy_signal.set(true);
                        error.set(None);
                        let state = state.clone();
                        spawn(async move {
                            match std::fs::read(&path) {
                                Ok(bytes) => {
                                    match state.contact_service.import_profile_bundle(&bytes).await {
                                        Ok(profile) => {
                                            status.set(Some(format!(
                                                "Imported profile \"{}\"",
                                                profile.name
                                            )));
                                            if let Ok(list) = state.list_profiles().await {
                                                profiles.set(list);
                                            }
                                        },
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                },
                                Err(err) => error.set(Some(err.to_string())),
                            }
                            busy_signal.set(false);
                        });
                    }
                },
                "Import profile bundle…"
            }
            button {
                disabled: busy,
                onclick: {
                    let state = state.clone();
                    move |_| {
                        busy_signal.set(true);
                        error.set(None);
                        let state = state.clone();
                        spawn(async move {
                            match state.contact_service.export_vcf_aggregate(profile_id).await {
                                Ok(bytes) => {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name("contacts.vcf")
                                        .add_filter("vCard", &["vcf"])
                                        .save_file()
                                    {
                                        let write_result = std::fs::write(&path, bytes);
                                        match write_result {
                                            Ok(()) => status.set(Some("Exported aggregate VCF".into())),
                                            Err(err) => error.set(Some(err.to_string())),
                                        }
                                    }
                                },
                                Err(err) => error.set(Some(err.to_string())),
                            }
                            busy_signal.set(false);
                        });
                    }
                },
                "Export VCF…"
            }
            button {
                disabled: busy,
                onclick: {
                    let state = state.clone();
                    move |_| {
                        busy_signal.set(true);
                        error.set(None);
                        let state = state.clone();
                        spawn(async move {
                            match state.contact_service.export_profile_bundle(profile_id).await {
                                Ok(bytes) => {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_file_name("profile.pp-profile")
                                        .add_filter("Profile Pulse bundle", &["pp-profile", "zip"])
                                        .save_file()
                                    {
                                        let write_result = std::fs::write(&path, bytes);
                                        match write_result {
                                            Ok(()) => {
                                                status.set(Some("Exported profile bundle".into()))
                                            },
                                            Err(err) => error.set(Some(err.to_string())),
                                        }
                                    }
                                },
                                Err(err) => error.set(Some(err.to_string())),
                            }
                            busy_signal.set(false);
                        });
                    }
                },
                "Export profile bundle…"
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn BackupsImportExportToolbar(
    _profile_id: ProfileId,
    _busy: bool,
    _busy_signal: Signal<bool>,
    _error: Signal<Option<String>>,
    _status: Signal<Option<String>>,
    _profiles: Signal<Vec<Profile>>,
) -> Element {
    rsx! {
        p {
            class: "hint",
            "File import and export uses the browser download flow in a later web build."
        }
    }
}
