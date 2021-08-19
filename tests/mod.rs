use sqlx::PgPool;

#[sqlx_database_tester::test(
	pool(variable = "migrated_pool", migrations = "./other_dir_migrations"),
	pool(variable = "default_migrated_pool"),
	pool(variable = "empty_pool", skip_migrations)
)]
async fn test_migrations() {
	let migrated_fox = sqlx::query("SELECT * FROM fox").fetch_all(&migrated_pool).await;
	let default_migrated_bat =
		sqlx::query("SELECT * FROM bat").fetch_all(&default_migrated_pool).await;
	let nonexistant_fox = sqlx::query("SELECT * FROM fox").fetch_all(&empty_pool).await;
	assert!(migrated_fox.is_ok());
	assert!(default_migrated_bat.is_ok());
	assert!(!nonexistant_fox.is_ok());
}

#[sqlx_database_tester::test(pool(variable = "empty_pool", skip_migrations))]
async fn test_case_returning() -> std::io::Result<()> {
	Ok(())
}

#[sqlx_database_tester::test(pool(variable = "empty_pool", skip_migrations))]
async fn test_pool_ownership_passed() {
	take_pool(empty_pool);
}

fn take_pool(_: PgPool) {}
