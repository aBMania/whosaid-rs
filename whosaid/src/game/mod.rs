use std::sync::Arc;
use serenity::all::GuildId;
use entity::{message, user};
use crate::database::{Database, DatabaseError};

pub struct Game {
    quotes: Vec<message::Model>,
    users: Vec<user::Model>
}

impl Game {
    pub async fn new(
        database: Arc<Database>,
        guild_id: GuildId,
        n_questions: u32,
        minimum_quote_length: u32
    ) -> Result<Self, DatabaseError> {
        let users= database.get_most_active_users(
            guild_id,
            1,
        ).await?;

        let quotes= database.get_random_messages(
            guild_id,
            n_questions,
            minimum_quote_length,
            &users,
        ).await?;



        Ok(Self {
            quotes,
            users
        })

    }

    pub fn messages(&self) -> &Vec<message::Model> {
        &self.quotes
    }

    pub fn users(&self) -> &Vec<user::Model> {
        &self.users
    }
}