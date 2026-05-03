/// Macro to implement Repository trait for a concrete entity
#[macro_export]
macro_rules! impl_repository {
    ($struct_name:ident, $entity:ty, $model:ty) => {
        pub struct $struct_name {
            base: $crate::BaseRepository,
        }

        impl $struct_name {
            pub fn new(db: $crate::DbContext) -> Self {
                Self {
                    base: $crate::BaseRepository::new(db),
                }
            }
        }

        impl $crate::Repository<$entity, $model> for $struct_name {
            fn db(&self) -> &$crate::DbContext {
                self.base.db()
            }
        }
    };
}
