use std::sync::Arc;

use serenity::all::GuildId;

use entity::message;

use crate::database::Database;
use crate::database::error::DatabaseError;
use crate::database::user::UserWithEmoji;

pub struct Game {
    quotes: Vec<message::Model>,
    users: Vec<UserWithEmoji>,
}

impl Game {
    pub async fn new(
        database: Arc<Database>,
        guild_id: GuildId,
        n_questions: u32,
        minimum_quote_length: u32,
        n_most_active_users: u32,
    ) -> Result<Self, DatabaseError> {
        let users = database
            .get_most_active_users_with_emoji(guild_id, n_most_active_users)
            .await?;

        let quotes = database
            .get_random_messages(
                guild_id,
                n_questions,
                minimum_quote_length,
                users.iter().map(|u| u.id).collect(),
            )
            .await?;

        Ok(Self { quotes, users })
    }

    pub fn messages(&self) -> &Vec<message::Model> {
        &self.quotes
    }

    pub fn users(&self) -> &Vec<UserWithEmoji> {
        &self.users
    }
}
