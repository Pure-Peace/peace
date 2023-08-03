use clap::{Parser, Subcommand};
use clap4 as clap;
use domain_users::{
    CreateUser, Email, Password, UsernameAscii, UsernameUnicode,
};
use dotenvy::dotenv;
use peace_db::{Database, DbConnection};
use peace_repositories::users::{UsersRepository, UsersRepositoryImpl};

#[derive(Debug, Parser)]
#[clap(version, author, about = "Peace db CLI")]
pub struct PeaceDbCli {
    #[arg(value_parser, global = true, short = 'u', long)]
    pub database_url: Option<String>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum Commands {
    #[clap(about = "[peace] Create a new user in database")]
    CreateUser {
        #[arg(long)]
        username: String,

        #[arg(long)]
        username_unicode: Option<String>,

        #[arg(long)]
        raw_password: Option<String>,

        #[arg(long)]
        md5_password: Option<String>,

        #[arg(long)]
        email: String,
    },
    #[clap(about = "[peace] Change user's password")]
    ChangeUserPassword {
        #[arg(long)]
        user_id: Option<i32>,

        #[arg(long)]
        username: Option<String>,

        #[arg(long)]
        username_unicode: Option<String>,

        #[arg(long)]
        raw_password: Option<String>,

        #[arg(long)]
        md5_password: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = PeaceDbCli::parse();

    match cli.command {
        Commands::CreateUser {
            username,
            username_unicode,
            raw_password,
            md5_password,
            email,
        } => {
            if raw_password.is_none() && md5_password.is_none() {
                panic!("raw-password or md5-password is required.");
            }

            let db = DbConnection::from(
                Database::connect(
                    cli.database_url.expect("database-url is required."),
                )
                .await
                .unwrap(),
            );

            let repo = UsersRepositoryImpl::new(db);

            println!("Creating user...");
            repo.create_user(CreateUser {
                name: UsernameAscii::new(username.as_str()).unwrap(),
                name_unicode: username_unicode
                    .as_ref()
                    .map(|s| UsernameUnicode::new(s.as_str()).unwrap()),
                password: Password::hash_password(
                    md5_password
                        .or_else(|| {
                            Some(format!(
                                "{:x}",
                                md5::compute(raw_password.unwrap().as_bytes())
                            ))
                        })
                        .unwrap(),
                )
                .unwrap(),
                email: Email::new(email.as_str()).unwrap(),
                country: None,
            })
            .await
            .unwrap();
            println!("Success")
        },
        Commands::ChangeUserPassword {
            user_id,
            username,
            username_unicode,
            raw_password,
            md5_password,
        } => {
            if raw_password.is_none() && md5_password.is_none() {
                panic!("raw-password or md5-password is required.");
            }

            let db = DbConnection::from(
                Database::connect(
                    cli.database_url.expect("database-url is required."),
                )
                .await
                .unwrap(),
            );

            let repo = UsersRepositoryImpl::new(db);

            println!("Changing user's password...");
            repo.change_user_password(
                user_id,
                username.map(|s| {
                    UsernameAscii::new(s.as_str()).unwrap().safe_name()
                }),
                username_unicode.map(|s| {
                    UsernameUnicode::new(s.as_str()).unwrap().safe_name()
                }),
                Password::hash_password(
                    md5_password
                        .or_else(|| {
                            Some(format!(
                                "{:x}",
                                md5::compute(raw_password.unwrap().as_bytes())
                            ))
                        })
                        .unwrap(),
                )
                .unwrap()
                .to_string(),
            )
            .await
            .unwrap();
            println!("Success")
        },
    }
}
