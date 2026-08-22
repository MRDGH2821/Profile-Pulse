use crate::views::{ContactDetail, ContactList, Profiles};
use dioxus::prelude::*;

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Profiles {},
    #[route("/profiles/:profile_id/contacts")]
    ContactList { profile_id: String },
    #[route("/profiles/:profile_id/contacts/:contact_id")]
    ContactDetail {
        profile_id: String,
        contact_id: String,
    },
}
