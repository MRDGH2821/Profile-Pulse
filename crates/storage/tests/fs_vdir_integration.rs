use chrono::Utc;
use profile_pulse_core::{
    contact_to_vcard_bytes, Contact, ContactId, EmailAddress, Profile, ProfileId, ProfileSettings,
};
use profile_pulse_storage::{ContactIndex, ContactService, FsVdirBackend, SqliteContactIndex, StorageBackend};
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn save_and_load_contact_round_trip() {
    let dir = tempdir().unwrap();
    let backend = FsVdirBackend::new(dir.path().to_path_buf());
    let profile_id = ProfileId(Uuid::new_v4());
    let contact_id = ContactId(Uuid::new_v4());

    backend
        .save_profile(&Profile {
            id: profile_id,
            name: "Test".into(),
            slug: "test".into(),
            settings: ProfileSettings {
                scheduled_backup_enabled: false,
                scheduled_backup_dir: None,
                scheduled_backup_last_run: None,
            },
            sync_targets: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .unwrap();

    let contact = Contact {
        id: contact_id,
        profile_id,
        display_name: "Jane".into(),
        given_name: None,
        family_name: None,
        emails: vec![EmailAddress {
            label: "home".into(),
            address: "jane@example.com".into(),
        }],
        phones: vec![],
        websites: vec![],
        photo_content_hash: None,
        updated_at: Utc::now(),
    };
    let vcard = contact_to_vcard_bytes(&contact).unwrap();
    backend.save_contact(&contact, &vcard).await.unwrap();

    let loaded = backend.load_contact(profile_id, contact_id).await.unwrap();
    assert_eq!(loaded.display_name, "Jane");
    assert!(dir.path().join("profiles/test/contacts").exists());
}

#[tokio::test]
async fn index_finds_contact_by_display_name() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let backend = Arc::new(FsVdirBackend::new(root.clone()));
    let index = Arc::new(SqliteContactIndex::new(root.join("index.sqlite")).unwrap());
    let service = ContactService::new(backend.clone(), index.clone(), root.clone());

    let profile_id = ProfileId(Uuid::new_v4());
    backend
        .save_profile(&Profile {
            id: profile_id,
            name: "Search".into(),
            slug: "search".into(),
            settings: ProfileSettings {
                scheduled_backup_enabled: false,
                scheduled_backup_dir: None,
                scheduled_backup_last_run: None,
            },
            sync_targets: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .unwrap();

    let contact = Contact {
        id: ContactId(Uuid::new_v4()),
        profile_id,
        display_name: "Unique Name XYZ".into(),
        given_name: None,
        family_name: None,
        emails: vec![],
        phones: vec![],
        websites: vec![],
        photo_content_hash: None,
        updated_at: Utc::now(),
    };
    service.update_contact(contact).await.unwrap();

    let ids = index.search(profile_id, "Unique Name", 10).await.unwrap();
    assert_eq!(ids.len(), 1);
}

#[tokio::test]
async fn import_vcf_and_export_bundle_round_trip() {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let backend = Arc::new(FsVdirBackend::new(root.clone()));
    let index = Arc::new(SqliteContactIndex::new(root.join("index.sqlite")).unwrap());
    let service = ContactService::new(backend.clone(), index.clone(), root.clone());

    let profile_id = ProfileId(Uuid::new_v4());
    backend
        .save_profile(&Profile {
            id: profile_id,
            name: "Backup".into(),
            slug: "backup".into(),
            settings: ProfileSettings {
                scheduled_backup_enabled: false,
                scheduled_backup_dir: None,
                scheduled_backup_last_run: None,
            },
            sync_targets: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
        .await
        .unwrap();

    let vcf = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:3.0\r\n",
        "FN:Import Me\r\n",
        "EMAIL;TYPE=home:import@example.com\r\n",
        "END:VCARD\r\n"
    );
    let imported = service.import_vcf(profile_id, vcf.as_bytes()).await.unwrap();
    assert_eq!(imported.len(), 1);

    let aggregate = service.export_vcf_aggregate(profile_id).await.unwrap();
    assert!(String::from_utf8_lossy(&aggregate).contains("Import Me"));

    let bundle = service.export_profile_bundle(profile_id).await.unwrap();
    let imported_profile = service.import_profile_bundle(&bundle).await.unwrap();
    assert_eq!(imported_profile.name, "Backup");
}
