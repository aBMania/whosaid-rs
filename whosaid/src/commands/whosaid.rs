use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use serenity::all::CreateQuickModal;

use serenity::builder::*;
use serenity::futures::stream::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use sqlx::types::chrono::Local;
use tokio::time::sleep;
use crate::database::Database;
use crate::game::Game;

pub async fn run(database: Arc<Database>, ctx: &Context, command_interaction: &CommandInteraction) -> anyhow::Result<()> {
    let guild_id = match command_interaction.guild_id {
        None => {
            return Ok(());
        }
        Some(guild_id) => guild_id
    };

    let modal = CreateQuickModal::new("Game parameters")
        .timeout(Duration::from_secs(60))
        .field(
            CreateInputText::new(
                InputTextStyle::Short,
                "Number of questions",
                "n_question",
            ).value("10".to_owned()),
        )
        .field(
            CreateInputText::new(
                InputTextStyle::Short,
                "Minimum quote length",
                "minimum_quote_length",
            ).value("10".to_owned()),
        );

    let response = command_interaction.quick_modal(ctx, modal).await?.unwrap();
    let inputs = response.inputs;
    let (n_question, minimum_quote_length) = (&inputs[0], &inputs[1]);

    let n_questions = n_question.parse::<u32>();
    let minimum_quote_length = minimum_quote_length.parse::<u32>();

    if n_questions.is_err() || minimum_quote_length.is_err() {
        let message = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
            .content("Invalid parameters")
        );

        response.interaction.create_response(&ctx, message).await?;
        return Ok(());
    }

    let n_questions = n_questions.unwrap();
    let minimum_quote_length = minimum_quote_length.unwrap();

    let game = Game::new(database, guild_id, n_questions, minimum_quote_length).await?;

    let message = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
        .content(format!("New game started with {} questions with a min quote length of {}", n_question, minimum_quote_length))
    );

    response.interaction.create_response(&ctx, message).await?;

    sleep(Duration::from_secs(5)).await;

    let quotes = game.messages();
    let users = game.users();

    for (i, quote) in quotes.iter().enumerate() {
        let mut message = CreateInteractionResponseFollowup::new()
            .content(format!(r#"
                Question {}: Who said this ?
                > {}
            "#, i + 1, quote.content));

        for user in users {
            message = message.button(CreateButton::new(user.id.to_string()).label(user.name.to_string()));
        }

        let mut message = command_interaction.create_followup(ctx, message).await?;


        let mut interaction_stream =
            message.await_component_interaction(&ctx.shard)
                .timeout(Duration::from_secs(15))
                .stream();

        let mut responses: Vec<(_, _)> = vec![];

        while let Some(interaction) = interaction_stream.next().await {
            let dt = Local::now().signed_duration_since(*message.timestamp);

            let id = &interaction.data.custom_id;

            if id.eq(&quote.author_id.to_string()) {
                responses.push((interaction.user.to_string(), dt));
            }

            interaction.create_response(&ctx, CreateInteractionResponse::Acknowledge).await?;
        }

        let fastest_msg = match responses.len() {
            0 => "No one found".to_string(),
            _ => {
                let (fatest, fastest_delta) = responses.get(0).unwrap();
                let mut msg = format!("Fastest was {} in {}.{}s", fatest, fastest_delta.num_seconds(), fastest_delta.num_milliseconds() / 100);
                msg.push_str("\ntest");

                msg
            }
        };

        message.edit(&ctx, EditMessage::new()
            .content(format!(r#"
Question {}: Who said this ?
> {}
---

Answer was: {}
{fastest_msg}"#
                             , i + 1, quote.content, quote.author_id))
            .components(vec![]),
        ).await?;

        sleep(Duration::from_secs(5)).await;
    }


    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("whosaid")
        .description("Start a whosaid game")
}
