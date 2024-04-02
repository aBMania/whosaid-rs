use sea_orm::{ActiveValue, ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter, QuerySelect, QueryTrait, RelationTrait};
use sea_query::{Expr, Func, JoinType, Order, SimpleExpr};
use serenity::all::{GuildId, Message as DiscordMessage, MessageId};

use entity::prelude::*;

use crate::database::{Database, DatabaseError};

impl Database {
    pub async fn save_messages(&self, discord_messages: &Vec<DiscordMessage>) -> Result<(), DatabaseError> {
        let new_messages: Vec<entity::message::ActiveModel> = discord_messages
            .iter()
            .map(Self::map_message_to_active_model)
            .collect();

        match Message::insert_many(new_messages)
            .on_conflict(
                sea_query::OnConflict::column(entity::message::Column::Id)
                    .do_nothing()
                    .to_owned()
            )
            .exec(&self.db).await {
            Ok(_) => Ok(()),
            Err(DbErr::RecordNotInserted) => Ok(()),
            Err(err) => Err(DatabaseError::from(err)),
        }
    }

    pub async fn get_random_messages(&self, guild_id: GuildId, n_messages: u32, minimum_length: u32, users: &Vec<entity::user::Model>) -> Result<Vec<entity::message::Model>, DatabaseError> {
        let select = Message::find()
            .join(JoinType::LeftJoin, entity::message::Relation::Channel.def())
            .filter(entity::channel::Column::GuildId.eq(i64::from(guild_id))).to_owned()
            .filter(Expr::expr(Func::char_length(Expr::col(entity::message::Column::Content))).gte(minimum_length)).to_owned()
            .filter(entity::message::Column::AuthorId.is_in(users.iter().map(|u| u.id)))
            .as_query().to_owned()
            .order_by_expr(SimpleExpr::FunctionCall(Func::random()), Order::Asc).to_owned()
            .limit(n_messages as u64).to_owned();

        let statement = self.db.get_database_backend().build(&select);

        Ok(entity::message::Model::find_by_statement(statement).all(&self.db).await?)
    }

    pub async fn get_message(&self, message_id: MessageId) -> Result<entity::message::Model, DatabaseError> {
        Message::find_by_id(i64::from(message_id)).one(&self.db).await?.ok_or(DatabaseError::NotFound)
    }

    fn map_message_to_active_model(discord_message: &DiscordMessage) -> entity::message::ActiveModel {
        entity::message::ActiveModel {
            id: ActiveValue::Set(discord_message.id.into()),
            channel_id: ActiveValue::Set(i64::from(discord_message.channel_id)),
            timestamp: ActiveValue::Set(*discord_message.timestamp),
            author_id: ActiveValue::Set(i64::from(discord_message.author.id)),
            content: ActiveValue::Set(discord_message.content.to_owned()),
        }
    }
}
