//! User interface module for Profile Pulse
//!
//! Contains Iced GUI components and views with comprehensive contact field support,
//! alphabetical pagination, and multiple value fields (emails, phones, URLs).

use crate::core::contact::{Contact, ContactBuilder, SocialPlatform, SocialProfile};
use crate::db::repository::ContactRepository;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column, Row},
    Element, Length, Task, Theme,
};
use uuid::Uuid;

/// Current view in the application
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    /// Main contact list view with optional letter filter
    List { letter_filter: Option<char> },
    /// Add new contact form
    Add,
    /// Edit existing contact form
    Edit(Uuid),
    /// Contact detail view
    Detail(Uuid),
}

/// Main application state
#[derive(Debug)]
pub struct State {
    /// Contact repository for database operations
    repository: ContactRepository,
    /// Current view
    current_view: View,
    /// List of contacts
    contacts: Vec<Contact>,
    /// Search query
    search_query: String,
    /// Form fields for add/edit
    form: ContactForm,
    /// Error message to display
    error_message: Option<String>,
    /// Loading state
    is_loading: bool,
    /// Current page for letter-filtered view
    current_page: usize,
    /// Items per page
    items_per_page: usize,
}

/// Form state for adding/editing contacts with comprehensive Google Contacts fields
#[derive(Debug, Clone, Default)]
pub struct ContactForm {
    // Basic fields
    pub name: String,
    pub nickname: String,
    pub birthday: String,
    pub notes: String,
    
    // Multiple value fields
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub urls: Vec<String>,
    pub addresses: Vec<Address>,
    pub significant_dates: Vec<SignificantDate>,
    
    // Work fields
    pub organization: String,
    pub title: String,
    pub department: String,
    
    // Photo
    pub photo_url: String,
    
    // Social profiles (for editing)
    pub social_profiles: Vec<SocialProfileForm>,
    
    // Custom fields (user-defined key-value pairs)
    pub custom_field_pairs: Vec<CustomFieldPair>,
}

/// Address form representation
#[derive(Debug, Clone, Default)]
pub struct Address {
    pub label: String, // home, work, other
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

/// Significant date representation (anniversary, graduation, etc.)
#[derive(Debug, Clone, Default)]
pub struct SignificantDate {
    pub label: String, // anniversary, graduation, other
    pub date: String, // YYYY-MM-DD format
}

/// Custom field key-value pair
#[derive(Debug, Clone, Default)]
pub struct CustomFieldPair {
    pub key: String,
    pub value: String,
}

/// Social profile form representation
#[derive(Debug, Clone)]
pub struct SocialProfileForm {
    pub platform: SocialPlatform,
    pub username: String,
    pub url: String,
}

impl ContactForm {
    fn new() -> Self {
        Self {
            name: String::new(),
            nickname: String::new(),
            birthday: String::new(),
            notes: String::new(),
            emails: vec![String::new()],
            phones: vec![String::new()],
            urls: vec![String::new()],
            addresses: vec![Address::default()],
            significant_dates: vec![SignificantDate::default()],
            organization: String::new(),
            title: String::new(),
            department: String::new(),
            photo_url: String::new(),
            social_profiles: Vec::new(),
            custom_field_pairs: Vec::new(),
        }
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn from_contact(contact: &Contact) -> Self {
        // Extract emails from primary email and custom fields
        let mut emails = Vec::new();
        if let Some(email) = &contact.email {
            emails.push(email.clone());
        }
        
        // Extract phones
        let mut phones = Vec::new();
        if let Some(phone) = &contact.phone {
            phones.push(phone.clone());
        }
        
        // Extract URLs from social profiles and custom fields
        let mut urls = Vec::new();
        if let Some(photo_url) = &contact.photo_url {
            urls.push(photo_url.clone());
        }
        for profile in &contact.social_profiles {
            if !urls.contains(&profile.url) {
                urls.push(profile.url.clone());
            }
        }
        
        // Convert social profiles
        let social_profiles = contact.social_profiles.iter()
            .map(|p| SocialProfileForm {
                platform: p.platform,
                username: p.username.clone(),
                url: p.url.clone(),
            })
            .collect();
        
        // Ensure at least one empty field for each type
        if emails.is_empty() {
            emails.push(String::new());
        }
        if phones.is_empty() {
            phones.push(String::new());
        }
        if urls.is_empty() {
            urls.push(String::new());
        }
        
        // Parse addresses from custom fields
        let mut addresses = Vec::new();
        let mut addr_indices = std::collections::HashSet::new();
        for key in contact.custom_fields.keys() {
            if let Some(stripped) = key.strip_prefix("address_") {
                if let Some(idx_str) = stripped.split('_').next() {
                    if let Ok(idx) = idx_str.parse::<usize>() {
                        addr_indices.insert(idx);
                    }
                }
            }
        }
        for idx in addr_indices {
            addresses.push(Address {
                label: contact.custom_fields.get(&format!("address_{}_label", idx)).cloned().unwrap_or_default(),
                street: contact.custom_fields.get(&format!("address_{}_street", idx)).cloned().unwrap_or_default(),
                city: contact.custom_fields.get(&format!("address_{}_city", idx)).cloned().unwrap_or_default(),
                state: contact.custom_fields.get(&format!("address_{}_state", idx)).cloned().unwrap_or_default(),
                postal_code: contact.custom_fields.get(&format!("address_{}_postal_code", idx)).cloned().unwrap_or_default(),
                country: contact.custom_fields.get(&format!("address_{}_country", idx)).cloned().unwrap_or_default(),
            });
        }
        if addresses.is_empty() {
            addresses.push(Address::default());
        }
        
        // Parse significant dates from custom fields
        let mut significant_dates = Vec::new();
        let mut date_indices = std::collections::HashSet::new();
        for key in contact.custom_fields.keys() {
            if let Some(stripped) = key.strip_prefix("date_") {
                if !stripped.ends_with("_label") {
                    if let Ok(idx) = stripped.parse::<usize>() {
                        date_indices.insert(idx);
                    }
                }
            }
        }
        for idx in date_indices {
            if let Some(date) = contact.custom_fields.get(&format!("date_{}", idx)) {
                significant_dates.push(SignificantDate {
                    label: contact.custom_fields.get(&format!("date_{}_label", idx)).cloned().unwrap_or_default(),
                    date: date.clone(),
                });
            }
        }
        if significant_dates.is_empty() {
            significant_dates.push(SignificantDate::default());
        }
        
        Self {
            name: contact.name.clone(),
            nickname: contact.custom_fields.get("nickname").cloned().unwrap_or_default(),
            birthday: contact.custom_fields.get("birthday").cloned().unwrap_or_default(),
            notes: contact.custom_fields.get("notes").cloned().unwrap_or_default(),
            emails,
            phones,
            urls,
            addresses,
            significant_dates,
            organization: contact.organization.clone().unwrap_or_default(),
            title: contact.title.clone().unwrap_or_default(),
            department: contact.custom_fields.get("department").cloned().unwrap_or_default(),
            photo_url: contact.photo_url.clone().unwrap_or_default(),
            social_profiles,
            custom_field_pairs: contact.custom_fields.iter()
                .filter(|(k, _)| !k.starts_with("email_") && !k.starts_with("phone_") && !k.starts_with("url_")
                    && !k.starts_with("address_") && !k.starts_with("date_")
                    && k.as_str() != "nickname" && k.as_str() != "birthday" && k.as_str() != "notes" && k.as_str() != "department")
                .map(|(k, v)| CustomFieldPair {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        }
    }

    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }
    
    fn to_contact(&self, id: Option<Uuid>) -> Result<Contact, String> {
        let mut builder = ContactBuilder::new().name(&self.name);
        
        // Set ID if editing
        if let Some(_contact_id) = id {
            // We'll set this after building
        }
        
        // Add primary email (first non-empty)
        if let Some(email) = self.emails.iter().find(|e| !e.trim().is_empty()) {
            builder = builder.email(email);
        }
        
        // Add primary phone (first non-empty)
        if let Some(phone) = self.phones.iter().find(|p| !p.trim().is_empty()) {
            builder = builder.phone(phone);
        }
        
        // Add organization and title
        if !self.organization.trim().is_empty() {
            builder = builder.organization(&self.organization);
        }
        if !self.title.trim().is_empty() {
            builder = builder.title(&self.title);
        }
        
        // Add photo URL (first non-empty URL)
        if !self.photo_url.trim().is_empty() {
            builder = builder.photo_url(&self.photo_url);
        } else if let Some(url) = self.urls.iter().find(|u| !u.trim().is_empty()) {
            builder = builder.photo_url(url);
        }
        
        // Add custom fields
        if !self.nickname.trim().is_empty() {
            builder = builder.custom_field("nickname".to_string(), self.nickname.clone());
        }
        if !self.birthday.trim().is_empty() {
            builder = builder.custom_field("birthday".to_string(), self.birthday.clone());
        }
        if !self.notes.trim().is_empty() {
            builder = builder.custom_field("notes".to_string(), self.notes.clone());
        }
        if !self.department.trim().is_empty() {
            builder = builder.custom_field("department".to_string(), self.department.clone());
        }
        
        // Add additional emails as custom fields
        for (i, email) in self.emails.iter().enumerate().skip(1) {
            if !email.trim().is_empty() {
                builder = builder.custom_field(format!("email_{}", i), email.clone());
            }
        }
        
        // Add additional phones as custom fields
        for (i, phone) in self.phones.iter().enumerate().skip(1) {
            if !phone.trim().is_empty() {
                builder = builder.custom_field(format!("phone_{}", i), phone.clone());
            }
        }
        
        // Add URLs as custom fields
        for (i, url) in self.urls.iter().enumerate() {
            if !url.trim().is_empty() {
                builder = builder.custom_field(format!("url_{}", i), url.clone());
            }
        }
        
        // Add addresses as custom fields
        for (i, addr) in self.addresses.iter().enumerate() {
            if !addr.street.trim().is_empty() || !addr.city.trim().is_empty() {
                if !addr.label.trim().is_empty() {
                    builder = builder.custom_field(format!("address_{}_label", i), addr.label.clone());
                }
                if !addr.street.trim().is_empty() {
                    builder = builder.custom_field(format!("address_{}_street", i), addr.street.clone());
                }
                if !addr.city.trim().is_empty() {
                    builder = builder.custom_field(format!("address_{}_city", i), addr.city.clone());
                }
                if !addr.state.trim().is_empty() {
                    builder = builder.custom_field(format!("address_{}_state", i), addr.state.clone());
                }
                if !addr.postal_code.trim().is_empty() {
                    builder = builder.custom_field(format!("address_{}_postal_code", i), addr.postal_code.clone());
                }
                if !addr.country.trim().is_empty() {
                    builder = builder.custom_field(format!("address_{}_country", i), addr.country.clone());
                }
            }
        }
        
        // Add significant dates as custom fields
        for (i, date) in self.significant_dates.iter().enumerate() {
            if !date.date.trim().is_empty() {
                if !date.label.trim().is_empty() {
                    builder = builder.custom_field(format!("date_{}_label", i), date.label.clone());
                }
                builder = builder.custom_field(format!("date_{}", i), date.date.clone());
            }
        }
        
        // Add user-defined custom fields
        for pair in &self.custom_field_pairs {
            if !pair.key.trim().is_empty() && !pair.value.trim().is_empty() {
                builder = builder.custom_field(pair.key.clone(), pair.value.clone());
            }
        }
        
        // Add social profiles
        for profile_form in &self.social_profiles {
            if !profile_form.username.trim().is_empty() && !profile_form.url.trim().is_empty() {
                let profile = SocialProfile {
                    id: Uuid::new_v4(),
                    platform: profile_form.platform,
                    username: profile_form.username.clone(),
                    url: profile_form.url.clone(),
                    profile_pic_url: None,
                    verified: false,
                    confidence_score: None,
                    discovered_at: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                builder = builder.social_profile(profile);
            }
        }
        
        let mut contact = builder.build().map_err(|e| e.to_string())?;
        
        // Set ID if editing
        if let Some(contact_id) = id {
            contact.id = contact_id;
        }
        
        Ok(contact)
    }
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    LoadContacts,
    ContactsLoaded(Result<Vec<Contact>, String>),
    ChangeView(View),
    FilterByLetter(Option<char>),
    NextPage,
    PreviousPage,
    SearchChanged(String),
    
    // Basic field changes
    NameChanged(String),
    NicknameChanged(String),
    BirthdayChanged(String),
    NotesChanged(String),
    OrganizationChanged(String),
    TitleChanged(String),
    DepartmentChanged(String),
    PhotoUrlChanged(String),
    
    // Multiple value field changes
    EmailChanged(usize, String),
    PhoneChanged(usize, String),
    UrlChanged(usize, String),
    AddEmail,
    AddPhone,
    AddUrl,
    RemoveEmail(usize),
    RemovePhone(usize),
    RemoveUrl(usize),
    
    // Address changes
    AddressChanged(usize, usize, String), // (address_index, field_index, value)
    AddAddress,
    RemoveAddress(usize),
    
    // Significant date changes
    SignificantDateChanged(usize, usize, String), // (date_index, field_index, value)
    AddSignificantDate,
    RemoveSignificantDate(usize),
    
    // Custom field changes
    CustomFieldKeyChanged(usize, String),
    CustomFieldValueChanged(usize, String),
    AddCustomField,
    RemoveCustomField(usize),
    
    // Social profile changes
    AddSocialProfile,
    RemoveSocialProfile(usize),
    SocialPlatformChanged(usize, SocialPlatform),
    SocialUsernameChanged(usize, String),
    SocialUrlChanged(usize, String),
    
    // CRUD operations
    SaveNewContact,
    ContactSaved(Result<Contact, String>),
    UpdateContact(Uuid),
    ContactUpdated(Result<Contact, String>),
    DeleteContact(Uuid),
    ContactDeleted(Result<Uuid, String>),
    
    // Import/Export
    ImportVcf,
    VcfImported(Result<Vec<Contact>, String>),
    ExportVcf,
    VcfExported(Result<String, String>),
    
    // UI
    ClearError,
}

/// Initialize the application state
fn new() -> (State, Task<Message>) {
    panic!("Use new_with_repository() instead of new()");
}

/// Initialize with existing repository
pub fn new_with_repository(repository: ContactRepository) -> (State, Task<Message>) {
    let state = State {
        repository,
        current_view: View::List { letter_filter: None },
        contacts: Vec::new(),
        search_query: String::new(),
        form: ContactForm::new(),
        error_message: None,
        is_loading: false,
        current_page: 0,
        items_per_page: 50,
    };

    (state, Task::perform(async {}, |_| Message::LoadContacts))
}

/// Handle application messages and update state
fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::LoadContacts => {
            state.is_loading = true;
            let repo = state.repository.clone();
            // Remove the limit to load all contacts
            Task::perform(
                async move {
                    repo.list(None, None).await.map_err(|e| e.to_string())
                },
                Message::ContactsLoaded,
            )
        }

        Message::ContactsLoaded(result) => {
            state.is_loading = false;
            match result {
                Ok(contacts) => {
                    state.contacts = contacts;
                    state.error_message = None;
                }
                Err(err) => {
                    state.error_message = Some(format!("Failed to load contacts: {}", err));
                }
            }
            Task::none()
        }

        Message::ChangeView(view) => {
            // Prepare form if switching to edit view
            if let View::Edit(contact_id) = &view {
                if let Some(contact) = state.contacts.iter().find(|c| c.id == *contact_id) {
                    state.form = ContactForm::from_contact(contact);
                }
            } else if matches!(view, View::Add) {
                state.form.clear();
            }
            state.current_view = view;
            state.current_page = 0;
            Task::none()
        }
        
        Message::FilterByLetter(letter) => {
            state.current_view = View::List { letter_filter: letter };
            state.current_page = 0;
            Task::none()
        }
        
        Message::NextPage => {
            state.current_page += 1;
            Task::none()
        }
        
        Message::PreviousPage => {
            if state.current_page > 0 {
                state.current_page -= 1;
            }
            Task::none()
        }

        Message::SearchChanged(query) => {
            state.search_query = query;
            state.current_page = 0;
            Task::none()
        }

        // Basic field changes
        Message::NameChanged(value) => {
            state.form.name = value;
            Task::none()
        }
        Message::NicknameChanged(value) => {
            state.form.nickname = value;
            Task::none()
        }
        Message::BirthdayChanged(value) => {
            state.form.birthday = value;
            Task::none()
        }
        Message::NotesChanged(value) => {
            state.form.notes = value;
            Task::none()
        }
        Message::OrganizationChanged(value) => {
            state.form.organization = value;
            Task::none()
        }
        Message::TitleChanged(value) => {
            state.form.title = value;
            Task::none()
        }
        Message::DepartmentChanged(value) => {
            state.form.department = value;
            Task::none()
        }
        Message::PhotoUrlChanged(value) => {
            state.form.photo_url = value;
            Task::none()
        }

        // Multiple value field changes
        Message::EmailChanged(index, value) => {
            if index < state.form.emails.len() {
                state.form.emails[index] = value;
            }
            Task::none()
        }
        Message::PhoneChanged(index, value) => {
            if index < state.form.phones.len() {
                state.form.phones[index] = value;
            }
            Task::none()
        }
        Message::UrlChanged(index, value) => {
            if index < state.form.urls.len() {
                state.form.urls[index] = value;
            }
            Task::none()
        }
        Message::AddEmail => {
            state.form.emails.push(String::new());
            Task::none()
        }
        Message::AddPhone => {
            state.form.phones.push(String::new());
            Task::none()
        }
        Message::AddUrl => {
            state.form.urls.push(String::new());
            Task::none()
        }
        Message::RemoveEmail(index) => {
            if state.form.emails.len() > 1 {
                state.form.emails.remove(index);
            }
            Task::none()
        }
        Message::RemovePhone(index) => {
            if state.form.phones.len() > 1 {
                state.form.phones.remove(index);
            }
            Task::none()
        }
        Message::RemoveUrl(index) => {
            if state.form.urls.len() > 1 {
                state.form.urls.remove(index);
            }
            Task::none()
        }

        // Social profile changes
        Message::AddSocialProfile => {
            state.form.social_profiles.push(SocialProfileForm {
                platform: SocialPlatform::Other,
                username: String::new(),
                url: String::new(),
            });
            Task::none()
        }
        Message::RemoveSocialProfile(index) => {
            if index < state.form.social_profiles.len() {
                state.form.social_profiles.remove(index);
            }
            Task::none()
        }
        Message::SocialPlatformChanged(index, platform) => {
            if index < state.form.social_profiles.len() {
                state.form.social_profiles[index].platform = platform;
            }
            Task::none()
        }
        Message::SocialUsernameChanged(index, value) => {
            if index < state.form.social_profiles.len() {
                state.form.social_profiles[index].username = value;
            }
            Task::none()
        }
        Message::SocialUrlChanged(index, value) => {
            if index < state.form.social_profiles.len() {
                state.form.social_profiles[index].url = value;
            }
            Task::none()
        }
        
        // Address changes
        Message::AddressChanged(addr_idx, field_idx, value) => {
            if addr_idx < state.form.addresses.len() {
                match field_idx {
                    0 => state.form.addresses[addr_idx].label = value,
                    1 => state.form.addresses[addr_idx].street = value,
                    2 => state.form.addresses[addr_idx].city = value,
                    3 => state.form.addresses[addr_idx].state = value,
                    4 => state.form.addresses[addr_idx].postal_code = value,
                    5 => state.form.addresses[addr_idx].country = value,
                    _ => {}
                }
            }
            Task::none()
        }
        Message::AddAddress => {
            state.form.addresses.push(Address::default());
            Task::none()
        }
        Message::RemoveAddress(index) => {
            if state.form.addresses.len() > 1 {
                state.form.addresses.remove(index);
            }
            Task::none()
        }
        
        // Significant date changes
        Message::SignificantDateChanged(date_idx, field_idx, value) => {
            if date_idx < state.form.significant_dates.len() {
                match field_idx {
                    0 => state.form.significant_dates[date_idx].label = value,
                    1 => state.form.significant_dates[date_idx].date = value,
                    _ => {}
                }
            }
            Task::none()
        }
        Message::AddSignificantDate => {
            state.form.significant_dates.push(SignificantDate::default());
            Task::none()
        }
        Message::RemoveSignificantDate(index) => {
            if state.form.significant_dates.len() > 1 {
                state.form.significant_dates.remove(index);
            }
            Task::none()
        }
        
        // Custom field changes
        Message::CustomFieldKeyChanged(index, value) => {
            if index < state.form.custom_field_pairs.len() {
                state.form.custom_field_pairs[index].key = value;
            }
            Task::none()
        }
        Message::CustomFieldValueChanged(index, value) => {
            if index < state.form.custom_field_pairs.len() {
                state.form.custom_field_pairs[index].value = value;
            }
            Task::none()
        }
        Message::AddCustomField => {
            state.form.custom_field_pairs.push(CustomFieldPair::default());
            Task::none()
        }
        Message::RemoveCustomField(index) => {
            if index < state.form.custom_field_pairs.len() {
                state.form.custom_field_pairs.remove(index);
            }
            Task::none()
        }

        Message::SaveNewContact => {
            if !state.form.is_valid() {
                state.error_message = Some("Name is required".to_string());
                return Task::none();
            }

            let repo = state.repository.clone();
            let contact_result = state.form.to_contact(None);
            
            match contact_result {
                Ok(contact) => {
                    Task::perform(
                        async move {
                            repo.create(&contact).await.map_err(|e| format!("{:?}", e))?;
                            Ok(contact)
                        },
                        Message::ContactSaved,
                    )
                }
                Err(e) => {
                    state.error_message = Some(format!("Failed to build contact: {}", e));
                    Task::none()
                }
            }
        }

        Message::ContactSaved(result) => {
            match result {
                Ok(contact) => {
                    state.contacts.push(contact);
                    state.form.clear();
                    state.current_view = View::List { letter_filter: None };
                    state.error_message = None;
                }
                Err(err) => {
                    state.error_message = Some(format!("Failed to save contact: {}", err));
                }
            }
            Task::none()
        }

        Message::UpdateContact(contact_id) => {
            if !state.form.is_valid() {
                state.error_message = Some("Name is required".to_string());
                return Task::none();
            }

            let repo = state.repository.clone();
            let contact_result = state.form.to_contact(Some(contact_id));
            
            match contact_result {
                Ok(contact) => {
                    Task::perform(
                        async move {
                            repo.update(&contact).await.map_err(|e| format!("{:?}", e))?;
                            Ok(contact)
                        },
                        Message::ContactUpdated,
                    )
                }
                Err(e) => {
                    state.error_message = Some(format!("Failed to build contact: {}", e));
                    Task::none()
                }
            }
        }

        Message::ContactUpdated(result) => {
            match result {
                Ok(contact) => {
                    if let Some(idx) = state.contacts.iter().position(|c| c.id == contact.id) {
                        state.contacts[idx] = contact;
                    }
                    state.form.clear();
                    state.current_view = View::List { letter_filter: None };
                    state.error_message = None;
                }
                Err(err) => {
                    state.error_message = Some(format!("Failed to update contact: {}", err));
                }
            }
            Task::none()
        }

        Message::DeleteContact(contact_id) => {
            let repo = state.repository.clone();
            Task::perform(
                async move {
                    repo.delete(contact_id)
                        .await
                        .map(|_| contact_id)
                        .map_err(|e| e.to_string())
                },
                Message::ContactDeleted,
            )
        }

        Message::ContactDeleted(result) => {
            match result {
                Ok(contact_id) => {
                    state.contacts.retain(|c| c.id != contact_id);
                    state.error_message = None;
                    if state.current_view == View::Detail(contact_id) {
                        state.current_view = View::List { letter_filter: None };
                    }
                }
                Err(err) => {
                    state.error_message = Some(format!("Failed to delete contact: {}", err));
                }
            }
            Task::none()
        }

        Message::ClearError => {
            state.error_message = None;
            Task::none()
        }

        Message::ImportVcf => {
            let repo = state.repository.clone();
            Task::perform(
                async move {
                    use rfd::AsyncFileDialog;
                    
                    let file = AsyncFileDialog::new()
                        .add_filter("vCard Files", &["vcf", "vcard"])
                        .pick_file()
                        .await;
                    
                    if let Some(file) = file {
                        let path = file.path();
                        match crate::vcf::import_from_file(path) {
                            Ok(contacts) => {
                                for contact in &contacts {
                                    if let Err(e) = repo.create(contact).await {
                                        return Err(format!("Failed to save contact: {:?}", e));
                                    }
                                }
                                Ok(contacts)
                            }
                            Err(e) => Err(format!("Failed to import VCF: {}", e)),
                        }
                    } else {
                        Err("No file selected".to_string())
                    }
                },
                Message::VcfImported,
            )
        }

        Message::VcfImported(result) => {
            match result {
                Ok(contacts) => {
                    state.contacts.extend(contacts.clone());
                    state.error_message = Some(format!("✅ Successfully imported {} contacts", contacts.len()));
                    Task::perform(async {}, |_| Message::LoadContacts)
                }
                Err(err) => {
                    if err != "No file selected" {
                        state.error_message = Some(err);
                    }
                    Task::none()
                }
            }
        }

        Message::ExportVcf => {
            let contacts = state.contacts.clone();
            Task::perform(
                async move {
                    use rfd::AsyncFileDialog;
                    
                    let file = AsyncFileDialog::new()
                        .add_filter("vCard Files", &["vcf"])
                        .set_file_name("contacts.vcf")
                        .save_file()
                        .await;
                    
                    if let Some(file) = file {
                        let path = file.path();
                        match crate::vcf::export_to_file(&contacts, path) {
                            Ok(_) => Ok(format!("Exported {} contacts", contacts.len())),
                            Err(e) => Err(format!("Failed to export VCF: {}", e)),
                        }
                    } else {
                        Err("No file selected".to_string())
                    }
                },
                Message::VcfExported,
            )
        }

        Message::VcfExported(result) => {
            match result {
                Ok(msg) => {
                    state.error_message = Some(format!("✅ {}", msg));
                }
                Err(err) => {
                    if err != "No file selected" {
                        state.error_message = Some(err);
                    }
                }
            }
            Task::none()
        }
    }
}

/// Render the application view
fn view(state: &State) -> Element<'_, Message> {
    let content = match &state.current_view {
        View::List { letter_filter } => view_list(state, *letter_filter),
        View::Add => view_add_form(state),
        View::Edit(id) => view_edit_form(state, *id),
        View::Detail(id) => view_detail(state, *id),
    };

    let mut layout = column![].spacing(10).padding(20);

    // Error/success message banner
    if let Some(message) = &state.error_message {
        let is_success = message.starts_with("✅");
        let (bg_color, border_color, text_color) = if is_success {
            (
                iced::Color::from_rgb(0.9, 1.0, 0.9),
                iced::Color::from_rgb(0.2, 0.8, 0.2),
                iced::Color::from_rgb(0.1, 0.5, 0.1),
            )
        } else {
            (
                iced::Color::from_rgb(1.0, 0.9, 0.9),
                iced::Color::from_rgb(0.8, 0.2, 0.2),
                iced::Color::from_rgb(0.8, 0.2, 0.2),
            )
        };
        
        layout = layout.push(
            container(
                row![
                    text(message).style(move |_theme| text::Style {
                        color: Some(text_color),
                    }),
                    button("✕").on_press(Message::ClearError),
                ]
                .spacing(10),
            )
            .padding(10)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
        );
    }

    layout = layout.push(content);

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render contact list view with alphabetical pagination
fn view_list(state: &State, letter_filter: Option<char>) -> Element<'_, Message> {
    let title = text("Profile Pulse").size(32);
    let add_button = button("+ Add Contact").on_press(Message::ChangeView(View::Add));
    let import_button = button("📥 Import VCF").on_press(Message::ImportVcf);
    let export_button = button("📤 Export VCF").on_press(Message::ExportVcf);

    let search_input = text_input("Search contacts...", &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(10);

    let header = row![title, add_button, import_button, export_button].spacing(10);

    // Alphabetical filter buttons
    let mut alphabet_row = Row::new().spacing(5);
    alphabet_row = alphabet_row.push(
        button("All")
            .on_press(Message::FilterByLetter(None))
            .style(if letter_filter.is_none() {
                button::primary
            } else {
                button::secondary
            })
    );
    
    for letter in 'A'..='Z' {
        alphabet_row = alphabet_row.push(
            button(text(letter.to_string()).size(12))
                .on_press(Message::FilterByLetter(Some(letter)))
                .style(if letter_filter == Some(letter) {
                    button::primary
                } else {
                    button::secondary
                })
        );
    }

    // Filter contacts
    let filtered_contacts: Vec<&Contact> = state
        .contacts
        .iter()
        .filter(|contact| {
            // Search filter
            let matches_search = if state.search_query.is_empty() {
                true
            } else {
                let query = state.search_query.to_lowercase();
                contact.name.to_lowercase().contains(&query)
                    || contact.email.as_ref().map(|e| e.to_lowercase().contains(&query)).unwrap_or(false)
                    || contact.organization.as_ref().map(|o| o.to_lowercase().contains(&query)).unwrap_or(false)
                    || contact.phone.as_ref().map(|p| p.contains(&query)).unwrap_or(false)
            };
            
            // Letter filter
            let matches_letter = if let Some(letter) = letter_filter {
                contact.name.chars().next().map(|c| c.to_uppercase().next() == Some(letter)).unwrap_or(false)
            } else {
                true
            };
            
            matches_search && matches_letter
        })
        .collect();

    // Pagination
    let total_contacts = filtered_contacts.len();
    let total_pages = (total_contacts + state.items_per_page - 1) / state.items_per_page;
    let start_idx = state.current_page * state.items_per_page;
    let end_idx = (start_idx + state.items_per_page).min(total_contacts);
    
    let page_contacts: Vec<&Contact> = filtered_contacts
        .into_iter()
        .skip(start_idx)
        .take(state.items_per_page)
        .collect();

    let contact_count = text(format!(
        "Showing {}-{} of {} contacts {}",
        if total_contacts > 0 { start_idx + 1 } else { 0 },
        end_idx,
        total_contacts,
        if letter_filter.is_some() {
            format!("(filtered by {})", letter_filter.unwrap())
        } else {
            String::new()
        }
    ));

    // Pagination controls
    let mut pagination_row = Row::new().spacing(10);
    if state.current_page > 0 {
        pagination_row = pagination_row.push(button("← Previous").on_press(Message::PreviousPage));
    }
    pagination_row = pagination_row.push(text(format!("Page {} of {}", state.current_page + 1, total_pages.max(1))));
    if state.current_page + 1 < total_pages {
        pagination_row = pagination_row.push(button("Next →").on_press(Message::NextPage));
    }

    let mut contact_list = Column::new().spacing(5);

    if page_contacts.is_empty() {
        contact_list = contact_list.push(text("No contacts found").size(16));
    } else {
        for contact in page_contacts {
            contact_list = contact_list.push(view_contact_item(contact));
        }
    }

    column![
        header,
        search_input,
        scrollable(alphabet_row).direction(scrollable::Direction::Horizontal(scrollable::Scrollbar::default())),
        contact_count,
        if total_pages > 1 { pagination_row } else { Row::new() },
        scrollable(contact_list).height(Length::Fill)
    ]
    .spacing(15)
    .into()
}

/// Render a single contact item in the list
fn view_contact_item(contact: &Contact) -> Element<'_, Message> {
    let name = text(&contact.name).size(18);
    let mut info_parts = Vec::new();

    if let Some(email) = &contact.email {
        info_parts.push(format!("📧 {}", email));
    }
    if let Some(phone) = &contact.phone {
        info_parts.push(format!("📱 {}", phone));
    }
    if let Some(org) = &contact.organization {
        info_parts.push(format!("🏢 {}", org));
    }

    let info = text(info_parts.join(" • ")).size(13);

    let mut detail_parts = Vec::new();
    if !contact.social_profiles.is_empty() {
        detail_parts.push(format!("🔗 {} profiles", contact.social_profiles.len()));
    }
    
    // Count additional fields
    let url_count = contact.custom_fields.iter().filter(|(k, _)| k.starts_with("url_")).count();
    if url_count > 0 {
        detail_parts.push(format!("🌐 {} URLs", url_count));
    }

    let details = if !detail_parts.is_empty() {
        text(detail_parts.join(" • ")).size(12)
    } else {
        text("")
    };

    let view_button = button("View").on_press(Message::ChangeView(View::Detail(contact.id)));
    let edit_button = button("Edit").on_press(Message::ChangeView(View::Edit(contact.id)));
    let delete_button = button("Delete").on_press(Message::DeleteContact(contact.id));

    container(
        row![
            column![name, info, details].spacing(5).width(Length::Fill),
            row![view_button, edit_button, delete_button].spacing(5)
        ]
        .padding(10),
    )
    .padding(10)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.95, 0.95))),
        border: iced::Border {
            color: iced::Color::from_rgb(0.8, 0.8, 0.8),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Render add contact form
fn view_add_form(state: &State) -> Element<'_, Message> {
    render_contact_form(state, None)
}

/// Render edit contact form
fn view_edit_form(state: &State, contact_id: Uuid) -> Element<'_, Message> {
    render_contact_form(state, Some(contact_id))
}

/// Render contact form (shared between add and edit)
fn render_contact_form(state: &State, contact_id: Option<Uuid>) -> Element<'_, Message> {
    let title = text(if contact_id.is_some() { "Edit Contact" } else { "Add New Contact" }).size(24);
    let back_button = button("← Back to List").on_press(Message::ChangeView(View::List { letter_filter: None }));

    let mut form_content = Column::new().spacing(15);

    // Basic Information Section
    form_content = form_content.push(text("Basic Information").size(18));
    
    form_content = form_content.push(text("Name (required)").size(14));
    form_content = form_content.push(
        text_input("Full name", &state.form.name)
            .on_input(Message::NameChanged)
            .padding(10)
    );

    form_content = form_content.push(text("Nickname").size(14));
    form_content = form_content.push(
        text_input("Nickname", &state.form.nickname)
            .on_input(Message::NicknameChanged)
            .padding(10)
    );

    form_content = form_content.push(text("Birthday (YYYY-MM-DD)").size(14));
    form_content = form_content.push(
        text_input("1990-01-01", &state.form.birthday)
            .on_input(Message::BirthdayChanged)
            .padding(10)
    );

    // Contact Information Section
    form_content = form_content.push(text("Contact Information").size(18));
    
    // Emails
    form_content = form_content.push(
        row![
            text("Email Addresses").size(14),
            button("+ Add Email").on_press(Message::AddEmail)
        ].spacing(10)
    );
    
    for (i, email) in state.form.emails.iter().enumerate() {
        let mut email_row = Row::new().spacing(5);
        email_row = email_row.push(
            text_input(&format!("Email {}", i + 1), email)
                .on_input(move |v| Message::EmailChanged(i, v))
                .padding(10)
                .width(Length::Fill)
        );
        if state.form.emails.len() > 1 {
            email_row = email_row.push(button("−").on_press(Message::RemoveEmail(i)));
        }
        form_content = form_content.push(email_row);
    }

    // Phones
    form_content = form_content.push(
        row![
            text("Phone Numbers").size(14),
            button("+ Add Phone").on_press(Message::AddPhone)
        ].spacing(10)
    );
    
    for (i, phone) in state.form.phones.iter().enumerate() {
        let mut phone_row = Row::new().spacing(5);
        phone_row = phone_row.push(
            text_input(&format!("Phone {}", i + 1), phone)
                .on_input(move |v| Message::PhoneChanged(i, v))
                .padding(10)
                .width(Length::Fill)
        );
        if state.form.phones.len() > 1 {
            phone_row = phone_row.push(button("−").on_press(Message::RemovePhone(i)));
        }
        form_content = form_content.push(phone_row);
    }

    // URLs (for profile pictures and websites)
    form_content = form_content.push(
        row![
            text("URLs (websites, profile pictures)").size(14),
            button("+ Add URL").on_press(Message::AddUrl)
        ].spacing(10)
    );
    form_content = form_content.push(text("Note: First URL will be used as profile picture source").size(12));
    
for (i, url) in state.form.urls.iter().enumerate() {
        let mut url_row = Row::new().spacing(5);
        url_row = url_row.push(
            text_input(&format!("URL {}", i + 1), url)
                .on_input(move |v| Message::UrlChanged(i, v))
                .padding(10)
                .width(Length::Fill)
        );
        if state.form.urls.len() > 1 {
            url_row = url_row.push(button("−").on_press(Message::RemoveUrl(i)));
        }
        form_content = form_content.push(url_row);
    }
    
    // Addresses
    form_content = form_content.push(
        row![
            text("Addresses").size(14),
            button("+ Add Address").on_press(Message::AddAddress)
        ].spacing(10)
    );
    
    for (i, addr) in state.form.addresses.iter().enumerate() {
        let mut addr_section = Column::new().spacing(5);
        
        let header_row = row![
            text(format!("Address {}", i + 1)).size(13),
            if state.form.addresses.len() > 1 {
                button("−").on_press(Message::RemoveAddress(i))
            } else {
                button("−")
            }
        ].spacing(5);
        addr_section = addr_section.push(header_row);
        
        // Label (home, work, other)
        addr_section = addr_section.push(
            text_input("Label (home, work, other)", &addr.label)
                .on_input(move |v| Message::AddressChanged(i, 0, v))
                .padding(8)
        );
        
        // Street
        addr_section = addr_section.push(
            text_input("Street address", &addr.street)
                .on_input(move |v| Message::AddressChanged(i, 1, v))
                .padding(8)
        );
        
        // City, State, Postal Code
        let city_state_row = row![
            text_input("City", &addr.city)
                .on_input(move |v| Message::AddressChanged(i, 2, v))
                .padding(8)
                .width(Length::FillPortion(2)),
            text_input("State", &addr.state)
                .on_input(move |v| Message::AddressChanged(i, 3, v))
                .padding(8)
                .width(Length::FillPortion(1)),
            text_input("Postal Code", &addr.postal_code)
                .on_input(move |v| Message::AddressChanged(i, 4, v))
                .padding(8)
                .width(Length::FillPortion(1)),
        ].spacing(5);
        addr_section = addr_section.push(city_state_row);
        
        // Country
        addr_section = addr_section.push(
            text_input("Country", &addr.country)
                .on_input(move |v| Message::AddressChanged(i, 5, v))
                .padding(8)
        );
        
        form_content = form_content.push(
            container(addr_section)
                .padding(10)
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.97, 0.97, 0.97))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
        );
    }

    // Work Information Section
    form_content = form_content.push(text("Work Information").size(18));
    
    form_content = form_content.push(text("Organization").size(14));
    form_content = form_content.push(
        text_input("Company name", &state.form.organization)
            .on_input(Message::OrganizationChanged)
            .padding(10)
    );

    form_content = form_content.push(text("Job Title").size(14));
    form_content = form_content.push(
        text_input("Position", &state.form.title)
            .on_input(Message::TitleChanged)
            .padding(10)
    );

    form_content = form_content.push(text("Department").size(14));
    form_content = form_content.push(
        text_input("Department", &state.form.department)
            .on_input(Message::DepartmentChanged)
            .padding(10)
    );
    
    // Significant Dates Section
    form_content = form_content.push(
        row![
            text("Significant Dates").size(14),
            button("+ Add Date").on_press(Message::AddSignificantDate)
        ].spacing(10)
    );
    form_content = form_content.push(text("(Anniversaries, graduations, etc.)").size(12));
    
    for (i, sig_date) in state.form.significant_dates.iter().enumerate() {
        let mut date_row = Row::new().spacing(5);
        date_row = date_row.push(
            text_input("Label (anniversary, graduation, etc.)", &sig_date.label)
                .on_input(move |v| Message::SignificantDateChanged(i, 0, v))
                .padding(8)
                .width(Length::FillPortion(2))
        );
        date_row = date_row.push(
            text_input("Date (YYYY-MM-DD)", &sig_date.date)
                .on_input(move |v| Message::SignificantDateChanged(i, 1, v))
                .padding(8)
                .width(Length::FillPortion(2))
        );
        if state.form.significant_dates.len() > 1 {
            date_row = date_row.push(button("−").on_press(Message::RemoveSignificantDate(i)));
        }
        form_content = form_content.push(date_row);
    }

    // Social Profiles Section
    form_content = form_content.push(
        row![
            text("Social Media Profiles").size(18),
            button("+ Add Profile").on_press(Message::AddSocialProfile)
        ].spacing(10)
    );
    
    for (i, profile) in state.form.social_profiles.iter().enumerate() {
        let platform_text = format!("{:?}", profile.platform);
        form_content = form_content.push(
            column![
                row![
                    text(format!("Profile {} - {}", i + 1, platform_text)).size(14),
                    button("Remove").on_press(Message::RemoveSocialProfile(i))
                ].spacing(10),
                text_input("Username", &profile.username)
                    .on_input(move |v| Message::SocialUsernameChanged(i, v))
                    .padding(10),
                text_input("Profile URL", &profile.url)
                    .on_input(move |v| Message::SocialUrlChanged(i, v))
                    .padding(10),
            ].spacing(5)
        );
    }
    
    // Custom Fields Section
    form_content = form_content.push(
        row![
            text("Custom Fields").size(18),
            button("+ Add Field").on_press(Message::AddCustomField)
        ].spacing(10)
    );
    form_content = form_content.push(text("Add any additional information as key-value pairs").size(12));
    
    for (i, field) in state.form.custom_field_pairs.iter().enumerate() {
        let mut field_row = Row::new().spacing(5);
        field_row = field_row.push(
            text_input("Field name", &field.key)
                .on_input(move |v| Message::CustomFieldKeyChanged(i, v))
                .padding(8)
                .width(Length::FillPortion(1))
        );
        field_row = field_row.push(
            text_input("Value", &field.value)
                .on_input(move |v| Message::CustomFieldValueChanged(i, v))
                .padding(8)
                .width(Length::FillPortion(2))
        );
        field_row = field_row.push(button("−").on_press(Message::RemoveCustomField(i)));
        form_content = form_content.push(field_row);
    }

    // Notes Section
    form_content = form_content.push(text("Notes").size(18));
    form_content = form_content.push(
        text_input("Additional notes...", &state.form.notes)
            .on_input(Message::NotesChanged)
            .padding(10)
    );

    // Action buttons
    let save_button = if state.form.is_valid() {
        if let Some(id) = contact_id {
            button("💾 Update Contact").on_press(Message::UpdateContact(id))
        } else {
            button("💾 Save Contact").on_press(Message::SaveNewContact)
        }
    } else {
        button("💾 Save Contact (Name required)")
    };

    let cancel_button = button("Cancel").on_press(Message::ChangeView(View::List { letter_filter: None }));

    form_content = form_content.push(
        row![save_button, cancel_button].spacing(10)
    );

    column![
        row![title, back_button].spacing(20),
        scrollable(form_content).height(Length::Fill)
    ]
    .spacing(20)
    .into()
}

/// Render contact detail view
fn view_detail(state: &State, contact_id: Uuid) -> Element<'_, Message> {
    let contact = state.contacts.iter().find(|c| c.id == contact_id);

    if contact.is_none() {
        return column![
            text("Contact not found").size(24),
            button("← Back to List").on_press(Message::ChangeView(View::List { letter_filter: None }))
        ]
        .spacing(20)
        .into();
    }

    let contact = contact.unwrap();

    let title = text(&contact.name).size(28);
    let back_button = button("← Back to List").on_press(Message::ChangeView(View::List { letter_filter: None }));
    let edit_button = button("✏️ Edit").on_press(Message::ChangeView(View::Edit(contact.id)));
    let delete_button = button("🗑️ Delete").on_press(Message::DeleteContact(contact.id));

    let mut details = Column::new().spacing(15);

    // Basic info
    if let Some(nickname) = contact.custom_fields.get("nickname") {
        if !nickname.is_empty() {
            details = details.push(
                row![text("Nickname:").size(14), text(nickname).size(14)].spacing(10)
            );
        }
    }

    if let Some(birthday) = contact.custom_fields.get("birthday") {
        if !birthday.is_empty() {
            details = details.push(
                row![text("Birthday:").size(14), text(birthday).size(14)].spacing(10)
            );
        }
    }

    // Contact info
    details = details.push(text("Contact Information").size(16));
    
    if let Some(email) = &contact.email {
        details = details.push(
            row![text("📧 Email:").size(14), text(email).size(14)].spacing(10)
        );
    }
    
    // Additional emails
    for (key, value) in &contact.custom_fields {
        if key.starts_with("email_") && !value.is_empty() {
            details = details.push(
                row![text("📧 Email:").size(14), text(value).size(14)].spacing(10)
            );
        }
    }

    if let Some(phone) = &contact.phone {
        details = details.push(
            row![text("📱 Phone:").size(14), text(phone).size(14)].spacing(10)
        );
    }
    
    // Additional phones
    for (key, value) in &contact.custom_fields {
        if key.starts_with("phone_") && !value.is_empty() {
            details = details.push(
                row![text("📱 Phone:").size(14), text(value).size(14)].spacing(10)
            );
        }
    }

    // URLs
    let mut has_urls = false;
    for (key, value) in &contact.custom_fields {
        if key.starts_with("url_") && !value.is_empty() {
            if !has_urls {
                details = details.push(text("Websites & URLs").size(16));
                has_urls = true;
            }
            details = details.push(
                row![text("🌐 URL:").size(14), text(value).size(14)].spacing(10)
            );
        }
    }

    // Work info
    if contact.organization.is_some() || contact.title.is_some() || contact.custom_fields.get("department").is_some() {
        details = details.push(text("Work Information").size(16));
        
        if let Some(org) = &contact.organization {
            details = details.push(
                row![text("🏢 Organization:").size(14), text(org).size(14)].spacing(10)
            );
        }

        if let Some(title_text) = &contact.title {
            details = details.push(
                row![text("💼 Title:").size(14), text(title_text).size(14)].spacing(10)
            );
        }

        if let Some(dept) = contact.custom_fields.get("department") {
            if !dept.is_empty() {
                details = details.push(
                    row![text("🏛️ Department:").size(14), text(dept).size(14)].spacing(10)
                );
            }
        }
    }

    // Social profiles
    if !contact.social_profiles.is_empty() {
        details = details.push(text("Social Media Profiles").size(16));
        for profile in &contact.social_profiles {
            details = details.push(
                column![
                    text(format!("{} {}", platform_emoji(&profile.platform), profile.platform.as_str())).size(14),
                    text(format!("  @{}", profile.username)).size(13),
                    text(format!("  {}", profile.url)).size(12),
                ].spacing(2)
            );
        }
    }
    
    // Addresses
    let mut has_addresses = false;
    for (_i, (key, value)) in contact.custom_fields.iter().enumerate() {
        if key.starts_with("address_") && key.ends_with("_street") && !value.is_empty() {
            if !has_addresses {
                details = details.push(text("Addresses").size(16));
                has_addresses = true;
            }
            
            let addr_idx = key.strip_prefix("address_").and_then(|s| s.strip_suffix("_street"))
                .and_then(|s| s.parse::<usize>().ok());
            
            if let Some(idx) = addr_idx {
                let mut addr_lines = Vec::new();
                
                if let Some(label) = contact.custom_fields.get(&format!("address_{}_label", idx)) {
                    if !label.is_empty() {
                        addr_lines.push(format!("📍 {} Address", label));
                    }
                }
                
                addr_lines.push(format!("  {}", value));
                
                let mut city_state_zip = Vec::new();
                if let Some(city) = contact.custom_fields.get(&format!("address_{}_city", idx)) {
                    if !city.is_empty() {
                        city_state_zip.push(city.clone());
                    }
                }
                if let Some(state) = contact.custom_fields.get(&format!("address_{}_state", idx)) {
                    if !state.is_empty() {
                        city_state_zip.push(state.clone());
                    }
                }
                if let Some(postal) = contact.custom_fields.get(&format!("address_{}_postal_code", idx)) {
                    if !postal.is_empty() {
                        city_state_zip.push(postal.clone());
                    }
                }
                if !city_state_zip.is_empty() {
                    addr_lines.push(format!("  {}", city_state_zip.join(", ")));
                }
                
                if let Some(country) = contact.custom_fields.get(&format!("address_{}_country", idx)) {
                    if !country.is_empty() {
                        addr_lines.push(format!("  {}", country));
                    }
                }
                
                for line in addr_lines {
                    details = details.push(text(line).size(14));
                }
            }
        }
    }
    
    // Significant Dates
    let mut has_dates = false;
    for (key, value) in &contact.custom_fields {
        if key.starts_with("date_") && !key.ends_with("_label") && !value.is_empty() {
            if !has_dates {
                details = details.push(text("Significant Dates").size(16));
                has_dates = true;
            }
            
            let date_idx = key.strip_prefix("date_").and_then(|s| s.parse::<usize>().ok());
            let label = date_idx.and_then(|idx| contact.custom_fields.get(&format!("date_{}_label", idx)));
            
            if let Some(label_text) = label {
                details = details.push(
                    text(format!("📅 {}: {}", label_text, value)).size(14)
                );
            } else {
                details = details.push(
                    text(format!("📅 Date: {}", value)).size(14)
                );
            }
        }
    }
    
    // Custom Fields (user-defined)
    let mut user_custom_fields = Vec::new();
    for (key, value) in &contact.custom_fields {
        // Filter out internal fields
        if !key.starts_with("email_") && !key.starts_with("phone_") && !key.starts_with("url_")
            && !key.starts_with("address_") && !key.starts_with("date_")
            && key != "nickname" && key != "birthday" && key != "notes" && key != "department"
            && !value.is_empty() {
            user_custom_fields.push((key, value));
        }
    }
    
    if !user_custom_fields.is_empty() {
        details = details.push(text("Custom Fields").size(16));
        for (key, value) in user_custom_fields {
            details = details.push(
                row![
                    text(format!("{}:", key)).size(14),
                    text(value).size(14)
                ].spacing(10)
            );
        }
    }

    // Notes
    if let Some(notes) = contact.custom_fields.get("notes") {
        if !notes.is_empty() {
            details = details.push(text("Notes").size(16));
            details = details.push(text(notes).size(14));
        }
    }

    // Metadata
    details = details.push(text("Metadata").size(16));
    details = details.push(
        text(format!("Created: {}", contact.created_at.format("%Y-%m-%d %H:%M:%S"))).size(12)
    );
    details = details.push(
        text(format!("Updated: {}", contact.updated_at.format("%Y-%m-%d %H:%M:%S"))).size(12)
    );

    column![
        row![title, back_button, edit_button, delete_button].spacing(10),
        scrollable(details).height(Length::Fill),
    ]
    .spacing(20)
    .into()
}

/// Get emoji for social platform
fn platform_emoji(platform: &SocialPlatform) -> &'static str {
    match platform {
        SocialPlatform::LinkedIn => "💼",
        SocialPlatform::Twitter => "🐦",
        SocialPlatform::Facebook => "👥",
        SocialPlatform::Instagram => "📷",
        SocialPlatform::GitHub => "🐙",
        SocialPlatform::Mastodon => "🐘",
        SocialPlatform::Other => "🔗",
    }
}

/// Get the application theme
fn theme(_state: &State) -> Theme {
    Theme::TokyoNight
}

/// Run the Iced application
pub fn run() -> iced::Result {
    iced::application(new, update, view).theme(theme).run()
}

/// Run with existing repository
pub fn run_with_repository(repository: ContactRepository) -> iced::Result {
    iced::application(
        move || new_with_repository(repository.clone()),
        update,
        view,
    )
    .theme(theme)
    .run()
}