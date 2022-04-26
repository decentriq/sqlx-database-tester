//! macros for sqlx-database-tester
#![deny(
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates,
	unused_import_braces,
	unused_qualifications
)]
#![warn(missing_debug_implementations, dead_code, clippy::unwrap_used, clippy::expect_used)]

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttributeArgs, Ident};
mod generators;

/// Pool configuration
#[derive(Debug, FromMeta)]
pub(crate) struct Pool {
	/// The variable with pool that will be exposed to the test function
	variable: Ident,
	/// The optional transaction variable
	#[darling(default)]
	transaction_variable: Option<Ident>,
	/// The migration directory path
	#[darling(default)]
	migrations: Option<String>,
	/// Should the migration be skipped
	#[darling(default)]
	skip_migrations: bool,
}

impl Pool {
	/// Return identifier for variable that will contain the database name
	fn database_name_var(&self) -> Ident {
		format_ident!("__{}_db_name", &self.variable)
	}
}

/// Test case configuration
#[derive(Debug, FromMeta)]
pub(crate) struct MacroArgs {
	/// Sqlx log level
	#[darling(default)]
	level: String,
	/// The variable the database pool will be exposed in
	#[darling(multiple)]
	pool: Vec<Pool>,
}

/// Marks async test function that exposes database pool to its scope
///
/// ## Macro attributes:
///
/// - `variable`: Variable of the PgPool to be exposed to the function scope
///   (mandatory)
/// - `other_dir_migrations`: Path to SQLX other_dir_migrations directory for
///   the specified pool (falls back to default ./migrations directory if left
///   out)
/// - `skip_migrations`: If present, doesn't run any other_dir_migrations
/// ```
/// #[sqlx_database_tester::test(
///     pool(variable = "default_migrated_pool"),
///     pool(variable = "migrated_pool", migrations = "./other_dir_migrations"),
///     pool(variable = "empty_db_pool",
///          transaction_variable = "empty_db_transaction",
///          skip_migrations),
/// )]
/// async fn test_server_sta_rt() {
///     let migrated_pool_tables = sqlx::query!("SELECT * FROM pg_catalog.pg_tables")
///         .fetch_all(&migrated_pool)
///         .await
///         .unwrap();
///     let empty_pool_tables = sqlx::query!("SELECT * FROM pg_catalog.pg_tables")
///         .fetch_all(&migrated_pool)
///         .await
///         .unwrap();
///     println!("Migrated pool tables: \n {:#?}", migrated_pool_tables);
///     println!("Empty pool tables: \n {:#?}", empty_pool_tables);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(test_attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut input = syn::parse_macro_input!(item as syn::ItemFn);
	let test_attr_args = parse_macro_input!(test_attr as AttributeArgs);
	let test_attr: MacroArgs = match MacroArgs::from_list(&test_attr_args) {
		Ok(v) => v,
		Err(e) => {
			return TokenStream::from(e.write_errors());
		}
	};

	let level = test_attr.level.as_str();
	let attrs = &input.attrs;
	let vis = &input.vis;
	let sig = &mut input.sig;
	let body = &input.block;

	let runtime = if let Some(runtime) = generators::runtime() {
		runtime
	} else {
		return syn::Error::new(
			Span::call_site(),
			"One of 'runtime-actix' and 'runtime-tokio' features needs to be enabled",
		)
		.into_compile_error()
		.into();
	};

	if sig.asyncness.is_none() {
		return syn::Error::new_spanned(
			input.sig.fn_token,
			"the async keyword is missing from the function declaration",
		)
		.to_compile_error()
		.into();
	}

	sig.asyncness = None;

	let database_name_vars = generators::database_name_vars(&test_attr);
	let database_creators = generators::database_creators(&test_attr);
	let database_migrations_exposures = generators::database_migrations_exposures(&test_attr);
	let database_closers = generators::database_closers(&test_attr);
	let database_destructors = generators::database_destructors(&test_attr);

	(quote! {
		#[::core::prelude::v1::test]
		#(#attrs)*
		#vis #sig {
			sqlx_database_tester::dotenv::dotenv().ok();
			#(#database_name_vars)*
			#runtime.block_on(async {
				#[allow(clippy::expect_used)]
				let db_pool = sqlx::PgPool::connect_with(
					sqlx_database_tester::connect_options(
						sqlx_database_tester::derive_db_prefix(
							&sqlx_database_tester::get_database_uri()
						).expect("Getting database name").as_deref().unwrap_or_default(), #level)
					).await.expect("connecting to db for creation");
				#(#database_creators)*
			});

			let result = std::panic::catch_unwind(|| {
				#runtime.block_on(async {
					#(#database_migrations_exposures)*
					let res = #body;
					#(#database_closers)*
					res
				})
			});

			#runtime.block_on(async {
				#[allow(clippy::expect_used)]
				let db_pool = sqlx::PgPool::connect_with(
					sqlx_database_tester::connect_options(
						sqlx_database_tester::derive_db_prefix(
							&sqlx_database_tester::get_database_uri()
						).expect("Getting database name").as_deref().unwrap_or_default(), #level)
					).await.expect("connecting to db for deletion");
				#(#database_destructors)*
			});

			match result {
				std::result::Result::Err(_) => std::panic!("The main test function crashed, the test database got cleaned"),
				std::result::Result::Ok(o) => o
			}
		}
	}).into()
}
