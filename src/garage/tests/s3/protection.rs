//! Unsupported protection must never be acknowledged as an ordinary mutation.
use aws_sdk_s3::primitives::ByteStream;
use hyper::{Method, StatusCode};

use crate::common;

#[tokio::test]
async fn test_protection_put_and_delete_leave_existing_object_unchanged() {
	let ctx = common::context();
	let bucket = ctx.create_bucket("protection-put");
	ctx.client
		.put_object()
		.bucket(&bucket)
		.key("existing")
		.body(ByteStream::from_static(b"original"))
		.send()
		.await
		.unwrap();

	for name in [
		"x-amz-object-lock-mode",
		"x-amz-object-lock-retain-until-date",
		"x-amz-object-lock-legal-hold",
		"x-amz-object-lock-event-hold",
	] {
		let response = ctx
			.custom_request
			.builder(bucket.clone())
			.method(Method::PUT)
			.path("existing")
			.signed_header(name, "unsupported")
			.body(b"replacement".to_vec())
			.send()
			.await
			.unwrap();
		assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
	}
	let response = ctx
		.custom_request
		.builder(bucket.clone())
		.method(Method::DELETE)
		.path("existing")
		.signed_header("x-amz-bypass-governance-retention", "true")
		.send()
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
	let stored = ctx
		.client
		.get_object()
		.bucket(&bucket)
		.key("existing")
		.send()
		.await
		.unwrap();
	assert_bytes_eq!(stored.body, b"original");
}

#[tokio::test]
async fn test_protection_copy_and_multipart_do_not_create_destination() {
	let ctx = common::context();
	let bucket = ctx.create_bucket("protection-copy");
	ctx.client
		.put_object()
		.bucket(&bucket)
		.key("source")
		.body(ByteStream::from_static(b"source"))
		.send()
		.await
		.unwrap();
	let response = ctx
		.custom_request
		.builder(bucket.clone())
		.method(Method::PUT)
		.path("destination")
		.signed_header("x-amz-copy-source", format!("/{}/source", bucket))
		.signed_header("x-amz-object-lock-mode", "COMPLIANCE")
		.send()
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
	let response = ctx
		.custom_request
		.builder(bucket.clone())
		.method(Method::POST)
		.path("destination")
		.query_param("uploads", None::<String>)
		.signed_header("x-amz-object-lock-legal-hold", "ON")
		.send()
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
	let objects = ctx
		.client
		.list_objects_v2()
		.bucket(&bucket)
		.send()
		.await
		.unwrap();
	assert_eq!(objects.contents().len(), 1);
	assert_eq!(objects.contents()[0].key(), Some("source"));
	let uploads = ctx
		.client
		.list_multipart_uploads()
		.bucket(&bucket)
		.send()
		.await
		.unwrap();
	assert!(uploads.uploads().is_empty());
}

#[tokio::test]
async fn test_protection_bucket_enablement_does_not_create_bucket() {
	let ctx = common::context();
	let bucket = format!("protection-enable-{:016x}", rand::random::<u64>());
	let response = ctx
		.custom_request
		.builder(bucket.clone())
		.method(Method::PUT)
		.signed_header("x-amz-bucket-object-lock-enabled", "true")
		.send()
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
	let buckets = ctx.client.list_buckets().send().await.unwrap();
	assert!(!buckets
		.buckets()
		.iter()
		.any(|entry| entry.name() == Some(bucket.as_str())));
}
