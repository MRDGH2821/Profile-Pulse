use crate::adapter::SyncAdapter;
use crate::conflict::RemoteChange;
use crate::error::SyncError;
use crate::google::oauth::refresh_google_access_token;
use crate::secrets::SecretStore;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use profile_pulse_core::{Contact, ContactId, contact_to_vcard_bytes};
use reqwest::Client;
use serde_json::json;

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
            "names":[{
                "displayName": contact.display_name,
                "givenName": contact.given_name.clone().unwrap_or_default(),
                "familyName": contact.family_name.clone().unwrap_or_default(),
            }],
        });
        if !contact.emails.is_empty() {
            body["emailAddresses"] = json!(
                contact
                    .emails
                    .iter()
                    .map(|e| json!({
                        "value": e.address,
                        "type": e.label
                    }))
                    .collect::<Vec<_>>()
            );
        }
        if !contact.phones.is_empty() {
            body["phoneNumbers"] = json!(
                contact
                    .phones
                    .iter()
                    .map(|p| json!({
                        "value": p.number,
                        "type": p.label
                    }))
                    .collect::<Vec<_>>()
            );
        }
        if !contact.websites.is_empty() {
            body["urls"] = json!(
                contact
                    .websites
                    .iter()
                    .map(|w| json!({
                        "value": w.url,
                        "type": w.label
                    }))
                    .collect::<Vec<_>>()
            );
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
                .query(&[(
                    "updatePersonFields",
                    "names,emailAddresses,phoneNumbers,urls",
                )])
                .bearer_auth(&token)
                .json(&json!({
                    "person": body
                }))
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
        let response =
            self
                .client
                .get(
                    format!(
                        "https://people.googleapis.com/v1/{remote_id}?personFields=names,emailAddresses,phoneNumbers,urls"
                    ),
                )
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
        since: DateTime<Utc>,
    ) -> Result<Vec<RemoteChange>, SyncError> {
        let token = self.access_token().await?;
        let mut changes = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let mut request = self
                .client
                .get("https://people.googleapis.com/v1/people/me/connections")
                .query(&[
                    ("personFields", "metadata,names"),
                    ("pageSize", "100"),
                ])
                .bearer_auth(&token);
            if let Some(token) = &page_token {
                request = request.query(&[("pageToken", token)]);
            }
            let response = request
                .send()
                .await
                .map_err(|e| SyncError::Http(e.to_string()))?;
            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(SyncError::Remote(format!("Google {status}: {text}")));
            }
            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| SyncError::Http(e.to_string()))?;
            if let Some(connections) = body["connections"].as_array() {
                for person in connections {
                    let Some(remote_id) = person["resourceName"].as_str() else {
                        continue;
                    };
                    let updated_at = person["metadata"]["sources"]
                        .as_array()
                        .and_then(|sources| sources.first())
                        .and_then(|source| source["updateTime"].as_str())
                        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                        .map(|value| value.with_timezone(&Utc));
                    let Some(updated_at) = updated_at else {
                        continue;
                    };
                    if updated_at <= since {
                        continue;
                    }
                    let display_name = person["names"]
                        .as_array()
                        .and_then(|names| names.first())
                        .and_then(|name| name["displayName"].as_str())
                        .unwrap_or("Unknown contact")
                        .to_string();
                    changes.push(RemoteChange {
                        remote_id: remote_id.to_string(),
                        display_name,
                        updated_at,
                    });
                }
            }
            page_token = body["nextPageToken"]
                .as_str()
                .map(std::string::ToString::to_string);
            if page_token.is_none() {
                break;
            }
        }
        Ok(changes)
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
