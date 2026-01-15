//! VCF (vCard) import and export functionality
//!
//! This module handles parsing vCard files, extracting contact information,
//! and converting between Contact models and vCard format.

use crate::core::contact::{
    Contact, ContactAddress, ContactBuilder, ContactDate, ContactEmail, ContactPhone, ContactUrl,
    SocialPlatform, SocialProfile,
};
use crate::core::labels::{AddressLabel, DateLabel, EmailLabel, PhoneLabel};
use chrono::NaiveDate;
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
    /// For itemN.* properties, stores the item group name
    item_group: Option<String>,
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
    let value = unescape_vcf_value(&line[colon_pos + 1..]);

    // Check for itemN. prefix (e.g., "item1.URL")
    let (item_group, actual_name_part) = if name_part.to_lowercase().starts_with("item") {
        if let Some(dot_pos) = name_part.find('.') {
            let group = name_part[..dot_pos].to_string();
            let rest = &name_part[dot_pos + 1..];
            (Some(group), rest)
        } else {
            (None, name_part)
        }
    } else {
        (None, name_part)
    };

    // Parse name and parameters
    let (name, params) = parse_name_and_params(actual_name_part);

    Some(VProperty {
        name,
        params,
        value,
        item_group,
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
            
            // Handle multiple values for the same key (e.g., TYPE=HOME;TYPE=FAX)
            if let Some(existing) = params.get(&key) {
                let combined = format!("{};{}", existing, value);
                params.insert(key, combined);
            } else {
                params.insert(key, value);
            }
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

    // Extract structured fields
    let emails = extract_emails(&vcard);
    for email in emails {
        builder = builder.email_entry(email);
    }

    let phones = extract_phones(&vcard);
    for phone in phones {
        builder = builder.phone_entry(phone);
    }

    let addresses = extract_addresses(&vcard);
    for address in addresses {
        builder = builder.address(address);
    }

    let dates = extract_dates(&vcard);
    for date in dates {
        builder = builder.date(date);
    }

    let urls = extract_urls(&vcard);
    for url in urls {
        builder = builder.url(url);
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

/// Extract all email addresses with their labels
fn extract_emails(vcard: &VCard) -> Vec<ContactEmail> {
    let mut emails = Vec::new();
    let mut item_labels: HashMap<String, String> = HashMap::new();

    // First pass: collect all itemN.X-ABLabel entries
    for prop in &vcard.properties {
        if prop.name == "X-ABLABEL" {
            if let Some(ref group) = prop.item_group {
                item_labels.insert(group.to_lowercase(), prop.value.clone());
            }
        }
    }

    // Second pass: collect all EMAIL entries and associate labels
    for prop in &vcard.properties {
        if prop.name == "EMAIL" {
            let email_value = prop.value.trim();
            if email_value.is_empty() {
                continue;
            }

            // Determine label
            let label = if let Some(ref group) = prop.item_group {
                // Try to get label from itemN.X-ABLabel
                item_labels.get(&group.to_lowercase()).cloned()
            } else {
                // Try to get label from TYPE parameter
                prop.params.get("TYPE").cloned()
            };

            let label_str = label
                .map(|l| EmailLabel::from_str(&l).to_string_value())
                .unwrap_or_else(|| EmailLabel::default().to_string_value());

            emails.push(ContactEmail::new(email_value.to_string(), label_str));
        }
    }

    emails
}

/// Extract all phone numbers with their labels
fn extract_phones(vcard: &VCard) -> Vec<ContactPhone> {
    let mut phones = Vec::new();
    let mut item_labels: HashMap<String, String> = HashMap::new();

    // First pass: collect all itemN.X-ABLabel entries
    for prop in &vcard.properties {
        if prop.name == "X-ABLABEL" {
            if let Some(ref group) = prop.item_group {
                item_labels.insert(group.to_lowercase(), prop.value.clone());
            }
        }
    }

    // Second pass: collect all TEL entries and associate labels
    for prop in &vcard.properties {
        if prop.name == "TEL" {
            let phone_value = prop.value.trim();
            if phone_value.is_empty() {
                continue;
            }

            // Determine label from itemN.X-ABLabel or TYPE parameter
            let label = if let Some(ref group) = prop.item_group {
                item_labels.get(&group.to_lowercase()).cloned()
            } else {
                // Get TYPE parameter (may contain multiple values separated by semicolons)
                prop.params.get("TYPE").cloned()
            };

            let label_str = label
                .map(|l| PhoneLabel::from_str(&l).to_string_value())
                .unwrap_or_else(|| PhoneLabel::default().to_string_value());

            phones.push(ContactPhone::new(phone_value.to_string(), label_str));
        }
    }

    phones
}

/// Extract all addresses with their labels
fn extract_addresses(vcard: &VCard) -> Vec<ContactAddress> {
    let mut addresses = Vec::new();
    let mut item_labels: HashMap<String, String> = HashMap::new();

    // First pass: collect all itemN.X-ABLabel entries
    for prop in &vcard.properties {
        if prop.name == "X-ABLABEL" {
            if let Some(ref group) = prop.item_group {
                item_labels.insert(group.to_lowercase(), prop.value.clone());
            }
        }
    }

    // Second pass: collect all ADR entries and associate labels
    for prop in &vcard.properties {
        if prop.name == "ADR" {
            // ADR format: PO Box;Extended;Street;City;State;Postal Code;Country
            let parts: Vec<&str> = prop.value.split(';').collect();

            // Determine label
            let label = if let Some(ref group) = prop.item_group {
                item_labels.get(&group.to_lowercase()).cloned()
            } else {
                prop.params.get("TYPE").cloned()
            };

            let label_str = label
                .map(|l| AddressLabel::from_str(&l).to_string_value())
                .unwrap_or_else(|| AddressLabel::default().to_string_value());

            let mut address = ContactAddress::new(&label_str);

            // Parse address components
            if parts.len() > 2 && !parts[2].is_empty() {
                address.street = Some(parts[2].to_string());
            }
            if parts.len() > 3 && !parts[3].is_empty() {
                address.city = Some(parts[3].to_string());
            }
            if parts.len() > 4 && !parts[4].is_empty() {
                address.state = Some(parts[4].to_string());
            }
            if parts.len() > 5 && !parts[5].is_empty() {
                address.postal_code = Some(parts[5].to_string());
            }
            if parts.len() > 6 && !parts[6].is_empty() {
                address.country = Some(parts[6].to_string());
            }

            // Only add if address has at least one field
            if !address.is_empty() {
                addresses.push(address);
            }
        }
    }

    addresses
}

/// Extract all significant dates with their labels
fn extract_dates(vcard: &VCard) -> Vec<ContactDate> {
    let mut dates = Vec::new();
    let mut item_labels: HashMap<String, String> = HashMap::new();

    // First pass: collect all itemN.X-ABLabel entries
    for prop in &vcard.properties {
        if prop.name == "X-ABLABEL" {
            if let Some(ref group) = prop.item_group {
                item_labels.insert(group.to_lowercase(), prop.value.clone());
            }
        }
    }

    // Second pass: collect all date entries
    for prop in &vcard.properties {
        let (date_value, default_label) = if prop.name == "BDAY" {
            (prop.value.trim(), "Birthday")
        } else if prop.name == "X-ABDATE" {
            (prop.value.trim(), "Other")
        } else {
            continue;
        };

        if date_value.is_empty() {
            continue;
        }

        // Determine label
        let label = if let Some(ref group) = prop.item_group {
            item_labels
                .get(&group.to_lowercase())
                .map(|l| DateLabel::from_str(l).to_string_value())
        } else {
            None
        };

        let label_str = label.unwrap_or_else(|| default_label.to_string());

        // Parse date (try YYYYMMDD format first, then ISO format)
        if let Ok(contact_date) = ContactDate::from_yyyymmdd(date_value, &label_str) {
            dates.push(contact_date);
        } else if let Ok(naive_date) = NaiveDate::parse_from_str(date_value, "%Y-%m-%d") {
            dates.push(ContactDate::new(naive_date, label_str));
        }
    }

    dates
}

/// Extract all URLs with their labels
fn extract_urls(vcard: &VCard) -> Vec<ContactUrl> {
    let mut urls = Vec::new();
    let mut item_labels: HashMap<String, String> = HashMap::new();

    // First pass: collect all itemN.X-ABLabel entries
    for prop in &vcard.properties {
        if prop.name == "X-ABLABEL" {
            if let Some(ref group) = prop.item_group {
                item_labels.insert(group.to_lowercase(), prop.value.clone());
            }
        }
    }

    // Second pass: collect all URL entries and associate labels
    for prop in &vcard.properties {
        if prop.name == "URL" || prop.name == "X-SOCIALPROFILE" {
            let url_value = prop.value.trim();
            if url_value.is_empty() {
                continue;
            }

            // Determine label
            let label = if let Some(ref group) = prop.item_group {
                // Try to get label from itemN.X-ABLabel
                item_labels.get(&group.to_lowercase()).cloned()
            } else {
                // Try to get label from TYPE parameter
                prop.params.get("TYPE").cloned()
            };

            urls.push(ContactUrl::new(url_value.to_string(), label));
        }
    }

    urls
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
        
        if prop.name == "NICKNAME" && !prop.value.is_empty() {
            custom_fields.insert("NICKNAME".to_string(), prop.value.clone());
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

    // Export structured emails with proper item numbering
    let mut email_item_counter = 1000;
    for contact_email in &contact.emails {
        let email_label = EmailLabel::from_str(&contact_email.label);
        match email_label {
            EmailLabel::Home => {
                lines.push(format!("EMAIL;TYPE=INTERNET;TYPE=HOME:{}", contact_email.email));
            }
            EmailLabel::Work => {
                lines.push(format!("EMAIL;TYPE=INTERNET;TYPE=WORK:{}", contact_email.email));
            }
            EmailLabel::Other => {
                lines.push(format!("EMAIL;TYPE=INTERNET;TYPE=OTHER:{}", contact_email.email));
            }
            EmailLabel::Custom(_) => {
                // Use itemN.X-ABLabel for custom labels
                let item_name = format!("item{}", email_item_counter);
                email_item_counter += 1;
                lines.push(format!("{}.EMAIL;TYPE=INTERNET:{}", item_name, contact_email.email));
                lines.push(format!("{}.X-ABLabel:{}", item_name, contact_email.label));
            }
        }
    }

    // Export structured phones with proper item numbering
    let mut phone_item_counter = 2000;
    for contact_phone in &contact.phones {
        let phone_label = PhoneLabel::from_str(&contact_phone.label);
        match phone_label {
            PhoneLabel::Home | PhoneLabel::Work | PhoneLabel::Mobile | PhoneLabel::Main
            | PhoneLabel::HomeFax | PhoneLabel::WorkFax | PhoneLabel::Pager | PhoneLabel::Other => {
                let vcard_type = phone_label.to_vcard_type();
                lines.push(format!("TEL;TYPE={}:{}", vcard_type, contact_phone.phone));
            }
            PhoneLabel::Custom(_) => {
                // Use itemN.X-ABLabel for custom labels
                let item_name = format!("item{}", phone_item_counter);
                phone_item_counter += 1;
                lines.push(format!("{}.TEL:{}", item_name, contact_phone.phone));
                lines.push(format!("{}.X-ABLabel:{}", item_name, contact_phone.label));
            }
        }
    }

    // Export structured addresses with proper item numbering
    let mut address_item_counter = 3000;
    for contact_address in &contact.addresses {
        let address_label = AddressLabel::from_str(&contact_address.label);
        
        // Format: PO Box;Extended;Street;City;State;Postal Code;Country
        let adr_value = format!(
            ";;{};{};{};{};{}",
            contact_address.street.as_deref().unwrap_or(""),
            contact_address.city.as_deref().unwrap_or(""),
            contact_address.state.as_deref().unwrap_or(""),
            contact_address.postal_code.as_deref().unwrap_or(""),
            contact_address.country.as_deref().unwrap_or("")
        );

        match address_label {
            AddressLabel::Home => {
                lines.push(format!("ADR;TYPE=HOME:{}", adr_value));
            }
            AddressLabel::Work => {
                lines.push(format!("ADR;TYPE=WORK:{}", adr_value));
            }
            AddressLabel::Other => {
                lines.push(format!("ADR;TYPE=OTHER:{}", adr_value));
            }
            AddressLabel::Custom(_) => {
                let item_name = format!("item{}", address_item_counter);
                address_item_counter += 1;
                lines.push(format!("{}.ADR:{}", item_name, adr_value));
                lines.push(format!("{}.X-ABLabel:{}", item_name, contact_address.label));
            }
        }
    }

    // Export structured dates with proper item numbering
    let mut date_item_counter = 4000;
    for contact_date in &contact.dates {
        let date_label = DateLabel::from_str(&contact_date.label);
        let date_str = contact_date.to_yyyymmdd();

        match date_label {
            DateLabel::Birthday => {
                lines.push(format!("BDAY:{}", date_str));
            }
            DateLabel::Anniversary => {
                // Use Apple format for Anniversary
                let item_name = format!("item{}", date_item_counter);
                date_item_counter += 1;
                lines.push(format!("{}.X-ABDATE:{}", item_name, date_str));
                lines.push(format!("{}.X-ABLabel:{}", item_name, date_label.to_apple_format()));
            }
            DateLabel::Other | DateLabel::Custom(_) => {
                let item_name = format!("item{}", date_item_counter);
                date_item_counter += 1;
                lines.push(format!("{}.X-ABDATE:{}", item_name, date_str));
                lines.push(format!("{}.X-ABLabel:{}", item_name, contact_date.label));
            }
        }
    }

    // Export URLs with labels using itemN format
    for (idx, contact_url) in contact.urls.iter().enumerate() {
        let item_name = format!("item{}", 5000 + idx);
        
        lines.push(format!("{}.URL:{}", item_name, contact_url.url));
        
        if let Some(ref label) = contact_url.label {
            lines.push(format!("{}.X-ABLabel:{}", item_name, label));
        }
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

/// Unescape VCF value (handle \: \; \, \\ \n)
fn unescape_vcf_value(value: &str) -> String {
    value
        .replace("\\:", ":")
        .replace("\\;", ";")
        .replace("\\,", ",")
        .replace("\\n", "\n")
        .replace("\\N", "\n")
        .replace("\\\\", "\\")
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
        assert_eq!(contact.urls.len(), 1);
        assert!(contact.urls[0].url.contains("github.com"));
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

    #[test]
    fn test_parse_url_with_label() {
        let vcf = "BEGIN:VCARD\r
VERSION:4.0\r
FN:John Doe\r
item1.URL:https://github.com/johndoe\r
item1.X-ABLabel:GitHub\r
item2.URL:https://myblog.com\r
item2.X-ABLabel:Blog\r
END:VCARD\r\n";

        let contacts = import_from_string(vcf).unwrap();
        assert_eq!(contacts.len(), 1);

        let contact = &contacts[0];
        assert_eq!(contact.urls.len(), 2);

        let github_urls = contact.find_urls_by_label("GitHub");
        assert_eq!(github_urls.len(), 1);
        assert_eq!(github_urls[0].url, "https://github.com/johndoe");

        let blog_urls = contact.find_urls_by_label("Blog");
        assert_eq!(blog_urls.len(), 1);
        assert_eq!(blog_urls[0].url, "https://myblog.com");
    }

    #[test]
    fn test_export_urls_with_labels() {
        let mut contact = Contact::new("John Doe");
        contact.add_url(ContactUrl::new("https://github.com/johndoe", Some("GitHub".to_string())));
        contact.add_url(ContactUrl::new("https://myblog.com", Some("Blog".to_string())));

        let vcf = export_contact(&contact).unwrap();

        assert!(vcf.contains("item5000.URL:https://github.com/johndoe"));
        assert!(vcf.contains("item5000.X-ABLabel:GitHub"));
        assert!(vcf.contains("item5001.URL:https://myblog.com"));
        assert!(vcf.contains("item5001.X-ABLabel:Blog"));
    }

    #[test]
    fn test_import_google_contacts_vcf() {
        // Test with the actual Google Contacts VCF sample
        let vcf = r#"BEGIN:VCARD
VERSION:3.0
FN:Prefix First name Middle name Surname Suffix
N:Surname;First name;Middle name;Prefix;Suffix
NICKNAME:Nickname
X-PHONETIC-FIRST-NAME:Phonetic first
X-PHONETIC-MIDDLE-NAME:Phonetic middle
X-PHONETIC-LAST-NAME:Phonetic last
X-FILE-AS:File as
EMAIL;TYPE=INTERNET;TYPE=HOME:home@email.com
EMAIL;TYPE=INTERNET;TYPE=WORK:work@email.com
item1.EMAIL;TYPE=INTERNET:custom@email.com
item1.X-ABLabel:Custom
TEL;TYPE=HOME:+91 99999 99999
TEL;TYPE=WORK:+1 444-444-4444
TEL:+213 22 22 22 22
TEL;TYPE=CELL:+54 11 2222-2222
TEL;TYPE=MAIN:+1 555-555-5555
TEL;TYPE=HOME;TYPE=FAX:+682 55 555
TEL;TYPE=WORK;TYPE=FAX:+679 555 5555
item2.TEL:+6905999
item2.X-ABLabel:googleVoice
TEL;TYPE=PAGER:+965 555 55555
ORG:Company;Department
TITLE:Job title
BDAY:19700101
item3.URL:https\://profile.com
item3.X-ABLabel:PROFILE
item4.URL:https\://blog.com
item4.X-ABLabel:BLOG
item5.URL:https\://homepage.com
item5.X-ABLabel:_$!<HomePage>!$_
URL;TYPE=WORK:https\://work.com
item6.URL:https\://github.com
item6.X-ABLabel:GitHub
item7.URL:https\://instagram.com
item7.X-ABLabel:Instagram
item8.X-ABDATE:20000101
item8.X-ABLabel:_$!<Anniversary>!$_
X-ABDATE:20100101
NOTE:Notes sample
CATEGORIES:myContacts
END:VCARD"#;

        let contacts = import_from_string(vcf).unwrap();
        assert_eq!(contacts.len(), 1);

        let contact = &contacts[0];
        
        // Check name
        assert_eq!(contact.name, "Prefix First name Middle name Surname Suffix");
        
        // Check organization and title
        assert_eq!(contact.organization.as_ref().unwrap(), "Company");
        assert_eq!(contact.title.as_ref().unwrap(), "Job title");
        
        // Check that we have URLs
        assert!(contact.urls.len() > 0, "Should have extracted URLs");
        
        // Check for specific labeled URLs
        let github_urls = contact.find_urls_by_label("GitHub");
        assert_eq!(github_urls.len(), 1, "Should have one GitHub URL");
        assert_eq!(github_urls[0].url, "https://github.com");
        
        let instagram_urls = contact.find_urls_by_label("Instagram");
        assert_eq!(instagram_urls.len(), 1, "Should have one Instagram URL");
        assert_eq!(instagram_urls[0].url, "https://instagram.com");
        
        let blog_urls = contact.find_urls_by_label("BLOG");
        assert_eq!(blog_urls.len(), 1, "Should have one Blog URL");
        assert_eq!(blog_urls[0].url, "https://blog.com");
        
        let profile_urls = contact.find_urls_by_label("PROFILE");
        assert_eq!(profile_urls.len(), 1, "Should have one Profile URL");
        assert_eq!(profile_urls[0].url, "https://profile.com");
        
        let homepage_urls = contact.find_urls_by_label("_$!<HomePage>!$_");
        assert_eq!(homepage_urls.len(), 1, "Should have one HomePage URL");
        assert_eq!(homepage_urls[0].url, "https://homepage.com");
        
        let work_urls = contact.find_urls_by_label("WORK");
        assert_eq!(work_urls.len(), 1, "Should have one Work URL");
        assert_eq!(work_urls[0].url, "https://work.com");
        
        // Check custom fields contain NOTE
        assert!(contact.custom_fields.contains_key("NOTE"));
        assert_eq!(contact.custom_fields.get("NOTE").unwrap(), "Notes sample");
        
        // Check custom fields contain NICKNAME
        assert!(contact.custom_fields.contains_key("NICKNAME"));
        assert_eq!(contact.custom_fields.get("NICKNAME").unwrap(), "Nickname");
        
        // Check structured emails
        assert_eq!(contact.emails.len(), 3, "Should have extracted 3 emails");
        assert!(contact.emails.iter().any(|e| e.email == "home@email.com" && e.label == "Home"));
        assert!(contact.emails.iter().any(|e| e.email == "work@email.com" && e.label == "Work"));
        assert!(contact.emails.iter().any(|e| e.email == "custom@email.com" && e.label == "Custom"));
        
        // Check structured phones
        assert_eq!(contact.phones.len(), 9, "Should have extracted 9 phones");
        assert!(contact.phones.iter().any(|p| p.phone == "+91 99999 99999" && p.label == "Home"));
        assert!(contact.phones.iter().any(|p| p.phone == "+1 444-444-4444" && p.label == "Work"));
        assert!(contact.phones.iter().any(|p| p.phone == "+54 11 2222-2222" && p.label == "Mobile"));
        assert!(contact.phones.iter().any(|p| p.phone == "+1 555-555-5555" && p.label == "Main"));
        // Note: The VCF has "TEL;TYPE=HOME;TYPE=FAX:+682 55 555" which parses to "HOME;FAX"
        // PhoneLabel::from_str("HOME;FAX") returns HomeFax with label "Home Fax"
        let has_home_fax = contact.phones.iter().any(|p| p.phone == "+682 55 555" && p.label == "Home Fax");
        assert!(has_home_fax, "Should have home fax phone");
        assert!(contact.phones.iter().any(|p| p.phone == "+679 555 5555" && p.label == "Work Fax"));
        assert!(contact.phones.iter().any(|p| p.phone == "+6905999" && p.label == "Google Voice"));
        assert!(contact.phones.iter().any(|p| p.phone == "+965 555 55555" && p.label == "Pager"));
        
        // Check structured dates (there are more X-ABDATE entries in the test VCF)
        assert!(contact.dates.len() >= 2, "Should have extracted at least 2 dates");
        assert!(contact.dates.iter().any(|d| d.label == "Birthday" && d.date.format("%Y%m%d").to_string() == "19700101"));
        assert!(contact.dates.iter().any(|d| d.label == "Anniversary" && d.date.format("%Y%m%d").to_string() == "20000101"));
    }

    #[test]
    fn test_export_structured_emails() {
        let mut contact = Contact::new("Test User");
        contact.emails.push(ContactEmail::new("home@example.com", "Home"));
        contact.emails.push(ContactEmail::new("work@example.com", "Work"));
        contact.emails.push(ContactEmail::new("custom@example.com", "Personal"));

        let vcf = export_contact(&contact).unwrap();

        assert!(vcf.contains("EMAIL;TYPE=INTERNET;TYPE=HOME:home@example.com"));
        assert!(vcf.contains("EMAIL;TYPE=INTERNET;TYPE=WORK:work@example.com"));
        assert!(vcf.contains("item1000.EMAIL;TYPE=INTERNET:custom@example.com"));
        assert!(vcf.contains("item1000.X-ABLabel:Personal"));
    }

    #[test]
    fn test_export_structured_phones() {
        let mut contact = Contact::new("Test User");
        contact.phones.push(ContactPhone::new("+1234567890", "Mobile"));
        contact.phones.push(ContactPhone::new("+9876543210", "Home"));
        contact.phones.push(ContactPhone::new("+1111111111", "Google Voice"));

        let vcf = export_contact(&contact).unwrap();

        assert!(vcf.contains("TEL;TYPE=CELL:+1234567890"));
        assert!(vcf.contains("TEL;TYPE=HOME:+9876543210"));
        assert!(vcf.contains("item2000.TEL:+1111111111"));
        assert!(vcf.contains("item2000.X-ABLabel:Google Voice"));
    }

    #[test]
    fn test_export_structured_addresses() {
        let mut contact = Contact::new("Test User");
        contact.addresses.push(
            ContactAddress::builder()
                .street("123 Main St")
                .city("Springfield")
                .state("IL")
                .postal_code("62701")
                .country("USA")
                .label("Home")
                .build()
        );
        contact.addresses.push(
            ContactAddress::builder()
                .street("456 Office Ave")
                .city("Chicago")
                .state("IL")
                .postal_code("60601")
                .country("USA")
                .label("Work")
                .build()
        );

        let vcf = export_contact(&contact).unwrap();

        assert!(vcf.contains("ADR;TYPE=HOME:;;123 Main St;Springfield;IL;62701;USA"));
        assert!(vcf.contains("ADR;TYPE=WORK:;;456 Office Ave;Chicago;IL;60601;USA"));
    }

    #[test]
    fn test_export_structured_dates() {
        use chrono::NaiveDate;
        
        let mut contact = Contact::new("Test User");
        contact.dates.push(ContactDate::new(
            NaiveDate::from_ymd_opt(1990, 5, 15).unwrap(),
            "Birthday"
        ));
        contact.dates.push(ContactDate::new(
            NaiveDate::from_ymd_opt(2010, 6, 20).unwrap(),
            "Anniversary"
        ));

        let vcf = export_contact(&contact).unwrap();

        assert!(vcf.contains("BDAY:19900515"));
        assert!(vcf.contains("item4000.X-ABDATE:20100620"));
        assert!(vcf.contains("item4000.X-ABLabel:_$!<Anniversary>!$_"));
    }

    #[test]
    fn test_import_export_roundtrip() {
        // Create a contact with all structured fields
        use chrono::NaiveDate;
        
        let mut contact = Contact::new("Jane Smith");
        contact.emails.push(ContactEmail::new("jane@home.com", "Home"));
        contact.phones.push(ContactPhone::new("+1234567890", "Mobile"));
        contact.addresses.push(
            ContactAddress::builder()
                .street("789 Test St")
                .city("Testville")
                .state("TX")
                .label("Home")
                .build()
        );
        contact.dates.push(ContactDate::new(
            NaiveDate::from_ymd_opt(1985, 3, 10).unwrap(),
            "Birthday"
        ));
        contact.urls.push(ContactUrl::new("https://github.com/janesmith", Some("GitHub".to_string())));

        // Export to VCF
        let vcf = export_contact(&contact).unwrap();

        // Import back
        let contacts = import_from_string(&vcf).unwrap();
        assert_eq!(contacts.len(), 1);

        let imported = &contacts[0];
        
        // Verify all fields survived the round trip
        assert_eq!(imported.name, "Jane Smith");
        assert_eq!(imported.emails.len(), 1);
        assert_eq!(imported.emails[0].email, "jane@home.com");
        assert_eq!(imported.phones.len(), 1);
        assert_eq!(imported.phones[0].phone, "+1234567890");
        assert_eq!(imported.addresses.len(), 1);
        assert_eq!(imported.addresses[0].city, Some("Testville".to_string()));
        assert_eq!(imported.dates.len(), 1);
        assert_eq!(imported.dates[0].label, "Birthday");
        assert_eq!(imported.urls.len(), 1);
        assert!(imported.urls[0].label.as_ref().unwrap().contains("GitHub"));
    }
}