# db-core-rs

Database connection management layer built on SeaORM.

## Purpose

`db-core-rs` provides the database connection layer, including:
- Multi-database connection management
- Connection pooling configuration
- Error handling for database operations

## Usage

This crate is typically used together with `pg-tables` which provides business logic.

```rust
use db_core_rs::{DatabaseConfig, DatabaseManager};

#[tokio::main]
async fn main() -> db_core_rs::Result<()> {
    let config = DatabaseConfig::new(
        "main",
        "postgres://user:password@localhost/mydb"
    )
    .max_connections(20)
    .with_sql_logging(true);

    let manager = DatabaseManager::new(vec![config]).await?;
    let db = manager.default()?;

    // Pass db to your business logic layer

    Ok(())
}
```

## Features

- **DatabaseConfig**: Flexible database configuration with builder pattern
- **DatabaseManager**: Manages multiple named database connections
- **DbContext**: The only public database context for normal and transactional work
- **Connection Pooling**: Built-in connection pool management via SeaORM
- **Error Handling**: Comprehensive error types for database operations

## Transactions

Business code decides when a transaction is needed, but repositories only need
`DbContext`. The transaction callback receives another `DbContext`, so the same
service constructors work inside and outside transactions.

```rust
db.transaction(|tx| {
    Box::pin(async move {
        let repo = UserRepository::new(tx.clone());
        repo.create_user(...).await?;
        Ok(())
    })
})
.await?;
```

## Configuration Options

```rust
DatabaseConfig::new("name", "connection_url")
    .max_connections(20)         // Max connections in pool (default: 10)
    .min_connections(5)          // Min connections in pool (default: 1)
    .connect_timeout(60)         // Connection timeout in seconds (default: 30)
    .idle_timeout(600)           // Idle timeout in seconds (default: 600)
    .with_sql_logging(true);     // Enable SQL logging (default: false)
```
