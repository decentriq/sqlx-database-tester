#![doc = include_str!("../README.md")]
#![deny(
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates,
	unused_import_braces,
	unused_qualifications
)]
#![warn(missing_debug_implementations, dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::{env, str::FromStr};

pub use dotenv;
use http::uri::Uri;
pub use sqlx_database_tester_macros::test;

/// Environmental variable containing database URL
const DATABASE_ENV_VAR: &str = "DATABASE_URL";

#[doc(hidden)]
/// Extract optional prefix from the database specified in the connection string
pub fn derive_db_prefix(uri: &str) -> http::Result<Option<String>> {
	let target_database_uri_parts = Uri::from_str(uri)?.into_parts();

	Ok(target_database_uri_parts
		.path_and_query
		.map(|paq| paq.path().replace('/', "").trim().to_owned())
		.filter(|p| !p.is_empty()))
}

#[doc(hidden)]
/// Create a UUID based database name with optional prefix from the database
/// specified in the connection string
pub fn derive_db_name(uri: &str) -> http::Result<String> {
	let random_part = uuid::Uuid::new_v4().simple().to_string();

	Ok(if let Some(prefix) = derive_db_prefix(uri)? {
		format!("{}_{}", prefix, random_part)
	} else {
		random_part
	})
}

/// Creates a `PgConnectOptions`
#[must_use]
pub fn connect_options(
	database_name: &str,
	#[allow(unused_variables)] level: &str,
) -> sqlx::postgres::PgConnectOptions {
	#[allow(unused_imports)]
	use sqlx::ConnectOptions;
	#[allow(clippy::expect_used, unused_mut)]
	let mut options = sqlx::postgres::PgConnectOptions::from_str(
		&get_target_database_uri(&get_database_uri(), database_name)
			.expect("Can't construct the target database URI"),
	)
	.expect("Failed to parse database URI");
	#[cfg(feature = "sqlx-log")]
	if let Ok(filter) = log::LevelFilter::from_str(level) {
		options.log_statements(filter);
	}
	options
}

#[doc(hidden)]
/// Retrieve the database uri for a specific database
pub fn get_target_database_uri(uri: &str, db_name: &str) -> http::Result<String> {
	let target_database_uri_parts = Uri::from_str(uri)?.into_parts();
	let builder = Uri::builder();
	let builder = if let Some(scheme) = target_database_uri_parts.scheme {
		builder.scheme(scheme)
	} else {
		builder
	};
	let builder = if let Some(authority) = target_database_uri_parts.authority {
		builder.authority(authority)
	} else {
		builder
	};
	Ok(builder.path_and_query(format!("/{}", db_name)).build()?.to_string())
}

#[doc(hidden)]
#[must_use]
/// Retrieve database_uri from the env variable, panics if it's not available
pub fn get_database_uri() -> String {
	env::var(DATABASE_ENV_VAR)
		.unwrap_or_else(|_| panic!("The env variable {} needs to be set", DATABASE_ENV_VAR))
}
