use sea_orm::QueryOrder;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    DbErr, EntityTrait, QueryFilter,
};
use sea_query::Expr;
use serenity::all::{ChannelId, GuildChannel as DiscordChannel};

use crate::database::error::DatabaseError;
use crate::database::Database;
use entity::prelude::*;
use entity::{channel, message};

impl Database {
    pub async fn _save_channel(
        &self,
        discord_channel: &DiscordChannel,
    ) -> Result<(), DatabaseError> {
        match Channel::insert(Self::map_channel_to_active_model(discord_channel))
            .on_conflict(
                sea_query::OnConflict::column(channel::Column::Id)
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
    pub async fn save_channels(
        &self,
        discord_channels: &[DiscordChannel],
    ) -> Result<(), DatabaseError> {
        let new_channels: Vec<channel::ActiveModel> = discord_channels
            .iter()
            .map(Self::map_channel_to_active_model)
            .collect();

        match Channel::insert_many(new_channels)
            .on_conflict(
                sea_query::OnConflict::column(channel::Column::Id)
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

    pub async fn get_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<channel::Model, DatabaseError> {
        Channel::find_by_id(i64::from(channel_id))
            .one(&self.db)
            .await?
            .ok_or(DatabaseError::NotFound)
    }

    pub async fn set_channel_backfilled(&self, channel_id: ChannelId) -> Result<(), DatabaseError> {
        let updated = channel::ActiveModel {
            id: Set(i64::from(channel_id)),
            backfill_done: Set(true),
            ..Default::default()
        };

        Channel::update(updated).exec(&self.db).await?;

        Ok(())
    }

    pub async fn get_channel_first_message(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<message::Model>, DatabaseError> {
        Ok(Message::find()
            .filter(Expr::col(message::Column::ChannelId).eq(i64::from(channel_id)))
            .order_by_asc(message::Column::Timestamp)
            .one(&self.db)
            .await?)
    }

    pub async fn get_channel_last_message(
        &self,
        channel_id: ChannelId,
    ) -> Result<Option<message::Model>, DatabaseError> {
        Ok(Message::find()
            .filter(Expr::col(message::Column::ChannelId).eq(i64::from(channel_id)))
            .order_by_desc(message::Column::Timestamp)
            .one(&self.db)
            .await?)
    }

    fn map_channel_to_active_model(discord_channel: &DiscordChannel) -> channel::ActiveModel {
        channel::ActiveModel {
            id: Set(discord_channel.id.into()),
            name: Set(discord_channel.name.to_owned()),
            guild_id: Set(i64::from(discord_channel.guild_id)),
            last_message_id: Set(discord_channel.last_message_id.map(i64::from)),
            backfill_done: Set(false),
        }
    }
}
