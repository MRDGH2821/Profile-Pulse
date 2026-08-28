#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    use super::slugify;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn slugify_normalizes_names() {
        assert_eq!(slugify("Personal Contacts"), "personal-contacts");
    }
}
