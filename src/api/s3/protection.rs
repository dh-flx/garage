//! Fail closed for protection semantics that this branch cannot enforce yet.
//!
//! This is not Object Lock implementation. In particular, accepting these
//! inputs while pruning old versions would mislead clients about durability.

pub(crate) fn unsupported_protection_name(name: &str) -> bool {
	let name = name.to_ascii_lowercase();
	name.starts_with("x-amz-object-lock-")
		|| name == "x-amz-bucket-object-lock-enabled"
		|| name == "x-amz-bypass-governance-retention"
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rejects_retention_hold_enablement_and_bypass_independently_of_value() {
		for name in [
			"x-amz-object-lock-mode",
			"x-amz-object-lock-retain-until-date",
			"x-amz-object-lock-legal-hold",
			"x-amz-object-lock-event-hold",
			"x-amz-bucket-object-lock-enabled",
			"x-amz-bypass-governance-retention",
		] {
			assert!(unsupported_protection_name(name), "{}", name);
		}
	}

	#[test]
	fn rejects_mixed_case_form_names() {
		assert!(unsupported_protection_name("X-Amz-Object-Lock-Mode"));
		assert!(unsupported_protection_name(
			"X-AMZ-BUCKET-OBJECT-LOCK-ENABLED"
		));
	}

	#[test]
	fn ordinary_metadata_and_encryption_are_not_lock_claims() {
		for name in [
			"x-amz-meta-object-lock-mode",
			"x-amz-server-side-encryption-customer-algorithm",
			"x-amz-checksum-sha256",
			"x-amz-content-sha256",
			"content-type",
			"key",
			"file",
		] {
			assert!(!unsupported_protection_name(name), "{}", name);
		}
	}
}
