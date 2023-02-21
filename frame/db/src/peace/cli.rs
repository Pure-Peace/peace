use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use peace_dal::Database;
use peace_db::peace::Repository;
use peace_domain::peace::{CreateUser, Email, Password, Username};

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
        username_unicode: Option<String>,

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
            username_unicode,
            password,
            email,
        } => {
            let db = Database::connect(
                database_url.expect("database-url is required."),
            )
            .await
            .unwrap();

            println!("Creating user...");
            Repository::create_user(
                &db,
                CreateUser {
                    name: Username::from_str(username.as_str()).unwrap(),
                    name_unicode: username_unicode
                        .as_ref()
                        .map(|s| Username::from_str(s.as_str()).unwrap()),
                    password: Password::hash_password(password).unwrap(),
                    email: Email::from_str(email.as_str()).unwrap(),
                    country: None,
                },
            )
            .await
            .unwrap();
            println!("Success")
        },
    }
}
