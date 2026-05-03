mod config;
mod context;
pub mod error;
mod manager;

pub mod query;
pub mod repository;

pub use config::DatabaseConfig;
pub use context::DbContext;
pub use manager::DatabaseManager;
pub use query::{OrderBy, PaginatedResponse, PaginationParams};
pub use repository::{Repository, base::BaseRepository};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = DatabaseConfig::new("test", "postgres://localhost/test")
            .max_connections(20)
            .min_connections(5)
            .connect_timeout(60)
            .with_sql_logging(true);

        assert_eq!(config.name, "test");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.connect_timeout, 60);
        assert!(config.sql_logging);
    }

    #[test]
    fn test_config_defaults() {
        let config = DatabaseConfig::new("test", "postgres://localhost/test");

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.connect_timeout, 30);
        assert_eq!(config.idle_timeout, 600);
        assert!(!config.sql_logging);
    }
}
