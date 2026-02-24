// Integration tests module
#[path = "integration/postgres_types_test.rs"]
mod postgres_types_test;

#[path = "integration/mysql_types_test.rs"]
mod mysql_types_test;

#[path = "integration/sqlite_types_test.rs"]
mod sqlite_types_test;

#[path = "integration/db_pool_integration_test.rs"]
mod db_pool_integration_test;

#[path = "integration/migration_runner_test.rs"]
mod migration_runner_test;

#[path = "integration/webdriver_parallel_test.rs"]
mod webdriver_parallel_test;
