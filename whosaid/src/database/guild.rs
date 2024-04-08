use sea_orm::{ActiveValue, DbErr, EntityTrait};
use serenity::all::PartialGuild as DiscordGuild;
use serenity::futures::TryFutureExt;

use entity::prelude::*;

use crate::database::Database;
use crate::database::error::DatabaseError;

impl Database {
    pub async fn get_guilds(&self) -> Result<Vec<entity::guild::Model>, DatabaseError> {
        Guild::find()
            .all(&self.db)
            .map_err(DatabaseError::DbError)
            .await
    }

    pub async fn save_guild(&self, discord_guild: &DiscordGuild) -> Result<(), DatabaseError> {
        let new_guild = entity::guild::ActiveModel {
            id: ActiveValue::Set(discord_guild.id.into()),
            name: ActiveValue::Set(discord_guild.name.to_owned()),
            owner_id: ActiveValue::Set(i64::from(discord_guild.owner_id)),
        };

        match Guild::insert(new_guild)
            .on_conflict(
                sea_query::OnConflict::column(entity::guild::Column::Id)
                    .do_nothing()
                    .to_owned()
            )
            .exec(&self.db).await {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }
}