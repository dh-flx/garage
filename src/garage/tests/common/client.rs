use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{Client, Config};

use super::garage::Key;

pub fn build_client(key: &Key, endpoint: http::Uri) -> Client {
	let credentials = Credentials::new(&key.id, &key.secret, None, None, "garage-integ-test");

	let config = Config::builder()
		.endpoint_url(endpoint.to_string())
		.region(super::REGION)
		.credentials_provider(credentials)
		.behavior_version(BehaviorVersion::latest())
		.build();

	Client::from_conf(config)
}
