use chrono::{DateTime, Utc};
use profile_pulse_core::{Contact, ContactId, PullConflictResolution};

/// A contact that changed on a remote sync target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChange {
    pub remote_id: String,
    pub display_name: String,
    pub updated_at: DateTime<Utc>,
}

/// Remote edits reported by a single sync target since the last poll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetRemoteChanges {
    pub target_kind: String,
    pub changes: Vec<RemoteChange>,
}

/// Local and remote versions disagree after the last successful link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullConflict {
    pub contact_id: ContactId,
    pub target_kind: String,
    pub remote_id: String,
    pub local: Contact,
    pub remote: Contact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullPrepareResult {
    /// Safe to apply the remote contact.
    Apply(Box<Contact>),
    /// User must choose a resolution strategy.
    Conflict(Box<PullConflict>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullApplyResult {
    Applied,
    KeptLocal,
    DeferredReview,
}

pub fn is_pull_conflict(
    local: &Contact,
    remote: &Contact,
    link_updated_at: DateTime<Utc>,
) -> bool {
    local.updated_at > link_updated_at && remote.updated_at > link_updated_at
}

pub fn resolve_pull_conflict(
    conflict: &PullConflict,
    resolution: PullConflictResolution,
) -> Result<Contact, &'static str> {
    match resolution {
        PullConflictResolution::KeepLocal => Ok(conflict.local.clone()),
        PullConflictResolution::TakeRemote => Ok(conflict.remote.clone()),
        PullConflictResolution::Review => Err("review in editor"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use profile_pulse_core::{EmailAddress, ProfileId};

    fn sample_contact(id: ContactId, updated_at: DateTime<Utc>) -> Contact {
        Contact {
            id,
            profile_id: ProfileId(uuid::Uuid::new_v4()),
            display_name: "Ada".into(),
            given_name: Some("Ada".into()),
            family_name: None,
            emails: vec![EmailAddress {
                label: "work".into(),
                address: "ada@example.com".into(),
            }],
            phones: vec![],
            websites: vec![],
            photo_content_hash: None,
            updated_at,
        }
    }

    #[test]
    fn detects_conflict_when_both_sides_changed_after_link() {
        let link_time = Utc::now() - chrono::Duration::hours(2);
        let local = sample_contact(ContactId(uuid::Uuid::new_v4()), link_time + chrono::Duration::hours(1));
        let remote = sample_contact(local.id, link_time + chrono::Duration::minutes(30));
        assert!(is_pull_conflict(&local, &remote, link_time));
    }

    #[test]
    fn no_conflict_when_only_remote_changed() {
        let link_time = Utc::now() - chrono::Duration::hours(1);
        let local = sample_contact(ContactId(uuid::Uuid::new_v4()), link_time - chrono::Duration::hours(1));
        let remote = sample_contact(local.id, link_time + chrono::Duration::minutes(10));
        assert!(!is_pull_conflict(&local, &remote, link_time));
    }
}
