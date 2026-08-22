use async_trait::async_trait;
use chrono::{DateTime, Utc};
use profile_pulse_core::{contact_to_vcard_bytes, Contact, ContactId};
use reqwest::Client;
use serde_json::json;

use crate::adapter::{RemoteChange, SyncAdapter};
use crate::error::SyncError;
use crate::google::oauth::refresh_google_access_token;
use crate::secrets::SecretStore;

pub struct GoogleContactsAdapter {
    client: Client,
    client_id: String,
    secrets: SecretStore,
    profile_id: profile_pulse_core::ProfileId,
}

impl GoogleContactsAdapter {
    pub fn new(
        secrets: SecretStore,
        profile_id: profile_pulse_core::ProfileId,
        client_id: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            client_id: client_id.into(),
            secrets,
            profile_id,
        }
    }

    async fn access_token(&self) -> Result<String, SyncError> {
        refresh_google_access_token(&self.client_id, &self.secrets, self.profile_id).await
    }

    fn person_body(contact: &Contact) -> serde_json::Value {
        let mut body = json!({
            "names": [{
                "displayName": contact.display_name,
                "givenName": contact.given_name.clone().unwrap_or_default(),
                "familyName": contact.family_name.clone().unwrap_or_default(),
            }],
        });
        if !contact.emails.is_empty() {
            body["emailAddresses"] = json!(contact
                .emails
                .iter()
                .map(|e| json!({ "value": e.address, "type": e.label }))
                .collect::<Vec<_>>());
        }
        if !contact.phones.is_empty() {
            body["phoneNumbers"] = json!(contact
                .phones
                .iter()
                .map(|p| json!({ "value": p.number, "type": p.label }))
                .collect::<Vec<_>>());
        }
        if !contact.websites.is_empty() {
            body["urls"] = json!(contact
                .websites
                .iter()
                .map(|w| json!({ "value": w.url, "type": w.label }))
                .collect::<Vec<_>>());
        }
        body
    }
}

#[async_trait]
impl SyncAdapter for GoogleContactsAdapter {
    fn target_kind(&self) -> &'static str {
        "google"
    }

    async fn push_contact(
        &self,
        contact: &Contact,
        _vcard_bytes: &[u8],
        existing_remote_id: Option<&str>,
    ) -> Result<String, SyncError> {
        let token = self.access_token().await?;
        let body = Self::person_body(contact);

        let response = if let Some(resource_name) = existing_remote_id {
            self.client
                .patch(format!(
                    "https://people.googleapis.com/v1/{resource_name}:updateContact"
                ))
                .query(&[("updatePersonFields", "names,emailAddresses,phoneNumbers,urls")])
                .bearer_auth(&token)
                .json(&json!({ "person": body }))
                .send()
                .await
                .map_err(|e| SyncError::Http(e.to_string()))?
        } else {
            self.client
                .post("https://people.googleapis.com/v1/people:createContact")
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .map_err(|e| SyncError::Http(e.to_string()))?
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SyncError::Remote(format!("Google API {status}: {text}")));
        }

        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;
        value
            .get("resourceName")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| SyncError::Remote("Google response missing resourceName".into()))
    }

    async fn pull_contact(&self, remote_id: &str) -> Result<(Contact, Vec<u8>), SyncError> {
        let token = self.access_token().await?;
        let response = self
            .client
            .get(format!(
                "https://people.googleapis.com/v1/{remote_id}?personFields=names,emailAddresses,phoneNumbers,urls"
            ))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(SyncError::Remote(format!("Google API {status}: {text}")));
        }

        let person: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SyncError::Http(e.to_string()))?;

        let display_name = person
            .pointer("/names/0/displayName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        let given_name = person
            .pointer("/names/0/givenName")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let family_name = person
            .pointer("/names/0/familyName")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        let mut emails = Vec::new();
        if let Some(list) = person.get("emailAddresses").and_then(|v| v.as_array()) {
            for item in list {
                if let Some(address) = item.get("value").and_then(|v| v.as_str()) {
                    emails.push(profile_pulse_core::EmailAddress {
                        label: item
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("other")
                            .into(),
                        address: address.into(),
                    });
                }
            }
        }

        let mut phones = Vec::new();
        if let Some(list) = person.get("phoneNumbers").and_then(|v| v.as_array()) {
            for item in list {
                if let Some(number) = item.get("value").and_then(|v| v.as_str()) {
                    phones.push(profile_pulse_core::PhoneNumber {
                        label: item
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("other")
                            .into(),
                        number: number.into(),
                    });
                }
            }
        }

        let mut websites = Vec::new();
        if let Some(list) = person.get("urls").and_then(|v| v.as_array()) {
            for item in list {
                if let Some(url) = item.get("value").and_then(|v| v.as_str()) {
                    websites.push(profile_pulse_core::WebsiteLink {
                        label: item
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("website")
                            .into(),
                        url: url.into(),
                    });
                }
            }
        }

        let contact = Contact {
            id: ContactId(uuid::Uuid::new_v4()),
            profile_id: self.profile_id,
            display_name,
            given_name,
            family_name,
            emails,
            phones,
            websites,
            photo_content_hash: None,
            updated_at: Utc::now(),
        };
        let vcard = contact_to_vcard_bytes(&contact)?;
        Ok((contact, vcard))
    }

    async fn check_remote_changes(
        &self,
        _since: DateTime<Utc>,
    ) -> Result<Vec<RemoteChange>, SyncError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn person_body_includes_display_name() {
        let contact = Contact {
            id: ContactId(uuid::Uuid::new_v4()),
            profile_id: profile_pulse_core::ProfileId(uuid::Uuid::new_v4()),
            display_name: "Ada".into(),
            given_name: Some("Ada".into()),
            family_name: Some("Lovelace".into()),
            emails: vec![],
            phones: vec![],
            websites: vec![],
            photo_content_hash: None,
            updated_at: Utc::now(),
        };
        let body = GoogleContactsAdapter::person_body(&contact);
        assert_eq!(body["names"][0]["displayName"], "Ada");
    }
}
