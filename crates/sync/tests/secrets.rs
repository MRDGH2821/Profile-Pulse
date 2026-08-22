use profile_pulse_sync::SecretStore;
use tempfile::tempdir;

#[test]
fn secret_store_round_trip() {
    let dir = tempdir().unwrap();
    let store = SecretStore::new(dir.path());
    store.put("google:test", "token").unwrap();
    assert_eq!(
        store.get("google:test").unwrap().as_deref(),
        Some("token")
    );
}
