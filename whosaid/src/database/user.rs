use sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, FromQueryResult, QuerySelect, QueryTrait, RelationTrait, QueryResult, EntityName, QueryOrder};
use sea_query::{Iden, JoinType, TableRef};
use sea_query::Order::Desc;
use serenity::all::{GuildId, User as DiscordUser};

use entity::prelude::*;

use crate::database::{Database, DatabaseError};

impl Database {
    pub async fn save_user(&self, discord_user: &DiscordUser) -> Result<(), DatabaseError> {
        match User::insert(Self::map_user_to_active_model(discord_user))
            .on_conflict(
                sea_query::OnConflict::column(entity::user::Column::Id)
                    .do_nothing()
                    .to_owned()
            )
            .exec(&self.db).await {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }
    pub async fn save_users(&self, discord_users: Vec<&DiscordUser>) -> Result<(), DatabaseError> {
        let new_users: Vec<entity::user::ActiveModel> = discord_users
            .into_iter()
            .map(Self::map_user_to_active_model)
            .collect();

        match User::insert_many(new_users)
            .on_conflict(
                sea_query::OnConflict::column(entity::user::Column::Id)
                    .do_nothing()
                    .to_owned()
            )
            .exec(&self.db).await {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }

    pub async fn get_most_active_users(&self, guild_id: GuildId, n_most_active_users: u32) -> Result<Vec<entity::user::Model>, DatabaseError> {
        let select = User::find()
            .join_rev(JoinType::LeftJoin, entity::message::Relation::Author.def())
            .join(JoinType::LeftJoin, entity::message::Relation::Channel.def())
            .filter(entity::channel::Column::GuildId.eq(i64::from(guild_id)))
            .filter(entity::user::Column::Bot.eq(false))
            .group_by(entity::user::Column::Id)
            .order_by_desc(entity::message::Column::Id.count())
            .limit(n_most_active_users as u64)
            .all(&self.db)
            .await?
            ;

        Ok(select)
    }

    fn map_user_to_active_model(discord_user: &DiscordUser) -> entity::user::ActiveModel {
        entity::user::ActiveModel {
            id: ActiveValue::Set(discord_user.id.into()),
            name: ActiveValue::Set(discord_user.name.to_owned()),
            discriminator: ActiveValue::Set(discord_user.discriminator.map(|a| a.get() as u32)),
            global_name: ActiveValue::Set(discord_user.global_name.to_owned()),
            bot: ActiveValue::Set(Some(discord_user.bot)),
        }
    }
}
