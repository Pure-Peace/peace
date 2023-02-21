use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use peace_dal::{Database, EntityTrait, Set};
use peace_db::peace::entity::{users, users::Entity as User};

#[derive(Debug, Parser)]
#[clap(version, author, about = "Peace db CLI")]
pub struct PeaceDbCli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum Commands {
    #[clap(about = "[peace] Create a new peace user in database")]
    CreatePeaceUser {
        #[arg(value_parser, global = true, short = 'u', long)]
        database_url: Option<String>,

        #[arg(long)]
        username: String,

        #[arg(long)]
        password: String,

        #[arg(long)]
        email: String,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = PeaceDbCli::parse();

    match cli.command {
        Commands::CreatePeaceUser {
            database_url,
            username,
            password,
            email,
        } => {
            let db = Database::connect(
                database_url.expect("database-url is required."),
            )
            .await
            .unwrap();

            println!("Creating user...");
            User::insert(users::ActiveModel {
                name: Set(username.trim().to_string()),
                name_safe: Set(username
                    .trim()
                    .to_ascii_lowercase()
                    .replace(' ', "_")),
                password: Set(password),
                email: Set(email),
                ..Default::default()
            })
            .exec(&db)
            .await
            .unwrap();
            println!("Success")
        },
    }
}
