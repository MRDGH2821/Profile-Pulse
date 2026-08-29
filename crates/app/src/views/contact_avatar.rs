use crate::state::AppState;
use dioxus::prelude::*;
use profile_pulse_pic_source_plugin_host::guess_content_type;
use profile_pulse_storage::load_photo;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ContactAvatarSize {
    #[default]
    Md,
    Lg,
}

impl ContactAvatarSize {
    fn class_name(self) -> &'static str {
        match self {
            Self::Md => "contact-avatar-md",
            Self::Lg => "contact-avatar-lg",
        }
    }
}

pub async fn photo_data_url(data_root: &Path, hash: &str) -> Result<String, String> {
    let bytes = load_photo(data_root, hash)
        .await
        .map_err(|e| e.to_string())?;
    let mime = guess_content_type(&bytes);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}

fn contact_initial(display_name: &str) -> String {
    display_name
        .chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn ContactAvatar(
    photo_hash: Option<String>,
    display_name: String,
    #[props(default)] size: ContactAvatarSize,
) -> Element {
    let state = use_context::<AppState>();
    let mut data_url = use_signal(|| None::<String>);
    let size_class = size.class_name();
    use_effect(move || {
        let hash = photo_hash.clone();
        let data_root = state.contact_service.data_root().to_path_buf();
        spawn(async move {
            if let Some(hash) = hash {
                match photo_data_url(&data_root, &hash).await {
                    Ok(url) => data_url.set(Some(url)),
                    Err(_) => data_url.set(None),
                }
            } else {
                data_url.set(None);
            }
        });
    });
    if let Some(url) = data_url() {
        rsx! {
            img {
                class: "contact-avatar {size_class}",
                src: "{url}",
                alt: "Profile picture for {display_name}",
            }
        }
    } else {
        rsx! {
            span {
                class: "contact-avatar contact-avatar-fallback {size_class}",
                aria_hidden: "true",
                "{contact_initial(&display_name)}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::contact_initial;

    #[test]
    fn initial_uses_first_alphanumeric_char() {
        assert_eq!(contact_initial("Alice Smith"), "A");
        assert_eq!(contact_initial("  bob"), "B");
        assert_eq!(contact_initial("!!!"), "?");
    }
}
