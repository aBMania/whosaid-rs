use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::Result;
use dotenv::dotenv;
use serenity::all::{Command, Interaction};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};

use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{info, Level};

use crate::database::Database;
use crate::scrapper::Scrapper;

mod database;
mod utils;
mod scrapper;
mod commands;

mod game;

struct Bot {
    is_loop_running: AtomicBool,
    database: Arc<Database>,
    scrapper: Arc<Scrapper>,
}

impl Bot {
    pub async fn new() -> Result<Self> {
        let database = Arc::new(Database::new().await?);
        let scrapper = Arc::new(Scrapper::new(database.clone()));
        Ok(Self {
            is_loop_running: AtomicBool::new(false),
            scrapper,
            database,
        })
    }
}

#[async_trait]
impl EventHandler for Bot {
    // #[tracing::instrument(skip(self, _ctx))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let _guild_command =
            Command::set_global_commands(&ctx.http, vec![
                commands::whosaid::register()
            ])
                .await;


        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            let scrapper = self.scrapper.clone();

            tokio::spawn(async move {
                loop {
                    scrapper.scrap(&ctx1).await;
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });
        }
    }

    // Set a handler for the `message` event - so that whenever a new message is received - the
    // closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be dispatched
    // simultaneously.
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "whosaid" => {
                    commands::whosaid::run(self.database.clone(), &ctx, &command).await.unwrap();
                    None
                },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_test_writer()
        .init();

    info!("This will be logged to stdout");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let bot = Bot::new().await?;


    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.

    let mut client =
        Client::builder(&token, intents)
            .event_handler(bot).await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
