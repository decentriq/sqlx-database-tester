#![doc = include_str!("../README.md")]
use std::{env, str::FromStr};

pub use dotenv;
pub use sqlx_database_tester_macros::test;

/// Environmental variable containing database URL
const DATABASE_ENV_VAR: &str = "DATABASE_URL";
#[doc(hidden)]
/// Extract optional prefix from the database specified in the connection string
pub fn derive_db_prefix(uri: &str) -> Result<Option<String>, sqlx::Error> {
	Ok(sqlx::postgres::PgConnectOptions::from_str(uri)?.get_database().map(str::to_owned))
}

#[doc(hidden)]
/// Create a UUID based database name with optional prefix from the database
/// specified in the connection string
pub fn derive_db_name(uri: &str) -> Result<String, sqlx::Error> {
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
	#[allow(clippy::expect_used)]
	let mut options = sqlx::postgres::PgConnectOptions::from_str(&get_database_uri())
		.expect("Failed to parse database URI");
	options = options.database(database_name);
	#[cfg(feature = "sqlx-log")]
	if let Ok(filter) = log::LevelFilter::from_str(level) {
		use sqlx::ConnectOptions;
		options.log_statements(filter);
	}
	options
}

#[doc(hidden)]
#[must_use]
/// Retrieve database_uri from the env variable, panics if it's not available
pub fn get_database_uri() -> String {
	env::var(DATABASE_ENV_VAR)
		.unwrap_or_else(|_| panic!("The env variable {} needs to be set", DATABASE_ENV_VAR))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
	use std::env;

	use crate::{connect_options, derive_db_name, derive_db_prefix, get_database_uri};

	#[test]
	fn test_db_prefix() {
		assert_eq!(derive_db_prefix("postgresql:///").unwrap(), None);
		assert_eq!(derive_db_prefix("postgres://").unwrap(), None);
		assert_eq!(derive_db_prefix("postgresql://localhost:5433").unwrap(), None);
		assert_eq!(
			derive_db_prefix("postgresql:///mydb?host=localhost&port=5433").unwrap(),
			Some("mydb".to_owned())
		);
		assert_eq!(derive_db_prefix("postgresql://workflow-engine:password@%2Fopt%2Fpostgresql%2Fsockets/workflow-engine").unwrap(), Some("workflow-engine".to_owned()));
		assert_eq!(
			derive_db_prefix(
				"postgresql://other@localhost/otherdb?connect_timeout=10&application_name=myapp"
			)
			.unwrap(),
			Some("otherdb".to_owned())
		);
	}

	#[test]
	fn test_derive_db_name() {
		assert!(derive_db_name("postgresql:///mydb?host=localhost&port=5433")
			.unwrap()
			.starts_with("mydb_"));
		assert_eq!(
			derive_db_name("postgresql:///mydb?host=localhost&port=5433").unwrap().len(),
			37
		);

		assert_eq!(derive_db_name("postgresql:///").unwrap().len(), 32);
	}

	#[test]
	fn test_connect_options() {
		env::set_var("DATABASE_URL", "postgresql:///");
		assert_eq!(
			connect_options("test_database", "info").get_database().unwrap(),
			"test_database"
		);

		env::set_var(
			"DATABASE_URL",
			"postgresql://workflow-engine:password@%2Fopt%2Fpostgresql%2Fsockets/workflow-engine",
		);
		assert_eq!(
			connect_options("test_database", "info").get_database().unwrap(),
			"test_database"
		);

		env::remove_var("DATABASE_URL");
	}

	#[test]
	fn test_get_database_uri() {
		env::set_var(
			"DATABASE_URL",
			"postgresql://workflow-engine:password@%2Fopt%2Fpostgresql%2Fsockets/workflow-engine",
		);
		assert_eq!(
			get_database_uri(),
			"postgresql://workflow-engine:password@%2Fopt%2Fpostgresql%2Fsockets/workflow-engine"
		);

		env::remove_var("DATABASE_URL");
	}

	#[test]
	#[should_panic(expected = "The env variable DATABASE_URL needs to be set")]
	fn test_get_database_uri_panic() {
		env::remove_var("DATABASE_URL");
		let _ = get_database_uri();
	}
}
