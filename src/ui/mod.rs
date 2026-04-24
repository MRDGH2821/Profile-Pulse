//! User interface module for Profile Pulse
//!
//! Contains Iced GUI components and views with comprehensive contact field support,
//! alphabetical pagination, and multiple value fields (emails, phones, URLs).

use crate::core::contact::{
    Contact, ContactAddress, ContactBuilder, ContactDate, ContactEmail, ContactPhone, ContactUrl,
    SocialPlatform,
};
use crate::core::labels::{AddressLabel, DateLabel, EmailLabel, PhoneLabel};
use crate::vcf::VcfRepository;
use crate::workspace::{Workspace, WorkspaceManager};
use chrono::NaiveDate;
use iced::{
    widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column, Row, Space},
    Element, Length, Task, Theme,
};

use uuid::Uuid;

/// Current view in the application
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    /// Workspace selector (choose which VCF file to work with)
    WorkspaceSelector,
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
    /// Workspace manager
    workspace_manager: WorkspaceManager,
    /// Current workspace (None if at selector)
    current_workspace: Option<Workspace>,
    /// List of all workspaces
    workspaces: Vec<Workspace>,
    /// Contact repository for VCF operations (None if no workspace selected)
    repository: Option<VcfRepository>,
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
    /// New workspace name input
    new_workspace_name: String,
}

/// Form state for adding/editing contacts with comprehensive Google Contacts fields
#[derive(Debug, Clone, Default)]
pub struct ContactForm {
    // Structured name fields
    pub name_prefix: String,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub name_suffix: String,
    pub nickname: String,
    pub notes: String,
    
    // Multiple value fields with labels
    pub emails: Vec<EmailForm>,
    pub phones: Vec<PhoneForm>,
    pub urls: Vec<UrlForm>,
    pub addresses: Vec<AddressForm>,
    pub dates: Vec<DateForm>,
    
    // Work fields
    pub organization: String,
    pub title: String,
    pub department: String,
    
    // Photo
    pub photo_url: String,
    
    // Custom fields (user-defined key-value pairs)
    pub custom_field_pairs: Vec<CustomFieldPair>,
}

/// Email form representation with label
#[derive(Debug, Clone, Default)]
pub struct EmailForm {
    pub email: String,
    pub label: String,
    pub selected_option: Option<EmailLabelOption>,
    pub custom_label: String,
}

/// Phone form representation with label
#[derive(Debug, Clone, Default)]
pub struct PhoneForm {
    pub phone: String,
    pub label: String,
    pub selected_option: Option<PhoneLabelOption>,
    pub custom_label: String,
}

/// Address form representation with label
#[derive(Debug, Clone, Default)]
pub struct AddressForm {
    pub label: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub selected_option: Option<AddressLabelOption>,
    pub custom_label: String,
}

/// Date form representation with label
#[derive(Debug, Clone, Default)]
pub struct DateForm {
    pub label: String,
    pub date: String, // YYYY-MM-DD format
    pub selected_option: Option<DateLabelOption>,
    pub custom_label: String,
}

/// Custom field key-value pair
#[derive(Debug, Clone, Default)]
pub struct CustomFieldPair {
    pub key: String,
    pub value: String,
}

/// URL form representation with label
#[derive(Debug, Clone, Default)]
pub struct UrlForm {
    pub url: String,
    pub label: String,
    pub selected_option: Option<UrlLabelOption>,
    pub custom_label: String,
}

impl ContactForm {
    fn new() -> Self {
        Self {
            name_prefix: String::new(),
            first_name: String::new(),
            middle_name: String::new(),
            last_name: String::new(),
            name_suffix: String::new(),
            nickname: String::new(),
            notes: String::new(),
            emails: vec![EmailForm {
                email: String::new(),
                label: EmailLabel::default().to_string_value(),
                selected_option: Some(EmailLabelOption::Home),
                custom_label: String::new(),
            }],
            phones: vec![PhoneForm {
                phone: String::new(),
                label: PhoneLabel::default().to_string_value(),
                selected_option: Some(PhoneLabelOption::Mobile),
                custom_label: String::new(),
            }],
            urls: vec![UrlForm {
                url: String::new(),
                label: String::new(),
                selected_option: Some(UrlLabelOption::Other),
                custom_label: String::new(),
            }],
            addresses: vec![AddressForm {
                label: AddressLabel::default().to_string_value(),
                selected_option: Some(AddressLabelOption::Home),
                custom_label: String::new(),
                ..Default::default()
            }],
            dates: vec![DateForm {
                label: DateLabel::default().to_string_value(),
                date: String::new(),
                selected_option: Some(DateLabelOption::Birthday),
                custom_label: String::new(),
            }],
            organization: String::new(),
            title: String::new(),
            department: String::new(),
            photo_url: String::new(),
            custom_field_pairs: Vec::new(),
        }
    }

    fn clear(&mut self) {
        *self = Self::new();
    }

    fn from_contact(contact: &Contact) -> Self {
        // Extract emails from structured fields
        let mut emails: Vec<EmailForm> = contact
            .emails
            .iter()
            .map(|e| {
                let (selected_option, custom_label) = Self::parse_email_label(&e.label);
                EmailForm {
                    email: e.email.clone(),
                    label: e.label.clone(),
                    selected_option,
                    custom_label,
                }
            })
            .collect();
        
        // Extract phones from structured fields
        let mut phones: Vec<PhoneForm> = contact
            .phones
            .iter()
            .map(|p| {
                let (selected_option, custom_label) = Self::parse_phone_label(&p.label);
                PhoneForm {
                    phone: p.phone.clone(),
                    label: p.label.clone(),
                    selected_option,
                    custom_label,
                }
            })
            .collect();
        
        // Extract addresses from structured fields
        let mut addresses: Vec<AddressForm> = contact
            .addresses
            .iter()
            .map(|a| {
                let (selected_option, custom_label) = Self::parse_address_label(&a.label);
                AddressForm {
                    label: a.label.clone(),
                    street: a.street.clone().unwrap_or_default(),
                    city: a.city.clone().unwrap_or_default(),
                    state: a.state.clone().unwrap_or_default(),
                    postal_code: a.postal_code.clone().unwrap_or_default(),
                    country: a.country.clone().unwrap_or_default(),
                    selected_option,
                    custom_label,
                }
            })
            .collect();
        
        // Extract dates from structured fields
        let mut dates: Vec<DateForm> = contact
            .dates
            .iter()
            .map(|d| {
                let (selected_option, custom_label) = Self::parse_date_label(&d.label);
                DateForm {
                    label: d.label.clone(),
                    date: d.date.format("%Y-%m-%d").to_string(),
                    selected_option,
                    custom_label,
                }
            })
            .collect();
        
        // Extract URLs from contact.urls with labels
        let mut urls: Vec<UrlForm> = contact
            .urls
            .iter()
            .map(|u| {
                let label = u.label.clone().unwrap_or_default();
                let (selected_option, custom_label) = Self::parse_url_label(&label);
                UrlForm {
                    url: u.url.clone(),
                    label,
                    selected_option,
                    custom_label,
                }
            })
            .collect();
        
        // Ensure at least one empty field for each type
        if emails.is_empty() {
            emails.push(EmailForm {
                email: String::new(),
                label: EmailLabel::default().to_string_value(),
                selected_option: Some(EmailLabelOption::Home),
                custom_label: String::new(),
            });
        }
        if phones.is_empty() {
            phones.push(PhoneForm {
                phone: String::new(),
                label: PhoneLabel::default().to_string_value(),
                selected_option: Some(PhoneLabelOption::Mobile),
                custom_label: String::new(),
            });
        }
        if addresses.is_empty() {
            addresses.push(AddressForm {
                label: AddressLabel::default().to_string_value(),
                selected_option: Some(AddressLabelOption::Home),
                custom_label: String::new(),
                ..Default::default()
            });
        }
        if dates.is_empty() {
            dates.push(DateForm {
                label: DateLabel::default().to_string_value(),
                date: String::new(),
                selected_option: Some(DateLabelOption::Birthday),
                custom_label: String::new(),
            });
        }
        if urls.is_empty() {
            urls.push(UrlForm {
                url: String::new(),
                label: String::new(),
                selected_option: Some(UrlLabelOption::Other),
                custom_label: String::new(),
            });
        }
        
        Self {
            name_prefix: contact.name_prefix.clone().unwrap_or_default(),
            first_name: contact.first_name.clone().unwrap_or_default(),
            middle_name: contact.middle_name.clone().unwrap_or_default(),
            last_name: contact.last_name.clone().unwrap_or_default(),
            name_suffix: contact.name_suffix.clone().unwrap_or_default(),
            nickname: contact.nickname.clone().unwrap_or_default(),
            notes: contact.notes.clone().unwrap_or_default(),
            emails,
            phones,
            urls,
            addresses,
            dates,
            organization: contact.organization.clone().unwrap_or_default(),
            title: contact.title.clone().unwrap_or_default(),
            department: contact.department.clone().unwrap_or_default(),
            photo_url: contact.photo_url.clone().unwrap_or_default(),
            custom_field_pairs: contact
                .custom_fields
                .iter()
                .map(|(k, v)| CustomFieldPair {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        }
    }

    fn is_valid(&self) -> bool {
        // At least first name or last name must be provided
        !self.first_name.trim().is_empty() || !self.last_name.trim().is_empty()
    }
    
    fn to_contact(&self, id: Option<Uuid>) -> Result<Contact, String> {
        // Build full name from structured fields
        let full_name = vec![
            self.name_prefix.trim(),
            self.first_name.trim(),
            self.middle_name.trim(),
            self.last_name.trim(),
            self.name_suffix.trim(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
        
        let name = if full_name.is_empty() {
            "Unnamed Contact".to_string()
        } else {
            full_name
        };
        
        let mut builder = ContactBuilder::new().name(&name);
        
        // Add structured name fields
        if !self.name_prefix.trim().is_empty() {
            builder = builder.name_prefix(&self.name_prefix);
        }
        if !self.first_name.trim().is_empty() {
            builder = builder.first_name(&self.first_name);
        }
        if !self.middle_name.trim().is_empty() {
            builder = builder.middle_name(&self.middle_name);
        }
        if !self.last_name.trim().is_empty() {
            builder = builder.last_name(&self.last_name);
        }
        if !self.name_suffix.trim().is_empty() {
            builder = builder.name_suffix(&self.name_suffix);
        }
        if !self.nickname.trim().is_empty() {
            builder = builder.nickname(&self.nickname);
        }
        if !self.notes.trim().is_empty() {
            builder = builder.notes(&self.notes);
        }
        
        // Add organization, title, and department
        if !self.organization.trim().is_empty() {
            builder = builder.organization(&self.organization);
        }
        if !self.title.trim().is_empty() {
            builder = builder.title(&self.title);
        }
        if !self.department.trim().is_empty() {
            builder = builder.department(&self.department);
        }
        
        // Add photo URL (first non-empty URL)
        if !self.photo_url.trim().is_empty() {
            builder = builder.photo_url(&self.photo_url);
        } else if let Some(url_form) = self.urls.iter().find(|u| !u.url.trim().is_empty()) {
            builder = builder.photo_url(&url_form.url);
        }
        
        // Add structured emails
        for email_form in &self.emails {
            if !email_form.email.trim().is_empty() {
                let contact_email = ContactEmail::new(
                    email_form.email.trim().to_string(),
                    email_form.label.trim().to_string(),
                );
                builder = builder.email_entry(contact_email);
            }
        }
        
        // Add structured phones
        for phone_form in &self.phones {
            if !phone_form.phone.trim().is_empty() {
                let contact_phone = ContactPhone::new(
                    phone_form.phone.trim().to_string(),
                    phone_form.label.trim().to_string(),
                );
                builder = builder.phone_entry(contact_phone);
            }
        }
        
        // Add structured addresses
        for addr_form in &self.addresses {
            if !addr_form.street.trim().is_empty() || !addr_form.city.trim().is_empty() {
                let contact_address = ContactAddress::builder()
                    .street(addr_form.street.trim().to_string())
                    .city(addr_form.city.trim().to_string())
                    .state(addr_form.state.trim().to_string())
                    .postal_code(addr_form.postal_code.trim().to_string())
                    .country(addr_form.country.trim().to_string())
                    .label(addr_form.label.trim().to_string())
                    .build();
                builder = builder.address(contact_address);
            }
        }
        
        // Add structured dates
        for date_form in &self.dates {
            if !date_form.date.trim().is_empty() {
                if let Ok(naive_date) = NaiveDate::parse_from_str(date_form.date.trim(), "%Y-%m-%d") {
                    let contact_date = ContactDate::new(
                        naive_date,
                        date_form.label.trim().to_string(),
                    );
                    builder = builder.date(contact_date);
                }
            }
        }
        
        // Add URLs as ContactUrl objects with labels
        for url_form in &self.urls {
            if !url_form.url.trim().is_empty() {
                let label = if url_form.label.trim().is_empty() {
                    None
                } else {
                    Some(url_form.label.trim().to_string())
                };
                let contact_url = ContactUrl::new(url_form.url.trim().to_string(), label);
                builder = builder.url(contact_url);
            }
        }
        
        // Add custom field pairs as custom fields
        let mut contact = builder.build().map_err(|e| e.to_string())?;
        
        // Set ID if editing
        if let Some(contact_id) = id {
            contact.id = contact_id;
        }
        
        Ok(contact)
    }

    /// Parse email label string to dropdown option and custom label
    fn parse_email_label(label: &str) -> (Option<EmailLabelOption>, String) {
        match label.to_lowercase().as_str() {
            "home" => (Some(EmailLabelOption::Home), String::new()),
            "work" => (Some(EmailLabelOption::Work), String::new()),
            "other" => (Some(EmailLabelOption::Other), String::new()),
            _ => (Some(EmailLabelOption::Custom), label.to_string()),
        }
    }

    /// Parse phone label string to dropdown option and custom label
    fn parse_phone_label(label: &str) -> (Option<PhoneLabelOption>, String) {
        match label.to_lowercase().as_str() {
            "mobile" | "cell" => (Some(PhoneLabelOption::Mobile), String::new()),
            "home" => (Some(PhoneLabelOption::Home), String::new()),
            "work" => (Some(PhoneLabelOption::Work), String::new()),
            "main" => (Some(PhoneLabelOption::Main), String::new()),
            "home fax" | "homefax" => (Some(PhoneLabelOption::HomeFax), String::new()),
            "work fax" | "workfax" => (Some(PhoneLabelOption::WorkFax), String::new()),
            "pager" => (Some(PhoneLabelOption::Pager), String::new()),
            "other" => (Some(PhoneLabelOption::Other), String::new()),
            _ => (Some(PhoneLabelOption::Custom), label.to_string()),
        }
    }

    /// Parse address label string to dropdown option and custom label
    fn parse_address_label(label: &str) -> (Option<AddressLabelOption>, String) {
        match label.to_lowercase().as_str() {
            "home" => (Some(AddressLabelOption::Home), String::new()),
            "work" => (Some(AddressLabelOption::Work), String::new()),
            "other" => (Some(AddressLabelOption::Other), String::new()),
            _ => (Some(AddressLabelOption::Custom), label.to_string()),
        }
    }

    /// Parse date label string to dropdown option and custom label
    fn parse_date_label(label: &str) -> (Option<DateLabelOption>, String) {
        match label.to_lowercase().as_str() {
            "birthday" | "bday" => (Some(DateLabelOption::Birthday), String::new()),
            "anniversary" => (Some(DateLabelOption::Anniversary), String::new()),
            "other" => (Some(DateLabelOption::Other), String::new()),
            _ => (Some(DateLabelOption::Custom), label.to_string()),
        }
    }

    /// Parse URL label string to dropdown option and custom label
    fn parse_url_label(label: &str) -> (Option<UrlLabelOption>, String) {
        match label.to_lowercase().as_str() {
            "homepage" | "home page" => (Some(UrlLabelOption::HomePage), String::new()),
            "work" => (Some(UrlLabelOption::Work), String::new()),
            "blog" => (Some(UrlLabelOption::Blog), String::new()),
            "profile" => (Some(UrlLabelOption::Profile), String::new()),
            "github" => (Some(UrlLabelOption::GitHub), String::new()),
            "linkedin" => (Some(UrlLabelOption::LinkedIn), String::new()),
            "twitter" | "x" => (Some(UrlLabelOption::Twitter), String::new()),
            "facebook" => (Some(UrlLabelOption::Facebook), String::new()),
            "instagram" => (Some(UrlLabelOption::Instagram), String::new()),
            "mastodon" => (Some(UrlLabelOption::Mastodon), String::new()),
            "other" => (Some(UrlLabelOption::Other), String::new()),
            _ => (Some(UrlLabelOption::Custom), label.to_string()),
        }
    }
}

/// Label selection options for email dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailLabelOption {
    Home,
    Work,
    Other,
    Custom,
}

impl EmailLabelOption {
    pub const ALL: [EmailLabelOption; 4] = [
        EmailLabelOption::Home,
        EmailLabelOption::Work,
        EmailLabelOption::Other,
        EmailLabelOption::Custom,
    ];
}

impl std::fmt::Display for EmailLabelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Home => "Home",
                Self::Work => "Work",
                Self::Other => "Other",
                Self::Custom => "Custom",
            }
        )
    }
}

/// Label selection options for phone dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneLabelOption {
    Mobile,
    Home,
    Work,
    Main,
    HomeFax,
    WorkFax,
    Pager,
    Other,
    Custom,
}

impl PhoneLabelOption {
    pub const ALL: [PhoneLabelOption; 9] = [
        PhoneLabelOption::Mobile,
        PhoneLabelOption::Home,
        PhoneLabelOption::Work,
        PhoneLabelOption::Main,
        PhoneLabelOption::HomeFax,
        PhoneLabelOption::WorkFax,
        PhoneLabelOption::Pager,
        PhoneLabelOption::Other,
        PhoneLabelOption::Custom,
    ];
}

impl std::fmt::Display for PhoneLabelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Mobile => "Mobile",
                Self::Home => "Home",
                Self::Work => "Work",
                Self::Main => "Main",
                Self::HomeFax => "Home Fax",
                Self::WorkFax => "Work Fax",
                Self::Pager => "Pager",
                Self::Other => "Other",
                Self::Custom => "Custom",
            }
        )
    }
}

/// Label selection options for address dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressLabelOption {
    Home,
    Work,
    Other,
    Custom,
}

impl AddressLabelOption {
    pub const ALL: [AddressLabelOption; 4] = [
        AddressLabelOption::Home,
        AddressLabelOption::Work,
        AddressLabelOption::Other,
        AddressLabelOption::Custom,
    ];
}

impl std::fmt::Display for AddressLabelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Home => "Home",
                Self::Work => "Work",
                Self::Other => "Other",
                Self::Custom => "Custom",
            }
        )
    }
}

/// Label selection options for date dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateLabelOption {
    Birthday,
    Anniversary,
    Other,
    Custom,
}

impl DateLabelOption {
    pub const ALL: [DateLabelOption; 4] = [
        DateLabelOption::Birthday,
        DateLabelOption::Anniversary,
        DateLabelOption::Other,
        DateLabelOption::Custom,
    ];
}

impl std::fmt::Display for DateLabelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Birthday => "Birthday",
                Self::Anniversary => "Anniversary",
                Self::Other => "Other",
                Self::Custom => "Custom",
            }
        )
    }
}

/// Label selection options for URL dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlLabelOption {
    HomePage,
    Work,
    Blog,
    Profile,
    GitHub,
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    Mastodon,
    Other,
    Custom,
}

impl UrlLabelOption {
    pub const ALL: [UrlLabelOption; 12] = [
        UrlLabelOption::HomePage,
        UrlLabelOption::Work,
        UrlLabelOption::Blog,
        UrlLabelOption::Profile,
        UrlLabelOption::GitHub,
        UrlLabelOption::LinkedIn,
        UrlLabelOption::Twitter,
        UrlLabelOption::Facebook,
        UrlLabelOption::Instagram,
        UrlLabelOption::Mastodon,
        UrlLabelOption::Other,
        UrlLabelOption::Custom,
    ];
}

impl std::fmt::Display for UrlLabelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::HomePage => "HomePage",
                Self::Work => "Work",
                Self::Blog => "Blog",
                Self::Profile => "Profile",
                Self::GitHub => "GitHub",
                Self::LinkedIn => "LinkedIn",
                Self::Twitter => "Twitter",
                Self::Facebook => "Facebook",
                Self::Instagram => "Instagram",
                Self::Mastodon => "Mastodon",
                Self::Other => "Other",
                Self::Custom => "Custom",
            }
        )
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
    NamePrefixChanged(String),
    FirstNameChanged(String),
    MiddleNameChanged(String),
    LastNameChanged(String),
    NameSuffixChanged(String),
    NicknameChanged(String),
    NotesChanged(String),
    OrganizationChanged(String),
    TitleChanged(String),
    DepartmentChanged(String),
    PhotoUrlChanged(String),
    
    // Multiple value field changes
    EmailChanged(usize, String),
    EmailLabelChanged(usize, String),
    EmailLabelSelected(usize, EmailLabelOption),
    EmailCustomLabelChanged(usize, String),
    PhoneChanged(usize, String),
    PhoneLabelChanged(usize, String),
    PhoneLabelSelected(usize, PhoneLabelOption),
    PhoneCustomLabelChanged(usize, String),
    UrlChanged(usize, String),
    UrlLabelChanged(usize, String),
    UrlLabelSelected(usize, UrlLabelOption),
    UrlCustomLabelChanged(usize, String),
    AddEmail,
    AddPhone,
    AddUrl,
    RemoveEmail(usize),
    RemovePhone(usize),
    RemoveUrl(usize),
    
    // Address changes
    AddressLabelChanged(usize, String),
    AddressLabelSelected(usize, AddressLabelOption),
    AddressCustomLabelChanged(usize, String),
    AddressStreetChanged(usize, String),
    AddressCityChanged(usize, String),
    AddressStateChanged(usize, String),
    AddressPostalCodeChanged(usize, String),
    AddressCountryChanged(usize, String),
    AddAddress,
    RemoveAddress(usize),
    
    // Date changes
    DateLabelChanged(usize, String),
    DateLabelSelected(usize, DateLabelOption),
    DateCustomLabelChanged(usize, String),
    DateValueChanged(usize, String),
    AddDate,
    RemoveDate(usize),
    
    // Custom field changes
    CustomFieldKeyChanged(usize, String),
    CustomFieldValueChanged(usize, String),
    AddCustomField,
    RemoveCustomField(usize),
    
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
    
    // Workspace management
    LoadWorkspaces,
    WorkspacesLoaded(Result<Vec<Workspace>, String>),
    SelectWorkspace(Uuid),
    WorkspaceSelected(Result<VcfRepository, String>),
    CreateNewWorkspace,
    NewWorkspaceNameChanged(String),
    CreateEmptyWorkspace,
    ImportVcfAsWorkspace,
    DeleteWorkspace(Uuid),
    BackToWorkspaceSelector,
    
    // UI
    ClearError,
}

/// Initialize the application state with workspace support
fn new() -> (State, Task<Message>) {
    // Get workspace root directory
    let proj_dirs = directories::ProjectDirs::from("com", "profile-pulse", "Profile Pulse")
        .expect("Failed to determine project directories");
    let data_dir = proj_dirs.data_dir().to_path_buf();
    let workspaces_dir = data_dir.join("workspaces");
    
    let workspace_manager = WorkspaceManager::new(workspaces_dir)
        .expect("Failed to create workspace manager");

    let state = State {
        workspace_manager,
        current_workspace: None,
        workspaces: Vec::new(),
        repository: None,
        current_view: View::WorkspaceSelector,
        contacts: Vec::new(),
        search_query: String::new(),
        form: ContactForm::new(),
        error_message: None,
        is_loading: false,
        current_page: 0,
        items_per_page: 50,
        new_workspace_name: String::new(),
    };

    (state, Task::perform(async {}, |_| Message::LoadWorkspaces))
}

/// Initialize with existing repository (legacy support)
pub fn new_with_repository(repository: VcfRepository) -> (State, Task<Message>) {
    // Get workspace root directory
    let proj_dirs = directories::ProjectDirs::from("com", "profile-pulse", "Profile Pulse")
        .expect("Failed to determine project directories");
    let data_dir = proj_dirs.data_dir().to_path_buf();
    let workspaces_dir = data_dir.join("workspaces");
    
    let workspace_manager = WorkspaceManager::new(workspaces_dir)
        .expect("Failed to create workspace manager");

    let state = State {
        workspace_manager,
        current_workspace: None,
        workspaces: Vec::new(),
        repository: Some(repository),
        current_view: View::List { letter_filter: None },
        contacts: Vec::new(),
        search_query: String::new(),
        form: ContactForm::new(),
        error_message: None,
        is_loading: false,
        current_page: 0,
        items_per_page: 50,
        new_workspace_name: String::new(),
    };

    (state, Task::perform(async {}, |_| Message::LoadContacts))
}

/// Handle application messages and update state
fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::LoadWorkspaces => {
            let manager = state.workspace_manager.clone();
            Task::perform(
                async move {
                    manager.load_workspaces().map_err(|e| e.to_string())
                },
                Message::WorkspacesLoaded,
            )
        }

        Message::WorkspacesLoaded(result) => {
            match result {
                Ok(workspaces) => {
                    state.workspaces = workspaces;
                    state.error_message = None;
                }
                Err(err) => {
                    state.error_message = Some(format!("Failed to load workspaces: {}", err));
                }
            }
            Task::none()
        }

        Message::SelectWorkspace(workspace_id) => {
            let workspace = state.workspaces.iter().find(|w| w.id == workspace_id).cloned();
            if let Some(mut workspace) = workspace {
                workspace.touch();
                let vcf_path = workspace.vcf_path.clone();
                
                // Update workspace in manager
                let manager = state.workspace_manager.clone();
                let _ = manager.update_workspace(&workspace);
                
                state.current_workspace = Some(workspace);
                state.is_loading = true;
                
                // Create VCF repository for this workspace
                Task::perform(
                    async move {
                        Ok(VcfRepository::new(vcf_path))
                    },
                    Message::WorkspaceSelected,
                )
            } else {
                state.error_message = Some("Workspace not found".to_string());
                Task::none()
            }
        }

        Message::WorkspaceSelected(result) => {
            state.is_loading = false;
            match result {
                Ok(repository) => {
                    state.repository = Some(repository);
                    state.current_view = View::List { letter_filter: None };
                    state.error_message = None;
                    // Load contacts from this workspace
                    Task::perform(async {}, |_| Message::LoadContacts)
                }
                Err(err) => {
                    state.error_message = Some(format!("Failed to initialize workspace: {}", err));
                    state.current_view = View::WorkspaceSelector;
                    Task::none()
                }
            }
        }

        Message::NewWorkspaceNameChanged(name) => {
            state.new_workspace_name = name;
            Task::none()
        }

        Message::CreateEmptyWorkspace => {
            if state.new_workspace_name.trim().is_empty() {
                state.error_message = Some("Workspace name cannot be empty".to_string());
                return Task::none();
            }
            
            let manager = state.workspace_manager.clone();
            let name = state.new_workspace_name.clone();
            state.new_workspace_name.clear();
            
            Task::perform(
                async move {
                    manager.create_empty_workspace(name).map_err(|e| e.to_string())
                },
                |result| match result {
                    Ok(_) => Message::LoadWorkspaces,
                    Err(err) => Message::ContactsLoaded(Err(err)),
                }
            )
        }

        Message::DeleteWorkspace(workspace_id) => {
            let manager = state.workspace_manager.clone();
            Task::perform(
                async move {
                    manager.delete_workspace(workspace_id).map_err(|e| e.to_string())
                },
                |result| match result {
                    Ok(_) => Message::LoadWorkspaces,
                    Err(err) => Message::ContactsLoaded(Err(err)),
                }
            )
        }

        Message::BackToWorkspaceSelector => {
            state.current_workspace = None;
            state.repository = None;
            state.contacts.clear();
            state.current_view = View::WorkspaceSelector;
            Task::perform(async {}, |_| Message::LoadWorkspaces)
        }

        Message::CreateNewWorkspace => {
            // TODO: Implement file picker to select VCF file
            state.error_message = Some("Create from VCF file not yet implemented".to_string());
            Task::none()
        }

        Message::ImportVcfAsWorkspace => {
            // TODO: Implement VCF import as new workspace
            state.error_message = Some("Import VCF as workspace not yet implemented".to_string());
            Task::none()
        }

        Message::LoadContacts => {
            if let Some(ref mut repo) = state.repository {
                state.is_loading = true;
                // Load all contacts synchronously
                match repo.list_all() {
                    Ok(contacts) => {
                        state.contacts = contacts;
                        state.is_loading = false;
                        state.error_message = None;
                    }
                    Err(err) => {
                        state.is_loading = false;
                        state.error_message = Some(err.to_string());
                    }
                }
            }
            Task::none()
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
        Message::NamePrefixChanged(value) => {
            state.form.name_prefix = value;
            Task::none()
        }
        Message::FirstNameChanged(value) => {
            state.form.first_name = value;
            Task::none()
        }
        Message::MiddleNameChanged(value) => {
            state.form.middle_name = value;
            Task::none()
        }
        Message::LastNameChanged(value) => {
            state.form.last_name = value;
            Task::none()
        }
        Message::NameSuffixChanged(value) => {
            state.form.name_suffix = value;
            Task::none()
        }
        Message::NicknameChanged(value) => {
            state.form.nickname = value;
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
                state.form.emails[index].email = value;
            }
            Task::none()
        }
        Message::EmailLabelChanged(index, value) => {
            if index < state.form.emails.len() {
                state.form.emails[index].label = value;
            }
            Task::none()
        }
        Message::EmailLabelSelected(index, option) => {
            if index < state.form.emails.len() {
                state.form.emails[index].selected_option = Some(option);
                // Update label based on selection
                let label = match option {
                    EmailLabelOption::Home => "Home".to_string(),
                    EmailLabelOption::Work => "Work".to_string(),
                    EmailLabelOption::Other => "Other".to_string(),
                    EmailLabelOption::Custom => state.form.emails[index].custom_label.clone(),
                };
                state.form.emails[index].label = label;
            }
            Task::none()
        }
        Message::EmailCustomLabelChanged(index, value) => {
            if index < state.form.emails.len() {
                state.form.emails[index].custom_label = value.clone();
                // If Custom is selected, update the label
                if state.form.emails[index].selected_option == Some(EmailLabelOption::Custom) {
                    state.form.emails[index].label = value;
                }
            }
            Task::none()
        }
        Message::PhoneChanged(index, value) => {
            if index < state.form.phones.len() {
                state.form.phones[index].phone = value;
            }
            Task::none()
        }
        Message::PhoneLabelChanged(index, value) => {
            if index < state.form.phones.len() {
                state.form.phones[index].label = value;
            }
            Task::none()
        }
        Message::PhoneLabelSelected(index, option) => {
            if index < state.form.phones.len() {
                state.form.phones[index].selected_option = Some(option);
                // Update label based on selection
                let label = match option {
                    PhoneLabelOption::Mobile => "Mobile".to_string(),
                    PhoneLabelOption::Home => "Home".to_string(),
                    PhoneLabelOption::Work => "Work".to_string(),
                    PhoneLabelOption::Main => "Main".to_string(),
                    PhoneLabelOption::HomeFax => "Home Fax".to_string(),
                    PhoneLabelOption::WorkFax => "Work Fax".to_string(),
                    PhoneLabelOption::Pager => "Pager".to_string(),
                    PhoneLabelOption::Other => "Other".to_string(),
                    PhoneLabelOption::Custom => state.form.phones[index].custom_label.clone(),
                };
                state.form.phones[index].label = label;
            }
            Task::none()
        }
        Message::PhoneCustomLabelChanged(index, value) => {
            if index < state.form.phones.len() {
                state.form.phones[index].custom_label = value.clone();
                // If Custom is selected, update the label
                if state.form.phones[index].selected_option == Some(PhoneLabelOption::Custom) {
                    state.form.phones[index].label = value;
                }
            }
            Task::none()
        }
        Message::UrlChanged(index, value) => {
            if index < state.form.urls.len() {
                state.form.urls[index].url = value;
            }
            Task::none()
        }
        Message::UrlLabelChanged(index, value) => {
            if index < state.form.urls.len() {
                state.form.urls[index].label = value;
            }
            Task::none()
        }
        Message::UrlLabelSelected(index, option) => {
            if index < state.form.urls.len() {
                state.form.urls[index].selected_option = Some(option);
                // Update label based on selection
                let label = match option {
                    UrlLabelOption::HomePage => "HomePage".to_string(),
                    UrlLabelOption::Work => "Work".to_string(),
                    UrlLabelOption::Blog => "Blog".to_string(),
                    UrlLabelOption::Profile => "Profile".to_string(),
                    UrlLabelOption::GitHub => "GitHub".to_string(),
                    UrlLabelOption::LinkedIn => "LinkedIn".to_string(),
                    UrlLabelOption::Twitter => "Twitter".to_string(),
                    UrlLabelOption::Facebook => "Facebook".to_string(),
                    UrlLabelOption::Instagram => "Instagram".to_string(),
                    UrlLabelOption::Mastodon => "Mastodon".to_string(),
                    UrlLabelOption::Other => "Other".to_string(),
                    UrlLabelOption::Custom => state.form.urls[index].custom_label.clone(),
                };
                state.form.urls[index].label = label;
            }
            Task::none()
        }
        Message::UrlCustomLabelChanged(index, value) => {
            if index < state.form.urls.len() {
                state.form.urls[index].custom_label = value.clone();
                // If Custom is selected, update the label
                if state.form.urls[index].selected_option == Some(UrlLabelOption::Custom) {
                    state.form.urls[index].label = value;
                }
            }
            Task::none()
        }
        Message::AddEmail => {
            state.form.emails.push(EmailForm {
                email: String::new(),
                label: EmailLabel::default().to_string_value(),
                selected_option: Some(EmailLabelOption::Home),
                custom_label: String::new(),
            });
            Task::none()
        }
        Message::AddPhone => {
            state.form.phones.push(PhoneForm {
                phone: String::new(),
                label: PhoneLabel::default().to_string_value(),
                selected_option: Some(PhoneLabelOption::Mobile),
                custom_label: String::new(),
            });
            Task::none()
        }
        Message::AddUrl => {
            state.form.urls.push(UrlForm {
                url: String::new(),
                label: String::new(),
                selected_option: Some(UrlLabelOption::Other),
                custom_label: String::new(),
            });
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
        
        // Address changes
        Message::AddressLabelChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].label = value;
            }
            Task::none()
        }
        Message::AddressLabelSelected(index, option) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].selected_option = Some(option);
                // Update label based on selection
                let label = match option {
                    AddressLabelOption::Home => "Home".to_string(),
                    AddressLabelOption::Work => "Work".to_string(),
                    AddressLabelOption::Other => "Other".to_string(),
                    AddressLabelOption::Custom => state.form.addresses[index].custom_label.clone(),
                };
                state.form.addresses[index].label = label;
            }
            Task::none()
        }
        Message::AddressCustomLabelChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].custom_label = value.clone();
                // If Custom is selected, update the label
                if state.form.addresses[index].selected_option == Some(AddressLabelOption::Custom) {
                    state.form.addresses[index].label = value;
                }
            }
            Task::none()
        }
        Message::AddressStreetChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].street = value;
            }
            Task::none()
        }
        Message::AddressCityChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].city = value;
            }
            Task::none()
        }
        Message::AddressStateChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].state = value;
            }
            Task::none()
        }
        Message::AddressPostalCodeChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].postal_code = value;
            }
            Task::none()
        }
        Message::AddressCountryChanged(index, value) => {
            if index < state.form.addresses.len() {
                state.form.addresses[index].country = value;
            }
            Task::none()
        }
        Message::AddAddress => {
            state.form.addresses.push(AddressForm {
                label: AddressLabel::default().to_string_value(),
                selected_option: Some(AddressLabelOption::Home),
                custom_label: String::new(),
                ..Default::default()
            });
            Task::none()
        }
        Message::RemoveAddress(index) => {
            if state.form.addresses.len() > 1 {
                state.form.addresses.remove(index);
            }
            Task::none()
        }
        
        // Date changes
        Message::DateLabelChanged(index, value) => {
            if index < state.form.dates.len() {
                state.form.dates[index].label = value;
            }
            Task::none()
        }
        Message::DateLabelSelected(index, option) => {
            if index < state.form.dates.len() {
                state.form.dates[index].selected_option = Some(option);
                // Update label based on selection
                let label = match option {
                    DateLabelOption::Birthday => "Birthday".to_string(),
                    DateLabelOption::Anniversary => "Anniversary".to_string(),
                    DateLabelOption::Other => "Other".to_string(),
                    DateLabelOption::Custom => state.form.dates[index].custom_label.clone(),
                };
                state.form.dates[index].label = label;
            }
            Task::none()
        }
        Message::DateCustomLabelChanged(index, value) => {
            if index < state.form.dates.len() {
                state.form.dates[index].custom_label = value.clone();
                // If Custom is selected, update the label
                if state.form.dates[index].selected_option == Some(DateLabelOption::Custom) {
                    state.form.dates[index].label = value;
                }
            }
            Task::none()
        }
        Message::DateValueChanged(index, value) => {
            if index < state.form.dates.len() {
                state.form.dates[index].date = value;
            }
            Task::none()
        }
        Message::AddDate => {
            state.form.dates.push(DateForm {
                label: DateLabel::default().to_string_value(),
                date: String::new(),
                selected_option: Some(DateLabelOption::Birthday),
                custom_label: String::new(),
            });
            Task::none()
        }
        Message::RemoveDate(index) => {
            if state.form.dates.len() > 1 {
                state.form.dates.remove(index);
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

            let contact_result = state.form.to_contact(None);
            
            match contact_result {
                Ok(contact) => {
                    if let Some(ref mut repo) = state.repository {
                        match repo.create(contact.clone()) {
                            Ok(_) => {
                                state.contacts.push(contact);
                                state.form.clear();
                                state.current_view = View::List { letter_filter: None };
                                state.error_message = None;
                            }
                            Err(err) => {
                                state.error_message = Some(format!("Failed to save contact: {}", err));
                            }
                        }
                    } else {
                        state.error_message = Some("No workspace selected".to_string());
                    }
                }
                Err(e) => {
                    state.error_message = Some(format!("Failed to build contact: {}", e));
                }
            }
            Task::none()
        }

        Message::ContactSaved(_result) => {
            // No longer used - keeping for compatibility
            Task::none()
        }

        Message::UpdateContact(contact_id) => {
            if !state.form.is_valid() {
                state.error_message = Some("Name is required".to_string());
                return Task::none();
            }

            let contact_result = state.form.to_contact(Some(contact_id));
            
            match contact_result {
                Ok(contact) => {
                    if let Some(ref mut repo) = state.repository {
                        match repo.update(contact.clone()) {
                            Ok(_) => {
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
                    } else {
                        state.error_message = Some("No workspace selected".to_string());
                    }
                }
                Err(e) => {
                    state.error_message = Some(format!("Failed to build contact: {}", e));
                }
            }
            Task::none()
        }

        Message::ContactUpdated(_result) => {
            // No longer used - keeping for compatibility
            Task::none()
        }

        Message::DeleteContact(contact_id) => {
            if let Some(ref mut repo) = state.repository {
                match repo.delete(contact_id) {
                    Ok(_) => {
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
            } else {
                state.error_message = Some("No workspace selected".to_string());
            }
            Task::none()
        }

        Message::ContactDeleted(_result) => {
            // No longer used - keeping for compatibility
            Task::none()
        }

        Message::ClearError => {
            state.error_message = None;
            Task::none()
        }

        Message::ImportVcf => {
            if let Some(ref mut _repo) = state.repository {
                Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("vCard", &["vcf"])
                            .pick_file()
                            .await;
                        
                        if let Some(file) = file {
                            let path = file.path().to_path_buf();
                            match crate::vcf::import_from_file(&path) {
                                Ok(contacts) => Ok(contacts),
                                Err(e) => Err(format!("Failed to import VCF: {}", e)),
                            }
                        } else {
                            Err("No file selected".to_string())
                        }
                    },
                    Message::VcfImported,
                )
            } else {
                state.error_message = Some("No workspace selected".to_string());
                Task::none()
            }
        }

        Message::VcfImported(result) => {
            match result {
                Ok(contacts) => {
                    if let Some(ref mut repo) = state.repository {
                        let mut imported = 0;
                        for contact in contacts {
                            if let Ok(_) = repo.create(contact) {
                                imported += 1;
                            }
                        }
                        // Reload contacts from VCF
                        if let Ok(all_contacts) = repo.list_all() {
                            state.contacts = all_contacts;
                        }
                        state.error_message = Some(format!("✅ Successfully imported {} contacts", imported));
                    }
                }
                Err(err) => {
                    if err != "No file selected" {
                        state.error_message = Some(err);
                    }
                }
            }
            Task::none()
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
        View::WorkspaceSelector => view_workspace_selector(state),
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

/// Render workspace selector view
fn view_workspace_selector(state: &State) -> Element<'_, Message> {
    let title = text("Select or Create a Workspace").size(32);
    let subtitle = text("Each workspace manages its own VCF file and database").size(14);

    let mut content = Column::new().spacing(20).padding(20);
    content = content.push(title);
    content = content.push(subtitle);

    // Create new workspace section
    content = content.push(text("Create New Workspace").size(20));
    let new_workspace_row = row![
        text_input("Workspace name (e.g., 'Personal', 'Work')", &state.new_workspace_name)
            .on_input(Message::NewWorkspaceNameChanged)
            .padding(10)
            .width(Length::FillPortion(3)),
        button("Create Empty Workspace")
            .on_press(Message::CreateEmptyWorkspace)
            .padding(10),
    ]
    .spacing(10);
    content = content.push(new_workspace_row);

    // Existing workspaces section
    content = content.push(text("Existing Workspaces").size(20));

    if state.workspaces.is_empty() {
        content = content.push(text("No workspaces yet. Create one above!").size(14));
    } else {
        for workspace in &state.workspaces {
            let workspace_card = container(
                column![
                    row![
                        text(&workspace.name).size(18),
                        Space::new(),
                        text(format!("{} contacts", workspace.contact_count)).size(14),
                    ]
                    .spacing(10),
                    text(format!("VCF: {}", workspace.vcf_path.display())).size(12),
                    text(format!("Last accessed: {}", workspace.last_accessed.format("%Y-%m-%d %H:%M"))).size(12),
                    row![
                        button("Open").on_press(Message::SelectWorkspace(workspace.id)),
                        button("Delete").on_press(Message::DeleteWorkspace(workspace.id)),
                    ]
                    .spacing(10),
                ]
                .spacing(5)
                .padding(15),
            )
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.95, 0.95))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });

            content = content.push(workspace_card);
        }
    }

    scrollable(content).into()
}

/// Render contact list view with alphabetical pagination
fn view_list(state: &State, letter_filter: Option<char>) -> Element<'_, Message> {
    let workspace_name = state.current_workspace.as_ref()
        .map(|w| w.name.clone())
        .unwrap_or_else(|| "No Workspace".to_string());
    
    let title = text(format!("Profile Pulse - {}", workspace_name)).size(32);
    let add_button = button("+ Add Contact").on_press(Message::ChangeView(View::Add));
    let import_button = button("📥 Import VCF").on_press(Message::ImportVcf);
    let export_button = button("📤 Export VCF").on_press(Message::ExportVcf);
    let workspace_button = button("📁 Workspaces").on_press(Message::BackToWorkspaceSelector);

    let search_input = text_input("Search contacts...", &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(10);

    let header = row![title, add_button, import_button, export_button, workspace_button].spacing(10);

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
    if !contact.urls.is_empty() {
        detail_parts.push(format!("🔗 {} URLs", contact.urls.len()));
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
    
    form_content = form_content.push(text("Name").size(14));
    
    // Name prefix and suffix row
    let name_prefix_suffix_row = row![
        text_input("Prefix (Dr., Mr., Ms.)", &state.form.name_prefix)
            .on_input(Message::NamePrefixChanged)
            .padding(10)
            .width(Length::FillPortion(1)),
        text_input("Suffix (Jr., Sr., III)", &state.form.name_suffix)
            .on_input(Message::NameSuffixChanged)
            .padding(10)
            .width(Length::FillPortion(1)),
    ].spacing(10);
    form_content = form_content.push(name_prefix_suffix_row);
    
    // First, middle, last name row
    let name_row = row![
        text_input("First name*", &state.form.first_name)
            .on_input(Message::FirstNameChanged)
            .padding(10)
            .width(Length::FillPortion(2)),
        text_input("Middle name", &state.form.middle_name)
            .on_input(Message::MiddleNameChanged)
            .padding(10)
            .width(Length::FillPortion(1)),
        text_input("Last name*", &state.form.last_name)
            .on_input(Message::LastNameChanged)
            .padding(10)
            .width(Length::FillPortion(2)),
    ].spacing(10);
    form_content = form_content.push(name_row);

    form_content = form_content.push(text("Nickname").size(14));
    form_content = form_content.push(
        text_input("Nickname", &state.form.nickname)
            .on_input(Message::NicknameChanged)
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
        let mut email_section = Column::new().spacing(5);
        
        // Email input and label dropdown in one row
        let mut email_row = Row::new().spacing(5);
        email_row = email_row.push(
            text_input(&format!("Email {}", i + 1), &email.email)
                .on_input(move |v| Message::EmailChanged(i, v))
                .padding(10)
                .width(Length::FillPortion(3))
        );
        
        // Label dropdown
        email_row = email_row.push(
            pick_list(
                &EmailLabelOption::ALL[..],
                email.selected_option,
                move |option| Message::EmailLabelSelected(i, option)
            )
            .padding(10)
            .width(Length::FillPortion(1))
        );
        
        if state.form.emails.len() > 1 {
            email_row = email_row.push(button("−").on_press(Message::RemoveEmail(i)));
        }
        
        email_section = email_section.push(email_row);
        
        // Show custom label input if "Custom" is selected
        if email.selected_option == Some(EmailLabelOption::Custom) {
            email_section = email_section.push(
                text_input("Custom label", &email.custom_label)
                    .on_input(move |v| Message::EmailCustomLabelChanged(i, v))
                    .padding(8)
            );
        }
        
        form_content = form_content.push(email_section);
    }

    // Phones
    form_content = form_content.push(
        row![
            text("Phone Numbers").size(14),
            button("+ Add Phone").on_press(Message::AddPhone)
        ].spacing(10)
    );
    
    for (i, phone) in state.form.phones.iter().enumerate() {
        let mut phone_section = Column::new().spacing(5);
        
        // Phone input and label dropdown in one row
        let mut phone_row = Row::new().spacing(5);
        phone_row = phone_row.push(
            text_input(&format!("Phone {}", i + 1), &phone.phone)
                .on_input(move |v| Message::PhoneChanged(i, v))
                .padding(10)
                .width(Length::FillPortion(3))
        );
        
        // Label dropdown
        phone_row = phone_row.push(
            pick_list(
                &PhoneLabelOption::ALL[..],
                phone.selected_option,
                move |option| Message::PhoneLabelSelected(i, option)
            )
            .padding(10)
            .width(Length::FillPortion(1))
        );
        
        if state.form.phones.len() > 1 {
            phone_row = phone_row.push(button("−").on_press(Message::RemovePhone(i)));
        }
        
        phone_section = phone_section.push(phone_row);
        
        // Show custom label input if "Custom" is selected
        if phone.selected_option == Some(PhoneLabelOption::Custom) {
            phone_section = phone_section.push(
                text_input("Custom label", &phone.custom_label)
                    .on_input(move |v| Message::PhoneCustomLabelChanged(i, v))
                    .padding(8)
            );
        }
        
        form_content = form_content.push(phone_section);
    }

    // URLs (for profile pictures and websites)
    form_content = form_content.push(
        row![
            text("URLs (websites, profile pictures)").size(14),
            button("+ Add URL").on_press(Message::AddUrl)
        ].spacing(10)
    );
    form_content = form_content.push(text("Note: First URL will be used as profile picture source").size(12));
    
    for (i, url_form) in state.form.urls.iter().enumerate() {
        let mut url_section = Column::new().spacing(5);
        
        // URL input and label dropdown in one row
        let mut url_row = Row::new().spacing(5);
        url_row = url_row.push(
            text_input(&format!("URL {}", i + 1), &url_form.url)
                .on_input(move |v| Message::UrlChanged(i, v))
                .padding(10)
                .width(Length::FillPortion(3))
        );
        
        // Label dropdown
        url_row = url_row.push(
            pick_list(
                &UrlLabelOption::ALL[..],
                url_form.selected_option,
                move |option| Message::UrlLabelSelected(i, option)
            )
            .padding(10)
            .width(Length::FillPortion(1))
        );
        
        if state.form.urls.len() > 1 {
            url_row = url_row.push(button("−").on_press(Message::RemoveUrl(i)));
        } else {
            url_row = url_row.push(button("−"));
        }
        
        url_section = url_section.push(url_row);
        
        // Show custom label input if "Custom" is selected
        if url_form.selected_option == Some(UrlLabelOption::Custom) {
            url_section = url_section.push(
                text_input("Custom label", &url_form.custom_label)
                    .on_input(move |v| Message::UrlCustomLabelChanged(i, v))
                    .padding(8)
            );
        }
        
        form_content = form_content.push(url_section);
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
        
        // Label dropdown
        addr_section = addr_section.push(
            pick_list(
                &AddressLabelOption::ALL[..],
                addr.selected_option,
                move |option| Message::AddressLabelSelected(i, option)
            )
            .padding(8)
        );
        
        // Show custom label input if "Custom" is selected
        if addr.selected_option == Some(AddressLabelOption::Custom) {
            addr_section = addr_section.push(
                text_input("Custom label", &addr.custom_label)
                    .on_input(move |v| Message::AddressCustomLabelChanged(i, v))
                    .padding(8)
            );
        }
        
        // Street
        addr_section = addr_section.push(
            text_input("Street address", &addr.street)
                .on_input(move |v| Message::AddressStreetChanged(i, v))
                .padding(8)
        );
        
        // City, State, Postal Code
        let city_state_row = row![
            text_input("City", &addr.city)
                .on_input(move |v| Message::AddressCityChanged(i, v))
                .padding(8)
                .width(Length::FillPortion(2)),
            text_input("State", &addr.state)
                .on_input(move |v| Message::AddressStateChanged(i, v))
                .padding(8)
                .width(Length::FillPortion(1)),
            text_input("Postal Code", &addr.postal_code)
                .on_input(move |v| Message::AddressPostalCodeChanged(i, v))
                .padding(8)
                .width(Length::FillPortion(1)),
        ].spacing(5);
        addr_section = addr_section.push(city_state_row);
        
        // Country
        addr_section = addr_section.push(
            text_input("Country", &addr.country)
                .on_input(move |v| Message::AddressCountryChanged(i, v))
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

    // Significant Dates Section
    form_content = form_content.push(text("Significant Dates").size(18));
    form_content = form_content.push(text("(Birthdays, anniversaries, etc.)").size(12));
    
    for (i, date) in state.form.dates.iter().enumerate() {
        let mut date_section = Column::new().spacing(5);
        
        // Label dropdown and date input in one row
        let mut date_row = Row::new().spacing(5);
        date_row = date_row.push(
            pick_list(
                &DateLabelOption::ALL[..],
                date.selected_option,
                move |option| Message::DateLabelSelected(i, option)
            )
            .padding(8)
            .width(Length::FillPortion(1))
        );
        date_row = date_row.push(
            text_input("Date (YYYY-MM-DD)", &date.date)
                .on_input(move |v| Message::DateValueChanged(i, v))
                .padding(8)
                .width(Length::FillPortion(2))
        );
        if state.form.dates.len() > 1 {
            date_row = date_row.push(button("−").on_press(Message::RemoveDate(i)));
        }
        
        date_section = date_section.push(date_row);
        
        // Show custom label input if "Custom" is selected
        if date.selected_option == Some(DateLabelOption::Custom) {
            date_section = date_section.push(
                text_input("Custom label", &date.custom_label)
                    .on_input(move |v| Message::DateCustomLabelChanged(i, v))
                    .padding(8)
            );
        }
        
        form_content = form_content.push(date_section);
    }
    
    form_content = form_content.push(
        button("+ Add Date").on_press(Message::AddDate)
            .padding(10)
    );
    
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
    form_content = form_content.push(text("Additional information about this contact").size(12));
    
    // Create a larger notes input area with multiple instances for visual height
    // Note: Iced's text_input doesn't support multiline natively, but we can make it visually larger
    form_content = form_content.push(
        container(
            text_input("Type your notes here...", &state.form.notes)
                .on_input(Message::NotesChanged)
                .padding(15)
                .width(Length::Fill)
        )
        .padding(5)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.98, 0.98, 0.98))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.7, 0.7, 0.7),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
    );

    // Action buttons
    let save_button = if state.form.is_valid() {
        if let Some(id) = contact_id {
            button("💾 Update Contact").on_press(Message::UpdateContact(id))
        } else {
            button("💾 Save Contact").on_press(Message::SaveNewContact)
        }
    } else {
        button("💾 Save Contact (First or Last name required)")
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

    // URLs
    if !contact.urls.is_empty() {
        details = details.push(text("URLs").size(16));
        for url_obj in &contact.urls {
            let label_text = url_obj.label.as_ref().map(|l| format!("{}: ", l)).unwrap_or_default();
            details = details.push(
                text(format!("  {}{}", label_text, url_obj.url)).size(13)
            );
        }
        details = details.push(Space::new());
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
pub fn run_with_repository(repository: VcfRepository) -> Result<(), iced::Error> {
    iced::application(
        move || new_with_repository(repository.clone()),
        update,
        view,
    )
    .theme(theme)
    .run()
}