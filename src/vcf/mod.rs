//! VCF (vCard) import and export functionality
//!
//! This module handles parsing vCard files, extracting contact information,
//! and converting between Contact models and vCard format.

use crate::core::contact::{Contact, ContactBuilder, SocialPlatform, SocialProfile};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse a VCF file and return a list of contacts
pub fn import_from_file(path: &Path) -> Result<Vec<Contact>> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read VCF file: {}", path.display()))?;
    import_from_string(&content)
}

/// Parse VCF content from a string and return a list of contacts
pub fn import_from_string(content: &str) -> Result<Vec<Contact>> {
    let vcards = parse_vcards(content)?;
    let mut contacts = Vec::new();

    for vcard in vcards {
        match parse_vcard(vcard) {
            Ok(contact) => contacts.push(contact),
            Err(e) => {
                eprintln!("Warning: Failed to parse vCard: {}", e);
                continue;
            }
        }
    }

    Ok(contacts)
}

/// Simple vCard representation
#[derive(Debug, Clone)]
struct VCard {
    properties: Vec<VProperty>,
}

/// vCard property
#[derive(Debug, Clone)]
struct VProperty {
    name: String,
    params: HashMap<String, String>,
    value: String,
}

/// Parse multiple vCards from content
fn parse_vcards(content: &str) -> Result<Vec<VCard>> {
    let mut vcards = Vec::new();
    let mut current_properties = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        
        if line.is_empty() {
            continue;
        }

        if line == "BEGIN:VCARD" {
            current_properties.clear();
        } else if line == "END:VCARD" {
            if !current_properties.is_empty() {
                vcards.push(VCard {
                    properties: current_properties.clone(),
                });
                current_properties.clear();
            }
        } else if let Some(property) = parse_property(line) {
            current_properties.push(property);
        }
    }

    if vcards.is_empty() {
        anyhow::bail!("No valid vCards found in content");
    }

    Ok(vcards)
}

/// Parse a single property line
fn parse_property(line: &str) -> Option<VProperty> {
    // Split on first colon to separate name/params from value
    let colon_pos = line.find(':')?;
    let name_part = &line[..colon_pos];
    let value = line[colon_pos + 1..].to_string();

    // Parse name and parameters
    let (name, params) = parse_name_and_params(name_part);

    Some(VProperty {
        name,
        params,
        value,
    })
}

/// Parse property name and parameters
fn parse_name_and_params(name_part: &str) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    let parts: Vec<&str> = name_part.split(';').collect();
    
    let name = parts[0].to_uppercase();
    
    for param in &parts[1..] {
        if let Some(eq_pos) = param.find('=') {
            let key = param[..eq_pos].to_uppercase();
            let value = param[eq_pos + 1..].to_string();
            params.insert(key, value);
        } else {
            // Parameter without value (like TYPE=HOME becomes just HOME)
            params.insert(param.to_uppercase(), String::new());
        }
    }

    (name, params)
}

/// Convert a VCard to a Contact
fn parse_vcard(vcard: VCard) -> Result<Contact> {
    let name = extract_name(&vcard)?;
    let mut builder = ContactBuilder::new().name(name);

    if let Some(email) = extract_email(&vcard) {
        builder = builder.email(email);
    }

    if let Some(phone) = extract_phone(&vcard) {
        builder = builder.phone(phone);
    }

    if let Some(org) = extract_organization(&vcard) {
        builder = builder.organization(org);
    }

    if let Some(title) = extract_title(&vcard) {
        builder = builder.title(title);
    }

    if let Some(photo_url) = extract_photo_url(&vcard) {
        builder = builder.photo_url(photo_url);
    }

    let social_profiles = extract_social_profiles(&vcard);
    for profile in social_profiles {
        builder = builder.social_profile(profile);
    }

    let custom_fields = extract_custom_fields(&vcard);
    for (key, value) in custom_fields {
        builder = builder.custom_field(key, value);
    }

    builder.build().context("Failed to build contact from vCard")
}

/// Extract name from vCard
fn extract_name(vcard: &VCard) -> Result<String> {
    // Try formatted name (FN)
    for prop in &vcard.properties {
        if prop.name == "FN" && !prop.value.is_empty() {
            return Ok(prop.value.clone());
        }
    }

    // Try structured name (N)
    for prop in &vcard.properties {
        if prop.name == "N" {
            // N format: Family;Given;Additional;Prefix;Suffix
            let parts: Vec<&str> = prop.value.split(';').collect();
            let mut name_parts = Vec::new();
            
            if parts.len() > 1 && !parts[1].is_empty() {
                name_parts.push(parts[1]); // Given name
            }
            if !parts.is_empty() && !parts[0].is_empty() {
                name_parts.push(parts[0]); // Family name
            }
            
            if !name_parts.is_empty() {
                return Ok(name_parts.join(" "));
            }
        }
    }

    Ok("Unknown Contact".to_string())
}

/// Extract primary email
fn extract_email(vcard: &VCard) -> Option<String> {
    for prop in &vcard.properties {
        if prop.name == "EMAIL" && !prop.value.is_empty() {
            return Some(prop.value.clone());
        }
    }
    None
}

/// Extract primary phone
fn extract_phone(vcard: &VCard) -> Option<String> {
    for prop in &vcard.properties {
        if prop.name == "TEL" && !prop.value.is_empty() {
            return Some(prop.value.clone());
        }
    }
    None
}

/// Extract organization
fn extract_organization(vcard: &VCard) -> Option<String> {
    for prop in &vcard.properties {
        if prop.name == "ORG" {
            let org = prop.value.split(';').next()?;
            if !org.is_empty() {
                return Some(org.to_string());
            }
        }
    }
    None
}

/// Extract title
fn extract_title(vcard: &VCard) -> Option<String> {
    for prop in &vcard.properties {
        if prop.name == "TITLE" && !prop.value.is_empty() {
            return Some(prop.value.clone());
        }
    }
    None
}

/// Extract photo URL
fn extract_photo_url(vcard: &VCard) -> Option<String> {
    for prop in &vcard.properties {
        if prop.name == "PHOTO" {
            if let Some(value_type) = prop.params.get("VALUE") {
                if value_type.to_lowercase().contains("uri") {
                    return Some(prop.value.clone());
                }
            }
        }
    }
    None
}

/// Extract social profiles from URLs
fn extract_social_profiles(vcard: &VCard) -> Vec<SocialProfile> {
    let mut profiles = Vec::new();

    for prop in &vcard.properties {
        if prop.name == "URL" || prop.name == "X-SOCIALPROFILE" {
            if let Some(profile) = parse_social_url(&prop.value) {
                profiles.push(profile);
            }
        }
    }

    profiles
}

/// Parse a social media URL and extract platform and username
fn parse_social_url(url: &str) -> Option<SocialProfile> {
    let url_lower = url.to_lowercase();

    // LinkedIn
    if url_lower.contains("linkedin.com/in/") {
        if let Some(username) = extract_path_segment(url, "linkedin.com/in/") {
            return Some(create_social_profile(
                SocialPlatform::LinkedIn,
                username,
                url.to_string(),
            ));
        }
    }

    // Twitter/X
    if url_lower.contains("twitter.com/") || url_lower.contains("x.com/") {
        let domain = if url_lower.contains("twitter.com/") {
            "twitter.com/"
        } else {
            "x.com/"
        };
        if let Some(username) = extract_path_segment(url, domain) {
            return Some(create_social_profile(
                SocialPlatform::Twitter,
                username,
                url.to_string(),
            ));
        }
    }

    // Facebook
    if url_lower.contains("facebook.com/") {
        if let Some(username) = extract_path_segment(url, "facebook.com/") {
            return Some(create_social_profile(
                SocialPlatform::Facebook,
                username,
                url.to_string(),
            ));
        }
    }

    // Instagram
    if url_lower.contains("instagram.com/") {
        if let Some(username) = extract_path_segment(url, "instagram.com/") {
            return Some(create_social_profile(
                SocialPlatform::Instagram,
                username,
                url.to_string(),
            ));
        }
    }

    // GitHub
    if url_lower.contains("github.com/") {
        if let Some(username) = extract_path_segment(url, "github.com/") {
            return Some(create_social_profile(
                SocialPlatform::GitHub,
                username,
                url.to_string(),
            ));
        }
    }

    // Mastodon
    if url_lower.contains("/@") {
        if let Some(username) = url.split("/@").nth(1).and_then(|s| s.split('/').next()) {
            return Some(create_social_profile(
                SocialPlatform::Mastodon,
                username.to_string(),
                url.to_string(),
            ));
        }
    }

    None
}

/// Extract username from URL path segment
fn extract_path_segment(url: &str, after: &str) -> Option<String> {
    url.to_lowercase()
        .split(after)
        .nth(1)
        .and_then(|s| s.split('/').next())
        .and_then(|s| s.split('?').next())
        .and_then(|s| s.split('#').next())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Create a SocialProfile instance
fn create_social_profile(
    platform: SocialPlatform,
    username: String,
    url: String,
) -> SocialProfile {
    SocialProfile::new(platform, username, url)
}

/// Extract custom fields from vCard
fn extract_custom_fields(vcard: &VCard) -> HashMap<String, String> {
    let mut custom_fields = HashMap::new();

    for prop in &vcard.properties {
        if prop.name == "NOTE" && !prop.value.is_empty() {
            custom_fields.insert("NOTE".to_string(), prop.value.clone());
        }
        
        if prop.name.starts_with("X-") 
            && prop.name != "X-SOCIALPROFILE" 
            && !prop.value.is_empty() 
        {
            custom_fields.insert(prop.name.clone(), prop.value.clone());
        }
    }

    custom_fields
}

/// Export a contact to vCard format
pub fn export_contact(contact: &Contact) -> Result<String> {
    let mut lines = Vec::new();

    lines.push("BEGIN:VCARD".to_string());
    lines.push("VERSION:4.0".to_string());
    
    lines.push(format!("FN:{}", contact.name));

    if let Some((family, given)) = split_name(&contact.name) {
        lines.push(format!("N:{};{};;;", family, given));
    }

    if let Some(email) = &contact.email {
        lines.push(format!("EMAIL:{}", email));
    }

    if let Some(phone) = &contact.phone {
        lines.push(format!("TEL:{}", phone));
    }

    if let Some(org) = &contact.organization {
        lines.push(format!("ORG:{}", org));
    }

    if let Some(title) = &contact.title {
        lines.push(format!("TITLE:{}", title));
    }

    if let Some(photo_url) = &contact.photo_url {
        lines.push(format!("PHOTO;VALUE=uri:{}", photo_url));
    }

    for profile in &contact.social_profiles {
        lines.push(format!("URL:{}", profile.url));
        lines.push(format!(
            "X-SOCIALPROFILE;type={}:{}",
            profile.platform.as_str(),
            profile.url
        ));
    }

    for (key, value) in &contact.custom_fields {
        if key == "NOTE" {
            lines.push(format!("NOTE:{}", value));
        } else {
            lines.push(format!("{}:{}", key, value));
        }
    }

    lines.push(format!("UID:{}", contact.id));
    lines.push("END:VCARD".to_string());

    Ok(lines.join("\r\n") + "\r\n")
}

/// Export multiple contacts to a VCF file
pub fn export_to_file(contacts: &[Contact], path: &Path) -> Result<()> {
    let mut vcf_content = String::new();

    for contact in contacts {
        let vcard_str = export_contact(contact)?;
        vcf_content.push_str(&vcard_str);
    }

    fs::write(path, vcf_content)
        .context(format!("Failed to write VCF file: {}", path.display()))?;

    Ok(())
}

/// Split a full name into family and given names
fn split_name(name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.len() >= 2 {
        let given = parts[0].to_string();
        let family = parts[1..].join(" ");
        Some((family, given))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_linkedin_url() {
        let url = "https://www.linkedin.com/in/johndoe/";
        let profile = parse_social_url(url).unwrap();
        assert_eq!(profile.platform, SocialPlatform::LinkedIn);
        assert_eq!(profile.username, "johndoe");
    }

    #[test]
    fn test_parse_twitter_url() {
        let url = "https://twitter.com/johndoe";
        let profile = parse_social_url(url).unwrap();
        assert_eq!(profile.platform, SocialPlatform::Twitter);
        assert_eq!(profile.username, "johndoe");
    }

    #[test]
    fn test_parse_github_url() {
        let url = "https://github.com/johndoe";
        let profile = parse_social_url(url).unwrap();
        assert_eq!(profile.platform, SocialPlatform::GitHub);
        assert_eq!(profile.username, "johndoe");
    }

    #[test]
    fn test_split_name() {
        let (family, given) = split_name("John Doe").unwrap();
        assert_eq!(given, "John");
        assert_eq!(family, "Doe");
    }

    #[test]
    fn test_import_from_string() {
        let vcf = "BEGIN:VCARD\r
VERSION:4.0\r
FN:John Doe\r
EMAIL:john@example.com\r
TEL:+1-555-1234\r
ORG:Example Corp\r
TITLE:Software Engineer\r
URL:https://github.com/johndoe\r
END:VCARD\r\n";

        let contacts = import_from_string(vcf).unwrap();
        assert_eq!(contacts.len(), 1);

        let contact = &contacts[0];
        assert_eq!(contact.name, "John Doe");
        assert_eq!(contact.email.as_ref().unwrap(), "john@example.com");
        assert_eq!(contact.phone.as_ref().unwrap(), "+1-555-1234");
        assert_eq!(contact.organization.as_ref().unwrap(), "Example Corp");
        assert_eq!(contact.title.as_ref().unwrap(), "Software Engineer");
        assert_eq!(contact.social_profiles.len(), 1);
        assert_eq!(contact.social_profiles[0].platform, SocialPlatform::GitHub);
    }

    #[test]
    fn test_export_contact() {
        let mut contact = Contact::new("Jane Smith");
        contact.email = Some("jane@example.com".to_string());
        contact.phone = Some("+1-555-5678".to_string());
        contact.organization = Some("Tech Co".to_string());

        let vcf = export_contact(&contact).unwrap();

        assert!(vcf.contains("FN:Jane Smith"));
        assert!(vcf.contains("EMAIL:jane@example.com"));
        assert!(vcf.contains("TEL:+1-555-5678"));
        assert!(vcf.contains("ORG:Tech Co"));
    }
}