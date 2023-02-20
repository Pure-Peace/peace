use clap_3::Parser;
use dotenvy::dotenv;
use sea_orm_cli::{
    handle_error, run_generate_command, run_migrate_command, Cli as SeaCli,
    Commands as SeaCommands,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = SeaCli::parse();
    let verbose = cli.verbose;

    match cli.command {
        SeaCommands::Generate { command } => {
            run_generate_command(command, verbose)
                .await
                .unwrap_or_else(handle_error);
        },
        SeaCommands::Migrate {
            migration_dir,
            database_schema,
            database_url,
            command,
        } => run_migrate_command(
            command,
            &migration_dir,
            database_schema,
            database_url,
            verbose,
        )
        .unwrap_or_else(handle_error),
    }
}
