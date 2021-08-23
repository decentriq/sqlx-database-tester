# SQLX Database tester

[![pipeline status][badge-pipeline-img]][badge-pipeline-url]
[![coverage report][badge-coverage-img]][badge-coverage-url]
[![docs main][badge-docs-main-img]][badge-docs-main-url]

[badge-pipeline-img]: https://gitlab.com/famedly/company/backend/libraries/sqlx-database-tester/badges/main/pipeline.svg
[badge-pipeline-url]: https://gitlab.com/famedly/company/backend/libraries/sqlx-database-tester/-/commits/main
[badge-coverage-img]: https://gitlab.com/famedly/company/backend/libraries/sqlx-database-tester/badges/main/coverage.svg
[badge-coverage-url]: https://gitlab.com/famedly/company/backend/libraries/sqlx-database-tester/-/commits/main
[badge-docs-main-img]: https://img.shields.io/badge/docs-main-blue
[badge-docs-main-url]: https://famedly.gitlab.io/company/backend/libraries/sqlx-database-tester/sqlx_database_tester/index.html

This library makes it possible to create rust test cases for unit / integration testing with database access to unique databases per test case.

Each database is created with the database name generated as UUID v4, to prevent any collision between test cases.

The sqlx database pool variable of requested name is exposed to the test function scope.

The macro allows for creation and exposure of multiple databases per test function if needed.

## Usage:
- For the database connection itself, set up the env variable `DATABASE_URL` with a proper postgresql connection URI.
  If `.env` exists, it will be used through the `dotenv` crate. (if it contains the database name, it will be used for a temporary database name prefix in form of `<PFX>_<UUID>`).
- Make sure that the user that connects to the database has permissions to create new databases.
- You must specify one of features for this crate, `runtime-actix` or `runtime-tokio` for use of respective runtimes

```rust
#[sqlx_database_tester::test(
    pool(variable = "default_migrated_pool"),
    pool(variable = "migrated_pool", migrations = "./test_migrations"),
    pool(variable = "empty_db_pool", 
         transaction_variable = "empty_db_transaction", 
         skip_migrations),
)]
async fn test_server_start() {
    let migrated_pool_tables = sqlx::query!("SELECT * FROM pg_catalog.pg_tables")
        .fetch_all(&migrated_pool)
        .await
        .unwrap();
    let empty_pool_tables = sqlx::query!("SELECT * FROM pg_catalog.pg_tables")
        .fetch_all(&migrated_pool)
        .await
        .unwrap();
    println!("Migrated pool tables: \n {:#?}", migrated_pool_tables);
    println!("Empty pool tables: \n {:#?}", empty_pool_tables);
}
```

## Macro attributes:

- `variable`: Variable of the PgPool to be exposed to the function scope (mandatory)
- `transaction_variable`: If present, starts a new transaction and exposes variable of this name to the function scope
- `migrations`: Path to SQLX migrations directory for the specified pool (falls back to default ./migrations directory if left out)
- `skip_migrations`: If present, doesn't run any migrations
----------------------------------------------------------------------

# Famedly

**This project is part of the source code of Famedly.**

We think that software for healthcare should be open source, so we publish most
parts of our source code at [gitlab.com/famedly](https://gitlab.com/famedly/company).

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of
conduct, and the process for submitting pull requests to us.

For licensing information of this project, have a look at the [LICENSE](LICENSE.md)
file within the repository.

If you compile the open source software that we make available to develop your
own mobile, desktop or embeddable application, and cause that application to
connect to our servers for any purposes, you have to aggree to our Terms of
Service. In short, if you choose to connect to our servers, certain restrictions
apply as follows:

- You agree not to change the way the open source software connects and
  interacts with our servers
- You agree not to weaken any of the security features of the open source software
- You agree not to use the open source software to gather data
- You agree not to use our servers to store data for purposes other than
  the intended and original functionality of the Software
- You acknowledge that you are solely responsible for any and all updates to
  your software

No license is granted to the Famedly trademark and its associated logos, all of
which will continue to be owned exclusively by Famedly GmbH. Any use of the
Famedly trademark and/or its associated logos is expressly prohibited without
the express prior written consent of Famedly GmbH.

For more
information take a look at [Famedly.com](https://famedly.com) or contact
us by [info@famedly.com](mailto:info@famedly.com?subject=[GitLab]%20More%20Information%20)
