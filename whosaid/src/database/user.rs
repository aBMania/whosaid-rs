use sea_orm::{
    ActiveValue, ColumnTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use sea_query::JoinType;
use serenity::all::{GuildId, User as DiscordUser, UserId};

use entity::prelude::*;

use crate::database::Database;
use crate::database::error::DatabaseError;

#[derive(FromQueryResult)]
pub struct UserWithEmoji {
    pub(crate) id: i64,
    pub(crate) name: String,
    // pub(crate) discriminator: Option<u32>,
    // pub(crate) global_name: Option<String>,
    // pub(crate) bot: Option<bool>,
    // pub(crate) emoji: String,
}

impl Database {
    pub async fn save_user(&self, discord_user: &DiscordUser) -> Result<(), DatabaseError> {
        match User::insert(Self::map_user_to_active_model(discord_user))
            .on_conflict(
                sea_query::OnConflict::column(entity::user::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await
        {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }
    pub async fn save_users(&self, discord_users: &Vec<&DiscordUser>) -> Result<(), DatabaseError> {
        let new_users: Vec<entity::user::ActiveModel> = discord_users
            .iter()
            .map(|discord_user: &&serenity::all::User| Self::map_user_to_active_model(discord_user))
            .collect();

        match User::insert_many(new_users)
            .on_conflict(
                sea_query::OnConflict::column(entity::user::Column::Id)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await
        {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }

    pub async fn get_most_active_users_with_emoji(
        &self,
        guild_id: GuildId,
        n_most_active_users: u32,
    ) -> Result<Vec<UserWithEmoji>, DatabaseError> {
        let select = User::find()
            .join_rev(JoinType::LeftJoin, entity::message::Relation::User.def())
            .join(JoinType::LeftJoin, entity::message::Relation::Channel.def())
            .join(JoinType::LeftJoin, entity::user::Relation::UserEmoji.def())
            .filter(entity::channel::Column::GuildId.eq(i64::from(guild_id)))
            .filter(entity::user::Column::Bot.eq(false))
            .group_by(entity::user::Column::Id)
            .order_by_desc(entity::message::Column::Id.count())
            .limit(n_most_active_users as u64)
            .into_model::<UserWithEmoji>()
            .all(&self.db)
            .await?;

        Ok(select)
    }

    pub async fn save_user_emoji(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        emoji: &str,
    ) -> Result<(), DatabaseError> {
        let user_emoji: entity::user_emoji::ActiveModel = entity::user_emoji::ActiveModel {
            user_id: ActiveValue::Set(user_id.into()),
            guild_id: ActiveValue::Set(guild_id.into()),
            emoji: ActiveValue::Set(emoji.into()),
        };

        match UserEmoji::insert(user_emoji)
            .on_conflict(
                sea_query::OnConflict::columns([
                    entity::user_emoji::Column::UserId,
                    entity::user_emoji::Column::GuildId,
                ])
                .update_column(entity::user_emoji::Column::Emoji)
                .to_owned(),
            )
            .exec(&self.db)
            .await
        {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }

    fn map_user_to_active_model(discord_user: &DiscordUser) -> entity::user::ActiveModel {
        entity::user::ActiveModel {
            id: ActiveValue::Set(discord_user.id.into()),
            name: ActiveValue::Set(discord_user.name.to_owned()),
            discriminator: ActiveValue::Set(discord_user.discriminator.map(|a| a.get() as i32)),
            global_name: ActiveValue::Set(discord_user.global_name.to_owned()),
            bot: ActiveValue::Set(Some(discord_user.bot)),
        }
    }
}
