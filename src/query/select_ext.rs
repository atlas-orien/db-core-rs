use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, IntoSimpleExpr, QueryFilter, QueryOrder,
    QuerySelect, Select, Value, prelude::Expr, sea_query::ExprTrait,
};

use super::PaginationParams;
use crate::query::{OrderBy, SortOrder};

#[async_trait::async_trait]
pub trait SelectExt<E>
where
    E: EntityTrait,
{
    /// Apply optional condition (recommended entry for all WHERE logic)
    fn apply_condition(self, cond: Option<Condition>) -> Self;

    /// Apply pagination (limit + offset)
    fn pagination(self, params: &PaginationParams) -> Self;

    /// Apply order by
    fn apply_order(self, order: &OrderBy<E>) -> Self;

    /// Apply group by columns
    fn apply_group_by<C>(self, columns: Vec<C>) -> Self
    where
        C: IntoSimpleExpr;

    /// Apply optional equal condition
    fn apply_optional_eq<C, V>(self, column: C, value: Option<V>) -> Self
    where
        C: ColumnTrait,
        V: Into<Value>;

    /// Apply time range condition (from / to)
    fn apply_time_range<C, T>(self, column: C, from: Option<T>, to: Option<T>) -> Self
    where
        C: ColumnTrait,
        T: Into<Value>;

    /// Get total count (for pagination)
    async fn total_count<C>(self, db: &C) -> u64
    where
        C: ConnectionTrait;
}

#[async_trait::async_trait]
impl<E> SelectExt<E> for Select<E>
where
    E: EntityTrait,
{
    fn apply_condition(self, cond: Option<Condition>) -> Self {
        match cond {
            Some(c) => self.filter(c),
            None => self,
        }
    }

    fn pagination(self, params: &PaginationParams) -> Self {
        let params = params.clone().validate();
        self.limit(params.page_size).offset(params.offset())
    }

    fn apply_order(mut self, ob: &OrderBy<E>) -> Self {
        match ob.order {
            SortOrder::Asc => self = self.order_by_asc(ob.column),
            SortOrder::Desc => self = self.order_by_desc(ob.column),
        }
        self
    }

    fn apply_group_by<C>(mut self, columns: Vec<C>) -> Self
    where
        C: IntoSimpleExpr,
    {
        for col in columns {
            self = self.group_by(col);
        }
        self
    }

    fn apply_optional_eq<C, V>(self, column: C, value: Option<V>) -> Self
    where
        C: ColumnTrait,
        V: Into<Value>,
    {
        match value {
            Some(v) => self.filter(column.eq(v.into())),
            None => self,
        }
    }

    fn apply_time_range<C, T>(self, column: C, from: Option<T>, to: Option<T>) -> Self
    where
        C: ColumnTrait,
        T: Into<Value>,
    {
        let mut q = self;

        if let Some(f) = from {
            q = q.filter(column.gte(f.into()));
        }
        if let Some(t) = to {
            q = q.filter(column.lte(t.into()));
        }

        q
    }

    async fn total_count<C>(self, db: &C) -> u64
    where
        C: ConnectionTrait,
    {
        match self
            .select_only()
            .column_as(Expr::value(1).count(), "count")
            .into_tuple::<i64>()
            .one(db)
            .await
        {
            Ok(Some(v)) => v as u64,
            _ => 0,
        }
    }
}
