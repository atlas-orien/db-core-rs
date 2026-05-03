use crate::DbContext;

/// Base repository implementation
pub struct BaseRepository {
    ctx: DbContext,
}

impl BaseRepository {
    /// Create a new base repository
    pub fn new(ctx: DbContext) -> Self {
        Self { ctx }
    }

    /// 仅供 pg-tables 内部使用
    pub fn db(&self) -> &DbContext {
        &self.ctx
    }
}
