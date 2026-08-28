//! Desktop-only sync polling prompt and pending pull conflicts.
use dioxus::prelude::*;
use profile_pulse_core::{ContactId, ProfileId};
use profile_pulse_sync::{PullConflict, TargetRemoteChanges};

#[derive(Clone, Copy)]
pub struct SyncPromptState {
    pub pending: Signal<Option<(ProfileId, Vec<TargetRemoteChanges>)>>,
    pub conflicts: Signal<Vec<PullConflict>>,
}

impl SyncPromptState {
    pub fn new() -> Self {
        Self {
            pending: use_signal(|| None),
            conflicts: use_signal(Vec::new),
        }
    }

    pub fn set_pending(&mut self, profile_id: ProfileId, changes: Vec<TargetRemoteChanges>) {
        self.pending.set(Some((profile_id, changes)));
    }

    pub fn clear_pending(&mut self) {
        self.pending.set(None);
    }

    pub fn add_conflict(&mut self, conflict: PullConflict) {
        self.conflicts.write().push(conflict);
    }

    pub fn remove_conflict(&mut self, contact_id: ContactId) {
        self.conflicts
            .write()
            .retain(|c| c.contact_id != contact_id);
    }

    pub fn pending_snapshot(&self) -> Option<(ProfileId, Vec<TargetRemoteChanges>)> {
        (self.pending)()
    }

    pub fn conflicts_snapshot(&self) -> Vec<PullConflict> {
        (self.conflicts)()
    }

    pub fn conflict_for(&self, contact_id: ContactId) -> Option<PullConflict> {
        self.conflicts_snapshot()
            .into_iter()
            .find(|c| c.contact_id == contact_id)
    }
}
