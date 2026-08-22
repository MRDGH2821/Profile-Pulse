use crate::routes::Route;
use crate::state::{ActiveProfile, AppState};
use dioxus::prelude::*;
use profile_pulse_core::{BackupRef, Profile, ProfileId};
use profile_pulse_storage::StorageBackend;

#[component]
pub fn BackupsSettings() -> Element {
    let state = use_context::<AppState>();
    let active_profile = use_context::<ActiveProfile>();
    let nav = navigator();

    let mut profiles = use_signal(Vec::<Profile>::new);
    let mut selected_profile = use_signal(|| None::<ProfileId>);
    let mut backup_dir = use_signal(String::new);
    let mut scheduled_enabled = use_signal(|| false);
    let mut backups = use_signal(Vec::<BackupRef>::new);
    let mut error = use_signal(|| None::<String>);
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    use_effect({
        let state = state.clone();
        let active_profile = active_profile;
        move || {
            let state = state.clone();
            spawn(async move {
                match state.list_profiles().await {
                    Ok(list) => {
                        profiles.set(list.clone());
                        if selected_profile().is_none() {
                            let pick = active_profile
                                .id()
                                .or_else(|| list.first().map(|p| p.id));
                            selected_profile.set(pick);
                        }
                    }
                    Err(err) => error.set(Some(err)),
                }
            });
        }
    });

    use_effect({
        let state = state.clone();
        move || {
        let Some(profile_id) = selected_profile() else {
            return;
        };
        let state = state.clone();
        spawn(async move {
            if let Ok(profile) = state.storage.load_profile(profile_id).await {
                scheduled_enabled.set(profile.settings.scheduled_backup_enabled);
                backup_dir.set(
                    profile
                        .settings
                        .scheduled_backup_dir
                        .clone()
                        .unwrap_or_default(),
                );
            }
            match state.contact_service.list_backups(profile_id).await {
                Ok(list) => backups.set(list),
                Err(err) => error.set(Some(err.to_string())),
            }
        });
        }
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
                h2 { "Backups & import/export" }
            }

            if let Some(message) = error() {
                p { class: "error", "{message}" }
            }
            if let Some(message) = status() {
                p { class: "hint", "{message}" }
            }

            label { "Profile"
                select {
                    class: "profile-select",
                    onchange: move |event| {
                        if let Ok(uuid) = uuid::Uuid::parse_str(&event.value()) {
                            selected_profile.set(Some(ProfileId(uuid)));
                        }
                    },
                    for profile in profiles.read().iter() {
                        option {
                            value: "{profile.id}",
                            selected: selected_profile() == Some(profile.id),
                            "{profile.name}"
                        }
                    }
                }
            }

            if let Some(profile_id) = selected_profile() {
                div { class: "pic-convenience",
                    h3 { "Scheduled backup" }
                    label {
                        input {
                            r#type: "checkbox",
                            checked: scheduled_enabled(),
                            onchange: move |event| scheduled_enabled.set(event.checked()),
                        }
                        " Enable scheduled export on app start"
                    }
                    label { "Backup directory"
                        input {
                            r#type: "text",
                            value: "{backup_dir}",
                            placeholder: "/path/to/backups",
                            oninput: move |event| backup_dir.set(event.value()),
                        }
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    let mut profile = match state.storage.load_profile(profile_id).await {
                                        Ok(p) => p,
                                        Err(err) => {
                                            error.set(Some(err.to_string()));
                                            busy.set(false);
                                            return;
                                        }
                                    };
                                    profile.settings.scheduled_backup_enabled = scheduled_enabled();
                                    let dir = backup_dir().trim().to_string();
                                    profile.settings.scheduled_backup_dir = if dir.is_empty() {
                                        None
                                    } else {
                                        Some(dir)
                                    };
                                    match state
                                        .contact_service
                                        .update_profile_settings(profile)
                                        .await
                                    {
                                        Ok(_) => status.set(Some("Backup settings saved".into())),
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Save backup settings"
                    }
                }

                div { class: "toolbar",
                    button {
                        disabled: busy(),
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                let Some(path) = rfd::FileDialog::new()
                                    .add_filter("vCard", &["vcf"])
                                    .pick_file()
                                else {
                                    return;
                                };
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    match std::fs::read(&path) {
                                        Ok(bytes) => {
                                            match state
                                                .contact_service
                                                .import_vcf(profile_id, &bytes)
                                                .await
                                            {
                                                Ok(ids) => status.set(Some(format!(
                                                    "Imported {} contact(s)",
                                                    ids.len()
                                                ))),
                                                Err(err) => error.set(Some(err.to_string())),
                                            }
                                        }
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Import VCF…"
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Profile Pulse bundle", &["pp-profile", "zip"])
                                    .pick_file()
                                else {
                                    return;
                                };
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    match std::fs::read(&path) {
                                        Ok(bytes) => {
                                            match state
                                                .contact_service
                                                .import_profile_bundle(&bytes)
                                                .await
                                            {
                                                Ok(profile) => {
                                                    status.set(Some(format!(
                                                        "Imported profile \"{}\"",
                                                        profile.name
                                                    )));
                                                    if let Ok(list) = state.list_profiles().await {
                                                        profiles.set(list);
                                                    }
                                                }
                                                Err(err) => error.set(Some(err.to_string())),
                                            }
                                        }
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Import profile bundle…"
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    match state
                                        .contact_service
                                        .export_vcf_aggregate(profile_id)
                                        .await
                                    {
                                        Ok(bytes) => {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_file_name("contacts.vcf")
                                                .add_filter("vCard", &["vcf"])
                                                .save_file()
                                            {
                                                let write_result = std::fs::write(&path, bytes);
                                                match write_result {
                                                    Ok(()) => status.set(Some(
                                                        "Exported aggregate VCF".into(),
                                                    )),
                                                    Err(err) => error.set(Some(err.to_string())),
                                                }
                                            }
                                        }
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Export VCF…"
                    }
                    button {
                        disabled: busy(),
                        onclick: {
                            let state = state.clone();
                            move |_| {
                                busy.set(true);
                                error.set(None);
                                let state = state.clone();
                                spawn(async move {
                                    match state
                                        .contact_service
                                        .export_profile_bundle(profile_id)
                                        .await
                                    {
                                        Ok(bytes) => {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_file_name("profile.pp-profile")
                                                .add_filter(
                                                    "Profile Pulse bundle",
                                                    &["pp-profile", "zip"],
                                                )
                                                .save_file()
                                            {
                                                let write_result = std::fs::write(&path, bytes);
                                                match write_result {
                                                    Ok(()) => status.set(Some(
                                                        "Exported profile bundle".into(),
                                                    )),
                                                    Err(err) => error.set(Some(err.to_string())),
                                                }
                                            }
                                        }
                                        Err(err) => error.set(Some(err.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        "Export profile bundle…"
                    }
                }

                h3 { "Pre-write backup snapshots" }
                p { class: "hint",
                    "Automatic snapshots created before contact edits. Restore replaces live profile data."
                }
                ul { class: "profile-list",
                    for backup in backups.read().iter() {
                        li { class: "pic-candidate",
                            span { "{backup.label}" }
                            button {
                                disabled: busy(),
                                onclick: {
                                    let label = backup.label.clone();
                                    let state = state.clone();
                                    move |_| {
                                        busy.set(true);
                                        error.set(None);
                                        let state = state.clone();
                                        let label = label.clone();
                                        spawn(async move {
                                            match state
                                                .contact_service
                                                .restore_backup(profile_id, &label)
                                                .await
                                            {
                                                Ok(()) => {
                                                    status.set(Some(format!(
                                                        "Restored backup {label}"
                                                    )));
                                                    match state
                                                        .contact_service
                                                        .list_backups(profile_id)
                                                        .await
                                                    {
                                                        Ok(list) => backups.set(list),
                                                        Err(err) => error.set(Some(err.to_string())),
                                                    }
                                                }
                                                Err(err) => error.set(Some(err.to_string())),
                                            }
                                            busy.set(false);
                                        });
                                    }
                                },
                                "Restore"
                            }
                        }
                    }
                }
            } else {
                p { class: "hint", "Create a profile to manage backups." }
            }
        }
    }
}
