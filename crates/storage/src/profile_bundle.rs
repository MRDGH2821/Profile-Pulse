use crate::error::StorageError;
use crate::traits::StorageBackend;
use chrono::Utc;
use profile_pulse_core::{Profile, ProfileId, import_contacts_from_vcf};
use std::io::{Cursor, Write};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const PROFILE_FILE: &str = "profile.toml";
const AGGREGATE_VCF: &str = "aggregate.vcf";

pub async fn export_profile_bundle<B: StorageBackend>(
    storage: &B,
    profile_id: ProfileId,
) -> Result<Vec<u8>, StorageError> {
    let profile = storage.load_profile(profile_id).await?;
    let vcf = storage.export_profile_vcf_aggregate(profile_id).await?;
    let profile_toml = toml::to_string_pretty(&profile)?;
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default();
        zip.start_file(PROFILE_FILE, options)
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        zip.write_all(profile_toml.as_bytes())
            .map_err(StorageError::Io)?;
        zip.start_file(AGGREGATE_VCF, options)
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        zip.write_all(&vcf).map_err(StorageError::Io)?;
        zip.finish()
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
    }
    Ok(buffer.into_inner())
}

pub async fn import_profile_bundle<B: StorageBackend>(
    storage: &B,
    bytes: &[u8],
) -> Result<Profile, StorageError> {
    let cursor = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(cursor).map_err(|e| StorageError::Vcard(format!("invalid bundle: {e}")))?;
    let mut profile_toml = None;
    let mut aggregate_vcf = None;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        let name = file.name().trim_start_matches("./").to_string();
        let mut contents = Vec::new();
        std::io::copy(&mut file, &mut contents).map_err(StorageError::Io)?;
        if name == PROFILE_FILE {
            profile_toml = Some(
                String::from_utf8(contents)
                    .map_err(|e| StorageError::Vcard(format!("profile.toml is not utf-8: {e}")))?,
            );
        } else if name == AGGREGATE_VCF {
            aggregate_vcf = Some(contents);
        }
    }
    let profile_text =
        profile_toml.ok_or_else(|| StorageError::Vcard("bundle missing profile.toml".into()))?;
    let vcf_bytes =
        aggregate_vcf.ok_or_else(|| StorageError::Vcard("bundle missing aggregate.vcf".into()))?;
    let mut profile: Profile = toml::from_str(&profile_text)?;
    profile.id = ProfileId(Uuid::new_v4());
    profile.slug = unique_profile_slug(storage, &profile.slug).await?;
    let now = Utc::now();
    profile.created_at = now;
    profile.updated_at = now;
    storage.save_profile(&profile).await?;
    let contacts = import_contacts_from_vcf(profile.id, &vcf_bytes)?;
    for contact in contacts {
        let vcard_bytes = profile_pulse_core::contact_to_vcard_bytes(&contact)
            .map_err(|e| StorageError::Vcard(e.to_string()))?;
        storage.save_contact(&contact, &vcard_bytes).await?;
    }
    Ok(profile)
}

async fn unique_profile_slug<B: StorageBackend>(
    storage: &B,
    base: &str,
) -> Result<String, StorageError> {
    let existing: Vec<String> = storage
        .list_profiles()
        .await?
        .into_iter()
        .map(|p| p.slug)
        .collect();
    if !existing.iter().any(|slug| slug == base) {
        return Ok(base.to_string());
    }
    for i in 2..1000 {
        let candidate = format!("{base}-{i}");
        if !existing.iter().any(|slug| slug == &candidate) {
            return Ok(candidate);
        }
    }
    Err(StorageError::Vcard(
        "could not allocate unique profile slug".into(),
    ))
}
