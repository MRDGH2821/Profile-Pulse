pub mod github;
pub mod gitlab;
pub mod gravatar;

#[cfg(not(target_arch = "wasm32"))]
use crate::desktop_host::DesktopHostApi;
#[cfg(target_arch = "wasm32")]
use crate::web_host::PluginHostApi as DesktopHostApi;
use profile_pulse_pic_source_plugin_api::ProfilePicSourcePlugin;
use std::sync::Arc;

pub fn all_builtins(host: Arc<DesktopHostApi>) -> Vec<Box<dyn ProfilePicSourcePlugin>> {
    vec![
        Box::new(gravatar::GravatarPicSource::new(host.clone())),
        Box::new(github::GithubPicSource::new(host.clone())),
        Box::new(gitlab::GitlabPicSource::new(host)),
    ]
}

pub fn github_username_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let rest = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("github.com/"))?;
    let segment = rest.split('/').next()?.trim();
    if segment.is_empty()
        || matches!(
            segment,
            "orgs" | "settings" | "marketplace" | "features" | "login" | "signup"
        )
    {
        return None;
    }
    Some(segment.to_string())
}

/// Normalize manual GitHub input (username, `@user`, or profile URL).
pub fn normalize_github_username(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(user) = github_username_from_url(trimmed) {
        return Some(user);
    }
    let user = trimmed.trim_start_matches('@').trim_end_matches('/');
    if user.is_empty() || user.contains('/') {
        return None;
    }
    Some(user.to_string())
}

pub fn gitlab_username_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let rest = trimmed
        .strip_prefix("https://gitlab.com/")
        .or_else(|| trimmed.strip_prefix("http://gitlab.com/"))
        .or_else(|| trimmed.strip_prefix("gitlab.com/"))?;
    let segment = rest.split('/').next()?.trim();
    if segment.is_empty() || segment == "users" || segment == "explore" {
        return None;
    }

    // Skip project paths (namespace/project) — keep only single-segment user URLs.
    if rest.contains('/') {
        return None;
    }
    Some(segment.to_string())
}

/// Normalize manual GitLab input (username, `@user`, or profile URL).
pub fn normalize_gitlab_username(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(user) = gitlab_username_from_url(trimmed) {
        return Some(user);
    }
    let user = trimmed.trim_start_matches('@').trim_end_matches('/');
    if user.is_empty() || user.contains('/') {
        return None;
    }
    Some(user.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_profile_urls() {
        assert_eq!(
            github_username_from_url("https://github.com/octocat"),
            Some("octocat".into())
        );
        assert_eq!(
            github_username_from_url("https://github.com/octocat/"),
            Some("octocat".into())
        );
        assert_eq!(
            github_username_from_url("https://github.com/orgs/rust-lang"),
            None
        );
    }

    #[test]
    fn normalizes_github_username_inputs() {
        assert_eq!(
            normalize_github_username("https://github.com/octocat"),
            Some("octocat".into())
        );
        assert_eq!(
            normalize_github_username("github.com/octocat/"),
            Some("octocat".into())
        );
        assert_eq!(
            normalize_github_username("@octocat"),
            Some("octocat".into())
        );
        assert_eq!(normalize_github_username("octocat"), Some("octocat".into()));
        assert_eq!(
            normalize_github_username("https://github.com/MRDGH2821/"),
            Some("MRDGH2821".into())
        );
        assert_eq!(
            normalize_github_username("https://github.com/orgs/rust-lang"),
            None
        );
    }

    #[test]
    fn normalizes_gitlab_username_inputs() {
        assert_eq!(
            normalize_gitlab_username("https://gitlab.com/gitlab"),
            Some("gitlab".into())
        );
        assert_eq!(normalize_gitlab_username("@gitlab"), Some("gitlab".into()));
    }

    #[test]
    fn parses_gitlab_profile_urls() {
        assert_eq!(
            gitlab_username_from_url("https://gitlab.com/gitlab"),
            Some("gitlab".into())
        );
        assert_eq!(gitlab_username_from_url("https://gitlab.com/foo/bar"), None);
    }
}
