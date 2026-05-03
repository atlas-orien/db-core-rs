use std::{future::Future, pin::Pin, sync::Arc};

use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, ExecResult, QueryResult, Statement,
};

use crate::{context::db_transaction::DbContextInner, error};

/// tables 对外暴露的数据库上下文
///
/// 约定：
/// - 外部 crate 只能“拿着它用”
/// - 不能依赖 SeaORM
#[derive(Clone)]
pub struct DbContext {
    inner: Arc<DbContextInner>,
}

impl DbContext {
    pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            inner: Arc::new(DbContextInner::from_connection(db)),
        }
    }

    fn from_inner(inner: DbContextInner) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Begin a database transaction and return it as another `DbContext`.
    ///
    /// Callers can pass the returned context into the same repo/service constructors
    /// used by non-transactional code.
    pub async fn begin(&self) -> error::Result<DbContext> {
        let tx = self.inner.begin().await?;
        Ok(Self::from_inner(DbContextInner::from_transaction(tx)))
    }

    /// Commit this transaction context.
    pub async fn commit(self) -> error::Result<()> {
        self.inner.commit().await
    }

    /// Roll back this transaction context.
    pub async fn rollback(self) -> error::Result<()> {
        self.inner.rollback().await
    }

    /// Execute operations in one transaction.
    ///
    /// - callback returns `Ok(T)`: commit
    /// - callback returns `Err(Error)`: rollback
    pub async fn transaction<F, T>(&self, callback: F) -> error::Result<T>
    where
        F: for<'c> FnOnce(
                &'c DbContext,
            )
                -> Pin<Box<dyn Future<Output = error::Result<T>> + Send + 'c>>
            + Send,
        T: Send,
    {
        let tx = self.begin().await?;
        match callback(&tx).await {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(err) => {
                if let Err(rb_err) = tx.rollback().await {
                    return Err(error::Error::Transaction(format!(
                        "{err}; rollback failed: {rb_err}"
                    )));
                }
                Err(err)
            }
        }
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for DbContext {
    fn get_database_backend(&self) -> DbBackend {
        self.inner.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.inner.execute_raw(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.inner.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.inner.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.inner.query_all_raw(stmt).await
    }

    fn support_returning(&self) -> bool {
        self.inner.support_returning()
    }

    fn is_mock_connection(&self) -> bool {
        self.inner.is_mock_connection()
    }
}
