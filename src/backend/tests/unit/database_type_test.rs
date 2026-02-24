use net_ninja::config::DatabaseType;

#[test]
fn test_database_type_from_postgres_url() {
    let url = "postgresql://localhost/db";
    let db_type = DatabaseType::from_url(url).unwrap();
    assert_eq!(db_type, DatabaseType::PostgreSQL);

    let url2 = "postgres://localhost/db";
    let db_type2 = DatabaseType::from_url(url2).unwrap();
    assert_eq!(db_type2, DatabaseType::PostgreSQL);
}

#[test]
fn test_database_type_from_mysql_url() {
    let url = "mysql://localhost/db";
    let db_type = DatabaseType::from_url(url).unwrap();
    assert_eq!(db_type, DatabaseType::MySQL);
}

#[test]
fn test_database_type_from_sqlite_url() {
    let url = "sqlite://path/to/db.db";
    let db_type = DatabaseType::from_url(url).unwrap();
    assert_eq!(db_type, DatabaseType::SQLite);
}

#[test]
fn test_database_type_invalid_url() {
    let url = "invalid://localhost/db";
    let result = DatabaseType::from_url(url);
    assert!(result.is_err());
}

#[test]
fn test_is_server_based() {
    assert!(DatabaseType::PostgreSQL.is_server_based());
    assert!(DatabaseType::MySQL.is_server_based());
    assert!(!DatabaseType::SQLite.is_server_based());
}
