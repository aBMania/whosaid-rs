use std::sync::Arc;
use std::time::Duration;

use serenity::all::{CreateQuickModal};
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::database::Database;

#[derive(Debug, thiserror::Error)]
pub enum EmojiError {
    #[error("You need to be in a guild to set an emoji")]
    NotInAGuild(),
    #[error("Could not find emoji")]
    UnknownEmoji(),
}

pub async fn run(database: Arc<Database>, ctx: &Context, command_interaction: &CommandInteraction) -> anyhow::Result<()> {
    let modal = CreateQuickModal::new("Your emoji")
        .timeout(Duration::from_secs(60))
        .field(
            CreateInputText::new(
                InputTextStyle::Short,
                "Emoji",
                "emoji",
            ).value(":lj:".to_owned()),
        );
    let response = command_interaction.quick_modal(ctx, modal).await?.unwrap();

    let inputs = response.inputs;

    let mut emoji_name: &str = &inputs[0];

    emoji_name = emoji_name.strip_prefix(':').unwrap_or(emoji_name);
    emoji_name = emoji_name.strip_suffix(':').unwrap_or(emoji_name);

    let guild = command_interaction.guild_id.unwrap().to_partial_guild(&ctx.http).await?;

    let emoji = guild.emojis
        .iter()
        .find(|&(_, emoji)| {
            &emoji.name == emoji_name
        });

    if emoji.is_none() {
        return Err(EmojiError::UnknownEmoji().into())
    }

    if command_interaction.guild_id.is_none() {
        return Err(EmojiError::NotInAGuild().into())
    }

    let (_, emoji) = emoji.unwrap();

    database.save_user_emoji(command_interaction.user.id, command_interaction.guild_id.unwrap(), &format!("{}", &emoji)).await?;

    let message = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
        .content(format!("Your emoji has been set to {}", emoji))
    );

    response.interaction.create_response(&ctx, message).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("emoji")
        .description("Set your emoji for whosaid")
}
