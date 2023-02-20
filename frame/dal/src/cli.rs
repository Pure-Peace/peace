use clap::{Parser, Subcommand};
use clap_3 as clap;
use dotenvy::dotenv;
use peace_dal::{db::peace::Repository, Database};
use sea_orm_cli::{
    handle_error, run_generate_command, run_migrate_command, Cli as SeaCli,
    Commands as SeaCommands,
};

#[derive(Debug, Parser)]
#[clap(version, author, about = "Peace db CLI")]
pub struct Cli {
    #[clap(flatten)]
    sea_cli: SeaCli,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum Commands {
    #[clap(about = "[peace] Create a new peace user in database")]
    CreatePeaceUser {
        #[clap(
            value_parser,
            global = true,
            short = 'u',
            long,
            env = "DATABASE_URL",
            help = "Database URL"
        )]
        database_url: String,

        #[clap(value_parser, long)]
        username: String,

        #[clap(value_parser, long)]
        password: String,

        #[clap(value_parser, long)]
        email: String,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();
    let verbose = cli.sea_cli.verbose;

    if let Some(command) = cli.command {
        match command {
            Commands::CreatePeaceUser {
                database_url,
                username,
                password,
                email,
            } => {
                /* Repository::create_user(
                    db: &DatabaseConnection,
                    id: i32,
                    name: String,
                    name_safe: String,
                    name_unicode: Option<String>,
                    name_unicode_safe: Option<String>,
                    password: String,
                    email: String,
                    country: Option<String>,
                ), */
                let db = Database::connect(database_url).await.unwrap();
                todo!()
            },
        }
    }

    match cli.sea_cli.command {
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
