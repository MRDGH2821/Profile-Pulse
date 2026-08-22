pub mod github;
pub mod gitlab;
pub mod gravatar;

use std::sync::Arc;

use profile_pulse_pic_source_plugin_api::ProfilePicSourcePlugin;

use crate::desktop_host::DesktopHostApi;

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
        assert_eq!(github_username_from_url("https://github.com/orgs/rust-lang"), None);
    }

    #[test]
    fn parses_gitlab_profile_urls() {
        assert_eq!(
            gitlab_username_from_url("https://gitlab.com/gitlab"),
            Some("gitlab".into())
        );
        assert_eq!(
            gitlab_username_from_url("https://gitlab.com/foo/bar"),
            None
        );
    }
}
