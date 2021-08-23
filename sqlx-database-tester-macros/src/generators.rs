use std::{iter::Map, slice::Iter};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::{MacroArgs, Pool};

pub(crate) fn database_destructors(
	test_attr: &MacroArgs,
) -> Map<Iter<Pool>, fn(&Pool) -> TokenStream> {
	test_attr.pool.iter().map(|pool| {
		let database_name = pool.database_name_var();
		quote! {
				#[allow(clippy::expect_used)]
				sqlx::query(&format!(r#"DROP DATABASE "{}""#, #database_name))
					.execute(&db_pool)
					.await
					.expect("Deleting the database");
		}
	})
}

pub(crate) fn runtime() -> Option<TokenStream> {
	if cfg!(feature = "runtime-tokio") {
		Some(quote! {
			#[allow(clippy::expect_used)]
			tokio::runtime::Runtime::new().expect("Starting a tokio runtime")
		})
	} else if cfg!(feature = "runtime-actix") {
		Some(quote! { actix_rt::System::new() })
	} else {
		None
	}
}

fn pool_variable_clone_ident(name: &Ident) -> Ident {
	format_ident!("__{}", name)
}
pub(crate) fn database_closers(test_attr: &MacroArgs) -> Map<Iter<Pool>, fn(&Pool) -> TokenStream> {
	test_attr.pool.iter().map(|Pool { variable, transaction_variable, .. }| {
		let pool_variable_ident = pool_variable_clone_ident(variable);
		let transaction_closer = transaction_variable.as_ref().map(|t| {
			quote! {
			#[allow(clippy::expect_used)]
			#t.commit().await.expect("Committing the transaction");}
		});
		quote! {
			#transaction_closer
			#pool_variable_ident.close().await;
		}
	})
}

pub(crate) fn database_migrations_exposures(
	test_attr: &MacroArgs,
) -> Map<Iter<Pool>, fn(&Pool) -> TokenStream> {
	test_attr.pool.iter().map(|pool| {
		let pool_variable_ident = &pool.variable;
		let migrations = &pool.migrations;
		let database_name = pool.database_name_var();
		let pool_variable_clone_id = pool_variable_clone_ident(pool_variable_ident);
		let mut result = quote! {
			#[allow(clippy::expect_used)]
			let #pool_variable_ident = sqlx::PgPool::connect(&sqlx_database_tester::get_target_database_uri(&sqlx_database_tester::get_database_uri(), &#database_name).expect("Can't construct the target database URI")).await.expect("connecting to db");
			let #pool_variable_clone_id = #pool_variable_ident.clone();
		};
		if !pool.skip_migrations {
			result.extend(quote! {
				#[allow(clippy::expect_used)]
				sqlx::migrate!(#migrations).run(&#pool_variable_ident).await.expect("Migrating");
			});
		}
		if let Some(transaction_variable) = &pool.transaction_variable {
			result.extend(quote! {
				#[allow(clippy::expect_used)]
				let mut #transaction_variable = #pool_variable_ident.begin().await.expect("Acquiring transaction");
			})
		}
		result
	})
}

pub(crate) fn database_creators(
	test_attr: &MacroArgs,
) -> Map<Iter<Pool>, fn(&Pool) -> TokenStream> {
	test_attr.pool.iter().map(|pool| {
		let database_name = pool.database_name_var();
		quote! {
			#[allow(clippy::expect_used)]
			{
				sqlx::query(&format!(r#"CREATE DATABASE "{}""#, &#database_name)).execute(&db_pool).await.expect(&format!(r#"Creating database "{}""#, &#database_name));
			}
	}
	})
}

pub(crate) fn database_name_vars(
	test_attr: &MacroArgs,
) -> Map<Iter<Pool>, fn(&Pool) -> TokenStream> {
	test_attr.pool.iter().map(|pool| {
		let database_name = pool.database_name_var();
		quote! {
			#[allow(clippy::expect_used)]
			let #database_name = sqlx_database_tester::derive_db_name(
				&sqlx_database_tester::get_database_uri()).expect("Getting database name");
		}
	})
}
