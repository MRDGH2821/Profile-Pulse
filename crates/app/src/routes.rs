use crate::AppShell;
use crate::views::{
    BackupsSettings, ContactDetail, ContactList, PicSourcesSettings, Profiles, SyncSettings,
};
use dioxus::prelude::*;

#[rustfmt::skip]
#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Profiles {},
        #[route("/profiles/:profile_id/contacts")]
        ContactList { profile_id: String },
        #[route("/profiles/:profile_id/contacts/:contact_id")]
        ContactDetail {
            profile_id: String,
            contact_id: String,
        },
        #[route("/settings/pic-sources")]
        PicSourcesSettings {},
        #[route("/settings/sync")]
        SyncSettings {},
        #[route("/settings/backups")]
        BackupsSettings {},
}
