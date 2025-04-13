use std::collections::HashSet;
use std::sync::Arc;

use serenity::all::{Context, GetMessages, GuildChannel, MessageId, User};
use tokio::sync::Semaphore;
use tracing::{error, info};

use crate::database::Database;

#[derive(Clone)]
pub struct Scrapper {
    database: Arc<Database>,
    scrap_semaphore: Arc<Semaphore>,
}

impl Scrapper {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            scrap_semaphore: Arc::from(Semaphore::const_new(1)),
        }
    }
}

impl Scrapper {
    pub async fn scrap(&self, ctx: &Context) {
        let Ok(permit) = self.scrap_semaphore.try_acquire() else {
            info!("Scrapping already running, skipping");
            return;
        };

        info!("Scrapping started");

        if let Err(e) = self._scrap(ctx).await {
            error!("Scrapping failed: {}", e);
        }

        info!("Scrapping done");

        drop(permit);
    }

    async fn _scrap(&self, ctx: &Context) -> anyhow::Result<()> {
        if let Ok(guilds) = self.database.get_guilds().await {
            info!("Guilds: {:?}", guilds);
        }

        let guilds = ctx.http.get_guilds(None, None).await?;

        for guild in guilds {
            let partial_guild = ctx.http.get_guild(guild.id).await?;
            let guild_owner = ctx.http.get_user(partial_guild.owner_id).await?;

            self.database.save_user(&guild_owner).await?;

            let guild_members = ctx.http.get_guild_members(guild.id, None, None).await?;
            let guild_users: Vec<_> = guild_members.iter().map(|member| &member.user).collect();

            self.database.save_users(&guild_users).await?;
            self.database.save_guild(&partial_guild).await?;

            let channels: Vec<GuildChannel> = ctx
                .http
                .get_channels(guild.id)
                .await?
                .into_iter()
                .filter(|channel| channel.is_text_based())
                .collect();
            self.database.save_channels(&channels).await?;

            let guild_users: HashSet<_> = guild_users.into_iter().collect();

            for channel in channels {
                match self.scrap_channel(ctx, &channel, &guild_users).await {
                    Ok(_) => {
                        info!(
                            "Scrapped channel {} from guild {}",
                            channel.name, guild.name
                        );
                    }
                    Err(e) => {
                        error!(
                            "Srapping of channel {} from guild {} failed: {}",
                            channel.name, guild.name, e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn scrap_channel(
        &self,
        ctx: &Context,
        channel: &GuildChannel,
        guild_users: &HashSet<&User>,
    ) -> anyhow::Result<()> {
        let db_channel = self.database.get_channel(channel.id).await?;

        let mut backfill_done = db_channel.backfill_done;

        while !backfill_done {
            let db_channel_first_message =
                self.database.get_channel_first_message(channel.id).await?;

            let builder = match db_channel_first_message {
                None => GetMessages::new(),
                Some(db_channel_first_message) => {
                    GetMessages::new().before(MessageId::new(db_channel_first_message.id as u64))
                }
            };

            let messages = channel.messages(&ctx, builder).await?;

            if messages.is_empty() {
                backfill_done = true;
                self.database.set_channel_backfilled(channel.id).await?;
            } else {
                self.database.save_messages(&messages, guild_users).await?;
            }
        }

        loop {
            let db_channel_last_message =
                self.database.get_channel_last_message(channel.id).await?;

            let builder = match db_channel_last_message {
                None => GetMessages::new().limit(u8::MAX),
                Some(db_channel_last_message) => GetMessages::new()
                    .limit(u8::MAX)
                    .after(MessageId::new(db_channel_last_message.id as u64)),
            };

            let messages = channel.messages(&ctx, builder).await?;

            if messages.is_empty() {
                break;
            } else {
                self.database.save_messages(&messages, guild_users).await?;
            }
        }

        Ok(())
    }
}
