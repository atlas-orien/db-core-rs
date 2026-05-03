use std::sync::Arc;

use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr, ExecResult,
    QueryResult, Statement, TransactionTrait,
};
use tokio::sync::Mutex;

use crate::error::{Error, Result};

pub(crate) enum DbContextInner {
    Connection(Arc<DatabaseConnection>),
    Transaction(Arc<TransactionState>),
}

pub(crate) struct TransactionState {
    backend: DbBackend,
    tx: Mutex<Option<DatabaseTransaction>>,
}

impl DbContextInner {
    pub(crate) fn from_connection(db: Arc<DatabaseConnection>) -> Self {
        Self::Connection(db)
    }

    pub(crate) fn from_transaction(tx: DatabaseTransaction) -> Self {
        Self::Transaction(Arc::new(TransactionState {
            backend: tx.get_database_backend(),
            tx: Mutex::new(Some(tx)),
        }))
    }

    pub(crate) async fn begin(&self) -> Result<DatabaseTransaction> {
        match self {
            Self::Connection(db) => Ok(db.begin().await?),
            Self::Transaction(state) => {
                let guard = state.tx.lock().await;
                let tx = guard.as_ref().ok_or_else(transaction_closed)?;
                Ok(tx.begin().await?)
            }
        }
    }

    pub(crate) async fn commit(&self) -> Result<()> {
        let Self::Transaction(state) = self else {
            return Err(Error::Transaction(
                "cannot commit a non-transaction context".to_owned(),
            ));
        };

        let mut guard = state.tx.lock().await;
        let tx = guard.take().ok_or_else(transaction_closed)?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn rollback(&self) -> Result<()> {
        let Self::Transaction(state) = self else {
            return Err(Error::Transaction(
                "cannot rollback a non-transaction context".to_owned(),
            ));
        };

        let mut guard = state.tx.lock().await;
        let tx = guard.take().ok_or_else(transaction_closed)?;
        tx.rollback().await?;
        Ok(())
    }

    pub(crate) fn get_database_backend(&self) -> DbBackend {
        match self {
            Self::Connection(db) => db.get_database_backend(),
            Self::Transaction(state) => state.backend,
        }
    }

    pub(crate) async fn execute_raw(
        &self,
        stmt: Statement,
    ) -> std::result::Result<ExecResult, DbErr> {
        match self {
            Self::Connection(db) => db.execute_raw(stmt).await,
            Self::Transaction(state) => {
                let guard = state.tx.lock().await;
                let tx = guard.as_ref().ok_or_else(transaction_closed_db_err)?;
                tx.execute_raw(stmt).await
            }
        }
    }

    pub(crate) async fn execute_unprepared(
        &self,
        sql: &str,
    ) -> std::result::Result<ExecResult, DbErr> {
        match self {
            Self::Connection(db) => db.execute_unprepared(sql).await,
            Self::Transaction(state) => {
                let guard = state.tx.lock().await;
                let tx = guard.as_ref().ok_or_else(transaction_closed_db_err)?;
                tx.execute_unprepared(sql).await
            }
        }
    }

    pub(crate) async fn query_one_raw(
        &self,
        stmt: Statement,
    ) -> std::result::Result<Option<QueryResult>, DbErr> {
        match self {
            Self::Connection(db) => db.query_one_raw(stmt).await,
            Self::Transaction(state) => {
                let guard = state.tx.lock().await;
                let tx = guard.as_ref().ok_or_else(transaction_closed_db_err)?;
                tx.query_one_raw(stmt).await
            }
        }
    }

    pub(crate) async fn query_all_raw(
        &self,
        stmt: Statement,
    ) -> std::result::Result<Vec<QueryResult>, DbErr> {
        match self {
            Self::Connection(db) => db.query_all_raw(stmt).await,
            Self::Transaction(state) => {
                let guard = state.tx.lock().await;
                let tx = guard.as_ref().ok_or_else(transaction_closed_db_err)?;
                tx.query_all_raw(stmt).await
            }
        }
    }

    pub(crate) fn support_returning(&self) -> bool {
        match self {
            Self::Connection(db) => db.support_returning(),
            Self::Transaction(state) => state.backend.support_returning(),
        }
    }

    pub(crate) fn is_mock_connection(&self) -> bool {
        match self {
            Self::Connection(db) => db.is_mock_connection(),
            Self::Transaction(_) => false,
        }
    }
}

fn transaction_closed() -> Error {
    Error::Transaction("transaction context is already closed".to_owned())
}

fn transaction_closed_db_err() -> DbErr {
    DbErr::Custom("transaction context is already closed".to_owned())
}
