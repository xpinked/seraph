mod m20220101_000001_create_table;
mod m20250818_000002_create_code_result_table;
mod m20250823_000003_add_missing_cascade_code_results;
pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250818_000002_create_code_result_table::Migration),
            Box::new(m20250823_000003_add_missing_cascade_code_results::Migration),
        ]
    }
}
