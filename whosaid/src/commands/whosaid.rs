use serenity::all::CreateQuickModal;
use std::sync::Arc;
use std::time::Duration;

use crate::database::Database;
use crate::game::Game;
use serenity::builder::*;
use serenity::futures::stream::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use sqlx::types::chrono::Local;
use tokio::time::sleep;

pub async fn run(
    database: Arc<Database>,
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> anyhow::Result<()> {
    let guild_id = match command_interaction.guild_id {
        None => {
            return Ok(());
        }
        Some(guild_id) => guild_id,
    };

    let modal = CreateQuickModal::new("Game parameters")
        .timeout(Duration::from_secs(60))
        .field(
            CreateInputText::new(InputTextStyle::Short, "Number of questions", "n_question")
                .value("10".to_owned()),
        )
        .field(
            CreateInputText::new(
                InputTextStyle::Short,
                "Minimum quote length",
                "minimum_quote_length",
            )
            .value("10".to_owned()),
        );

    let response = command_interaction.quick_modal(ctx, modal).await?.unwrap();
    let inputs = response.inputs;
    let (n_question, minimum_quote_length) = (&inputs[0], &inputs[1]);

    let n_questions = n_question.parse::<u32>();
    let minimum_quote_length = minimum_quote_length.parse::<u32>();

    if n_questions.is_err() || minimum_quote_length.is_err() {
        let message = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content("Invalid parameters"),
        );

        response.interaction.create_response(&ctx, message).await?;
        return Ok(());
    }

    let n_questions = n_questions.unwrap();
    let minimum_quote_length = minimum_quote_length.unwrap();

    let game = Game::new(database, guild_id, n_questions, minimum_quote_length).await?;

    let message = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content(format!(
            "New game started with {} questions with a minimum quote length of {}",
            n_question, minimum_quote_length
        )),
    );

    response.interaction.create_response(&ctx, message).await?;

    sleep(Duration::from_secs(5)).await;

    let quotes = game.messages();
    let users = game.users();

    for (i, quote) in quotes.iter().enumerate() {
        let mut message = CreateInteractionResponseFollowup::new().content(format!(
            r#"
                Question {}: Who said this ?
                > {}
            "#,
            i + 1,
            quote.content
        ));

        for user in users {
            message =
                message.button(CreateButton::new(user.id.to_string()).label(user.name.to_string()));
        }

        let mut message = command_interaction.create_followup(ctx, message).await?;

        let mut interaction_stream = message
            .await_component_interaction(&ctx.shard)
            .timeout(Duration::from_secs(15))
            .stream();

        let mut responses: Vec<(_, _)> = vec![];

        while let Some(interaction) = interaction_stream.next().await {
            let dt = Local::now().signed_duration_since(*message.timestamp);

            let id = &interaction.data.custom_id;

            if quote.author_id == id.parse().ok() {
                responses.push((interaction.user.mention(), dt));
            }

            interaction
                .create_response(&ctx, CreateInteractionResponse::Acknowledge)
                .await?;
        }

        let scores_msg = match responses.len() {
            0 => "No one found".to_string(),
            _ => {
                let (username, delta) = responses.first().unwrap();
                let mut msg = format!(
                    ":confetti_ball: Fastest was {} in {}.{}s",
                    username,
                    delta.num_seconds(),
                    delta.num_milliseconds() / 100
                );

                for (username, delta) in responses.iter().skip(1).take(9) {
                    let line = format!(
                        "\n{}: {}.{}s",
                        username,
                        delta.num_seconds(),
                        delta.num_milliseconds() / 100
                    );
                    msg.push_str(&line);
                }

                msg
            }
        };

        let quote_author_representation = game
            .users()
            .iter()
            .find(|u| quote.author_id == Some(u.id))
            .map_or("???", |u| &*u.name);

        message
            .edit(
                &ctx,
                EditMessage::new()
                    .content(format!(
                        r#"
Question {}: Who said this ?
> {}
---

Answer was: {}

---

{scores_msg}"#,
                        i + 1,
                        quote.content,
                        quote_author_representation
                    ))
                    .components(vec![]),
            )
            .await?;

        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("whosaid").description("Start a whosaid game")
}
