use crate::error::CoreError;
use crate::model::{Contact, ContactId, EmailAddress, PhoneNumber, ProfileId, WebsiteLink};
use chrono::Utc;

const PHOTO_HASH_PROP: &str = "X-PROFILE-PULSE-PHOTO-HASH";

pub fn contact_to_vcard_bytes(contact: &Contact) -> Result<Vec<u8>, CoreError> {
    let mut lines = vec![
        "BEGIN:VCARD".to_string(),
        "VERSION:3.0".to_string(),
        format!("UID:{}", contact.id.0),
        format!("FN:{}", escape_text(&contact.display_name)),
    ];
    if let Some(given) = &contact.given_name {
        lines.push(format!("N:;{};;;", escape_text(given)));
    }
    if let Some(family) = &contact.family_name
        && contact.given_name.is_none()
    {
        lines.push(format!("N:{};;;;", escape_text(family)));
    }
    for email in &contact.emails {
        let label = escape_param(&email.label);
        lines.push(format!(
            "EMAIL;TYPE={label}:{}",
            escape_text(&email.address)
        ));
    }
    for phone in &contact.phones {
        let label = escape_param(&phone.label);
        lines.push(format!("TEL;TYPE={label}:{}", escape_text(&phone.number)));
    }
    for site in &contact.websites {
        let label = escape_param(&site.label);
        lines.push(format!("URL;TYPE={label}:{}", escape_text(&site.url)));
    }
    if let Some(hash) = &contact.photo_content_hash {
        lines.push(format!("{PHOTO_HASH_PROP}:{}", escape_text(hash)));
    }
    lines.push("END:VCARD".to_string());
    Ok(lines.join("\r\n").into_bytes())
}

pub fn contact_from_vcard_bytes(
    profile_id: ProfileId,
    contact_id: ContactId,
    bytes: &[u8],
) -> Result<Contact, CoreError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| CoreError::Validation(format!("vcard is not utf-8: {e}")))?;
    let mut display_name = String::new();
    let mut given_name = None;
    let mut family_name = None;
    let mut emails = Vec::new();
    let mut phones = Vec::new();
    let mut websites = Vec::new();
    let mut photo_content_hash = None;
    let mut uid_from_vcard = None;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("BEGIN:") || line.starts_with("END:") {
            continue;
        }
        let (key, params, value) = parse_line(line)?;
        match key {
            "FN" => display_name = unescape_text(value),
            "N" => {
                let parts: Vec<&str> = value.split(';').collect();
                if parts.len() > 1 && !parts[1].is_empty() {
                    given_name = Some(unescape_text(parts[1]));
                }
                if !parts[0].is_empty() {
                    family_name = Some(unescape_text(parts[0]));
                }
            }
            "EMAIL" => emails.push(EmailAddress {
                label: params
                    .get("TYPE")
                    .cloned()
                    .unwrap_or_else(|| "other".into()),
                address: unescape_text(value),
            }),
            "TEL" => phones.push(PhoneNumber {
                label: params
                    .get("TYPE")
                    .cloned()
                    .unwrap_or_else(|| "other".into()),
                number: unescape_text(value),
            }),
            "URL" => websites.push(WebsiteLink {
                label: params
                    .get("TYPE")
                    .cloned()
                    .unwrap_or_else(|| "website".into()),
                url: unescape_text(value),
            }),
            PHOTO_HASH_PROP => photo_content_hash = Some(unescape_text(value)),
            "UID" => {
                if let Ok(uuid) = uuid::Uuid::parse_str(value.trim()) {
                    uid_from_vcard = Some(ContactId(uuid));
                }
            }
            _ => {}
        }
    }
    if display_name.is_empty() {
        return Err(CoreError::Validation("vcard missing FN".into()));
    }
    Ok(Contact {
        id: uid_from_vcard.unwrap_or(contact_id),
        profile_id,
        display_name,
        given_name,
        family_name,
        emails,
        phones,
        websites,
        photo_content_hash,
        updated_at: Utc::now(),
    })
}

/// Split a VCF file or string into individual vCard byte blocks.
pub fn split_vcard_blocks(bytes: &[u8]) -> Result<Vec<Vec<u8>>, CoreError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|e| CoreError::Validation(format!("vcf is not utf-8: {e}")))?;
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_card = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.eq_ignore_ascii_case("BEGIN:VCARD") {
            in_card = true;
            current.clear();
            current.push("BEGIN:VCARD".to_string());
        } else if line.eq_ignore_ascii_case("END:VCARD") {
            if in_card {
                current.push("END:VCARD".to_string());
                blocks.push(current.join("\r\n").into_bytes());
                current.clear();
                in_card = false;
            }
        } else if in_card && !line.is_empty() {
            current.push(raw_line.to_string());
        }
    }
    if blocks.is_empty() {
        return Err(CoreError::Validation("no vCards found in import".into()));
    }
    Ok(blocks)
}

/// Parse all contacts from a multi-contact VCF payload.
pub fn import_contacts_from_vcf(
    profile_id: ProfileId,
    bytes: &[u8],
) -> Result<Vec<Contact>, CoreError> {
    let blocks = split_vcard_blocks(bytes)?;
    let mut contacts = Vec::new();
    for block in blocks {
        let fallback_id = ContactId(uuid::Uuid::new_v4());
        match contact_from_vcard_bytes(profile_id, fallback_id, &block) {
            Ok(contact) => contacts.push(contact),
            Err(_) => continue,
        }
    }
    if contacts.is_empty() {
        return Err(CoreError::Validation(
            "no valid contacts found in vcf import".into(),
        ));
    }
    Ok(contacts)
}

fn parse_line(
    line: &str,
) -> Result<(&str, std::collections::HashMap<String, String>, &str), CoreError> {
    let (head, value) = line
        .split_once(':')
        .ok_or_else(|| CoreError::Validation(format!("invalid vcard line: {line}")))?;
    let mut parts = head.split(';');
    let key = parts
        .next()
        .ok_or_else(|| CoreError::Validation(format!("invalid vcard line: {line}")))?;
    let mut params = std::collections::HashMap::new();
    for part in parts {
        if let Some((k, v)) = part.split_once('=') {
            params.insert(k.to_ascii_uppercase(), unescape_text(v));
        }
    }
    Ok((key, params, value))
}

fn escape_text(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace(',', "\\,")
        .replace(';', "\\;")
}

fn escape_param(input: &str) -> String {
    input.replace(',', "\\,")
}

fn unescape_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some(c) => out.push(c),
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn vcard_round_trip_preserves_display_name_and_email() {
        let profile_id = ProfileId(Uuid::new_v4());
        let contact_id = ContactId(Uuid::new_v4());
        let original = Contact {
            id: contact_id,
            profile_id,
            display_name: "Test User".into(),
            given_name: None,
            family_name: None,
            emails: vec![EmailAddress {
                label: "home".into(),
                address: "test@example.com".into(),
            }],
            phones: vec![],
            websites: vec![],
            photo_content_hash: None,
            updated_at: Utc::now(),
        };
        let bytes = contact_to_vcard_bytes(&original).unwrap();
        let parsed = contact_from_vcard_bytes(profile_id, contact_id, &bytes).unwrap();
        assert_eq!(parsed.display_name, "Test User");
        assert_eq!(parsed.emails[0].address, "test@example.com");
    }
}
