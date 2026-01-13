//! User interface module for Profile Pulse
//!
//! Contains Iced GUI components and views.

use crate::core::contact::{Contact, ContactBuilder, SocialPlatform};
use crate::db::repository::ContactRepository;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Column},
    Element, Length, Task, Theme,
};
use uuid::Uuid;

/// Current view in the application
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    /// Main contact list view
    List,
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
}

/// Form state for adding/editing contacts
#[derive(Debug, Clone, Default)]
pub struct ContactForm {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub organization: String,
    pub title: String,
}

impl ContactForm {
    fn clear(&mut self) {
        self.name.clear();
        self.email.clear();
        self.phone.clear();
        self.organization.clear();
        self.title.clear();
    }

    fn from_contact(contact: &Contact) -> Self {
        Self {
            name: contact.name.clone(),
            email: contact.email.clone().unwrap_or_default(),
            phone: contact.phone.clone().unwrap_or_default(),
            organization: contact.organization.clone().unwrap_or_default(),
            title: contact.title.clone().unwrap_or_default(),
        }
    }

    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

/// Application messages
#[derive(Debug, Clone)]
pub enum Message {
    /// Load all contacts from database
    LoadContacts,
    /// Contacts loaded successfully
    ContactsLoaded(Result<Vec<Contact>, String>),
    /// Switch to a different view
    ChangeView(View),
    /// Update search query
    SearchChanged(String),
    /// Form field updates
    NameChanged(String),
    EmailChanged(String),
    PhoneChanged(String),
    OrganizationChanged(String),
    TitleChanged(String),
    /// Save new contact
    SaveNewContact,
    /// Contact saved successfully
    ContactSaved(Result<Contact, String>),
    /// Update existing contact
    UpdateContact(Uuid),
    /// Contact updated successfully
    ContactUpdated(Result<Contact, String>),
    /// Delete contact
    DeleteContact(Uuid),
    /// Contact deleted successfully
    ContactDeleted(Result<Uuid, String>),
    /// Clear error message
    ClearError,
    /// Import VCF file
    ImportVcf,
    /// VCF imported successfully
    VcfImported(Result<Vec<Contact>, String>),
    /// Export all contacts to VCF
    ExportVcf,
    /// VCF exported successfully
    VcfExported(Result<String, String>),
}

/// Initialize the application state
fn new() -> (State, Task<Message>) {
    // Note: This should not be used - always use new_with_repository
    panic!("Use new_with_repository() instead of new()");
}

/// Initialize with existing repository
pub fn new_with_repository(repository: ContactRepository) -> (State, Task<Message>) {
    let state = State {
        repository,
        current_view: View::List,
        contacts: Vec::new(),
        search_query: String::new(),
        form: ContactForm::default(),
        error_message: None,
        is_loading: false,
    };

    (state, Task::perform(async {}, |_| Message::LoadContacts))
}

/// Handle application messages and update state
fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::LoadContacts => {
            state.is_loading = true;
            let repo = state.repository.clone();
            Task::perform(
                async move {
                    repo.list(Some(100), Some(0)).await.map_err(|e| e.to_string())
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
            } else if view == View::Add {
                state.form.clear();
            }
            state.current_view = view;
            Task::none()
        }

        Message::SearchChanged(query) => {
            state.search_query = query;
            Task::none()
        }

        Message::NameChanged(value) => {
            state.form.name = value;
            Task::none()
        }

        Message::EmailChanged(value) => {
            state.form.email = value;
            Task::none()
        }

        Message::PhoneChanged(value) => {
            state.form.phone = value;
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

        Message::SaveNewContact => {
            if !state.form.is_valid() {
                state.error_message = Some("Name is required".to_string());
                return Task::none();
            }

            let repo = state.repository.clone();
            let form = state.form.clone();

            Task::perform(
                async move {
                    let mut builder = ContactBuilder::new().name(&form.name);

                    if !form.email.is_empty() {
                        builder = builder.email(&form.email);
                    }
                    if !form.phone.is_empty() {
                        builder = builder.phone(&form.phone);
                    }
                    if !form.organization.is_empty() {
                        builder = builder.organization(&form.organization);
                    }
                    if !form.title.is_empty() {
                        builder = builder.title(&form.title);
                    }

                    let contact = builder.build().map_err(|e| e.to_string())?;
                    repo.create(&contact).await.map_err(|e| format!("{:?}", e))?;
                    Ok(contact)
                },
                Message::ContactSaved,
            )
        }

        Message::ContactSaved(result) => {
            match result {
                Ok(contact) => {
                    state.contacts.push(contact);
                    state.form.clear();
                    state.current_view = View::List;
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
            let form = state.form.clone();

            Task::perform(
                async move {
                    let mut builder = ContactBuilder::new().name(&form.name);

                    if !form.email.is_empty() {
                        builder = builder.email(&form.email);
                    }
                    if !form.phone.is_empty() {
                        builder = builder.phone(&form.phone);
                    }
                    if !form.organization.is_empty() {
                        builder = builder.organization(&form.organization);
                    }
                    if !form.title.is_empty() {
                        builder = builder.title(&form.title);
                    }

                    let mut contact = builder.build().map_err(|e| e.to_string())?;
                    contact.id = contact_id;

                    repo.update(&contact).await.map_err(|e| format!("{:?}", e))?;
                    Ok(contact)
                },
                Message::ContactUpdated,
            )
        }

        Message::ContactUpdated(result) => {
            match result {
                Ok(contact) => {
                    if let Some(idx) = state.contacts.iter().position(|c| c.id == contact.id) {
                        state.contacts[idx] = contact;
                    }
                    state.form.clear();
                    state.current_view = View::List;
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
                        state.current_view = View::List;
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
            // Open file picker and import VCF
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
                                // Save contacts to database
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
                    state.error_message = Some(format!("Successfully imported {} contacts", contacts.len()));
                    // Reload contacts from database to get correct IDs
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
                    state.error_message = Some(msg);
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
        View::List => view_list(state),
        View::Add => view_add_form(state),
        View::Edit(id) => view_edit_form(state, *id),
        View::Detail(id) => view_detail(state, *id),
    };

    let mut layout = column![].spacing(10).padding(20);

    // Error message banner
    if let Some(error) = &state.error_message {
        layout = layout.push(
            container(
                row![
                    text(error).style(|_theme| text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.2, 0.2)),
                    }),
                    button("✕").on_press(Message::ClearError),
                ]
                .spacing(10),
            )
            .padding(10)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    1.0, 0.9, 0.9,
                ))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.8, 0.2, 0.2),
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

/// Render contact list view
fn view_list(state: &State) -> Element<'_, Message> {
    let title = text("Profile Pulse").size(32);
    let add_button = button("+ Add Contact").on_press(Message::ChangeView(View::Add));
    let import_button = button("📥 Import VCF").on_press(Message::ImportVcf);
    let export_button = button("📤 Export VCF").on_press(Message::ExportVcf);

    let search_input = text_input("Search contacts...", &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(10);

    let header = row![title, add_button, import_button, export_button].spacing(10);

    // Filter contacts by search query
    let filtered_contacts: Vec<&Contact> = state
        .contacts
        .iter()
        .filter(|contact| {
            if state.search_query.is_empty() {
                true
            } else {
                let query = state.search_query.to_lowercase();
                contact.name.to_lowercase().contains(&query)
                    || contact
                        .email
                        .as_ref()
                        .map(|e| e.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || contact
                        .organization
                        .as_ref()
                        .map(|o| o.to_lowercase().contains(&query))
                        .unwrap_or(false)
            }
        })
        .collect();

    let contact_count = text(format!("{} contacts", filtered_contacts.len()));

    let mut contact_list = Column::new().spacing(5);

    if filtered_contacts.is_empty() {
        contact_list = contact_list.push(text("No contacts found").size(16));
    } else {
        for contact in filtered_contacts {
            contact_list = contact_list.push(view_contact_item(contact));
        }
    }

    column![
        header,
        search_input,
        contact_count,
        scrollable(contact_list).height(Length::Fill)
    ]
    .spacing(20)
    .into()
}

/// Render a single contact item in the list
fn view_contact_item(contact: &Contact) -> Element<'_, Message> {
    let name = text(&contact.name).size(18);
    let mut info_parts = Vec::new();

    if let Some(email) = &contact.email {
        info_parts.push(email.clone());
    }
    if let Some(org) = &contact.organization {
        info_parts.push(org.clone());
    }

    let info = text(info_parts.join(" • ")).size(14);

    let social_count = if contact.social_profiles.is_empty() {
        text("")
    } else {
        text(format!("🔗 {} profiles", contact.social_profiles.len())).size(12)
    };

    let view_button = button("View").on_press(Message::ChangeView(View::Detail(contact.id)));
    let edit_button = button("Edit").on_press(Message::ChangeView(View::Edit(contact.id)));
    let delete_button = button("Delete").on_press(Message::DeleteContact(contact.id));

    container(
        row![
            column![name, info, social_count].spacing(5).width(Length::Fill),
            row![view_button, edit_button, delete_button].spacing(5)
        ]
        .padding(10),
    )
    .padding(10)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgb(
            0.95, 0.95, 0.95,
        ))),
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
    let title = text("Add New Contact").size(24);
    let back_button = button("← Back to List").on_press(Message::ChangeView(View::List));

    let name_input = text_input("Name (required)", &state.form.name)
        .on_input(Message::NameChanged)
        .padding(10);

    let email_input = text_input("Email", &state.form.email)
        .on_input(Message::EmailChanged)
        .padding(10);

    let phone_input = text_input("Phone", &state.form.phone)
        .on_input(Message::PhoneChanged)
        .padding(10);

    let org_input = text_input("Organization", &state.form.organization)
        .on_input(Message::OrganizationChanged)
        .padding(10);

    let title_input = text_input("Title", &state.form.title)
        .on_input(Message::TitleChanged)
        .padding(10);

    let save_button = if state.form.is_valid() {
        button("Save Contact").on_press(Message::SaveNewContact)
    } else {
        button("Save Contact")
    };

    let cancel_button = button("Cancel").on_press(Message::ChangeView(View::List));

    column![
        row![title, back_button].spacing(20),
        text("Name").size(14),
        name_input,
        text("Email").size(14),
        email_input,
        text("Phone").size(14),
        phone_input,
        text("Organization").size(14),
        org_input,
        text("Job Title").size(14),
        title_input,
        row![save_button, cancel_button].spacing(10),
    ]
    .spacing(10)
    .into()
}

/// Render edit contact form
fn view_edit_form(state: &State, contact_id: Uuid) -> Element<'_, Message> {
    let title = text("Edit Contact").size(24);
    let back_button = button("← Back to List").on_press(Message::ChangeView(View::List));

    let name_input = text_input("Name (required)", &state.form.name)
        .on_input(Message::NameChanged)
        .padding(10);

    let email_input = text_input("Email", &state.form.email)
        .on_input(Message::EmailChanged)
        .padding(10);

    let phone_input = text_input("Phone", &state.form.phone)
        .on_input(Message::PhoneChanged)
        .padding(10);

    let org_input = text_input("Organization", &state.form.organization)
        .on_input(Message::OrganizationChanged)
        .padding(10);

    let title_input = text_input("Title", &state.form.title)
        .on_input(Message::TitleChanged)
        .padding(10);

    let save_button = if state.form.is_valid() {
        button("Update Contact").on_press(Message::UpdateContact(contact_id))
    } else {
        button("Update Contact")
    };

    let cancel_button = button("Cancel").on_press(Message::ChangeView(View::List));

    column![
        row![title, back_button].spacing(20),
        text("Name").size(14),
        name_input,
        text("Email").size(14),
        email_input,
        text("Phone").size(14),
        phone_input,
        text("Organization").size(14),
        org_input,
        text("Job Title").size(14),
        title_input,
        row![save_button, cancel_button].spacing(10),
    ]
    .spacing(10)
    .into()
}

/// Render contact detail view
fn view_detail(state: &State, contact_id: Uuid) -> Element<'_, Message> {
    let contact = state.contacts.iter().find(|c| c.id == contact_id);

    if contact.is_none() {
        return column![
            text("Contact not found").size(24),
            button("← Back to List").on_press(Message::ChangeView(View::List))
        ]
        .spacing(20)
        .into();
    }

    let contact = contact.unwrap();

    let title = text(&contact.name).size(28);
    let back_button = button("← Back to List").on_press(Message::ChangeView(View::List));
    let edit_button = button("Edit").on_press(Message::ChangeView(View::Edit(contact.id)));
    let delete_button = button("Delete").on_press(Message::DeleteContact(contact.id));

    let mut details = Column::new().spacing(10);

    if let Some(email) = &contact.email {
        details = details.push(row![text("Email:").size(14), text(email).size(14)].spacing(10));
    }

    if let Some(phone) = &contact.phone {
        details = details.push(row![text("Phone:").size(14), text(phone).size(14)].spacing(10));
    }

    if let Some(org) = &contact.organization {
        details = details.push(
            row![text("Organization:").size(14), text(org).size(14)].spacing(10),
        );
    }

    if let Some(title_text) = &contact.title {
        details = details.push(row![text("Title:").size(14), text(title_text).size(14)].spacing(10));
    }

    let mut social_section = Column::new().spacing(5);
    if !contact.social_profiles.is_empty() {
        social_section = social_section.push(text("Social Profiles:").size(16));
        for profile in &contact.social_profiles {
            social_section = social_section.push(
                text(format!("  {} - @{}", platform_emoji(&profile.platform), profile.username))
                    .size(14),
            );
        }
    }

    column![
        row![title, back_button, edit_button, delete_button].spacing(10),
        details,
        social_section,
        text(format!("Created: {}", contact.created_at.format("%Y-%m-%d %H:%M"))).size(12),
        text(format!("Updated: {}", contact.updated_at.format("%Y-%m-%d %H:%M"))).size(12),
    ]
    .spacing(15)
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