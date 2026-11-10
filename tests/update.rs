use super::is_newer;

#[test]
fn compares_versions_numerically() {
	assert!(is_newer("0.3.0", "0.2.0"));
	assert!(is_newer("0.10.0", "0.9.9"));
	assert!(is_newer("1.0.0", "0.99.0"));
	assert!(is_newer("0.2.1", "0.2.0"));
	assert!(!is_newer("0.2.0", "0.2.0"));
	assert!(!is_newer("0.1.9", "0.2.0"));
}

#[test]
fn ignores_suffixes_and_rejects_garbage() {
	assert!(is_newer("0.3.0-beta.1", "0.2.0"));
	assert!(!is_newer("latest", "0.2.0"));
	assert!(!is_newer("0.3", "0.2.0"));
	assert!(!is_newer("0.3.0.1", "0.2.0"));
}
