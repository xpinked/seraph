use sea_orm_migration::prelude::*;
use seraph_core::config::Config;

#[async_std::main]
async fn main() {
    let config = Config::from_env();
    unsafe {
        std::env::set_var("DATABASE_URL", &config.db_url);
    }

    cli::run_cli(migration::Migrator).await;
}
