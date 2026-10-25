//! Runs every query against a real Postgres (migrations included).
//! Skipped unless WHOSAID_TEST_DATABASE_URL points to a scratch database (its tables are dropped), e.g.
//! `docker run --rm -d -p 25432:5432 -e POSTGRES_PASSWORD=test postgres:16`, then
//! `WHOSAID_TEST_DATABASE_URL=postgres://postgres:test@localhost:25432/postgres cargo test`.

use std::collections::HashSet;
use std::env;

use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, Database as SeaOrmDatabase, DbErr, EntityTrait};
use serenity::all::{
    ChannelId, GuildChannel, GuildId, Message as DiscordMessage, MessageId, Timestamp,
    User as DiscordUser, UserId,
};

use entity::prelude::*;
use migration::{Migrator, MigratorTrait};

use crate::database::Database;

const GUILD: u64 = 10;
const CHANNEL: u64 = 20;

fn user(id: u64, bot: bool) -> DiscordUser {
    let mut u = DiscordUser::default();
    u.id = UserId::new(id);
    u.name = format!("user{id}");
    u.bot = bot;
    u
}

fn message(id: u64, author: &DiscordUser, content: &str, unix: i64) -> DiscordMessage {
    let mut m = DiscordMessage::default();
    m.id = MessageId::new(id);
    m.channel_id = ChannelId::new(CHANNEL);
    m.author = author.clone();
    m.content = content.to_string();
    m.timestamp = Timestamp::from_unix_timestamp(unix).unwrap();
    m
}

async fn database() -> Option<Database> {
    let url = env::var("WHOSAID_TEST_DATABASE_URL").ok()?;
    let db = SeaOrmDatabase::connect(url).await.unwrap();
    // start from empty tables: drops everything, then runs every migration
    Migrator::fresh(&db).await.unwrap();
    // the startup path on an up-to-date database is a no-op
    migration::migrate(&db).await.unwrap();
    Some(Database { db })
}

#[tokio::test]
async fn queries_roundtrip() {
    let Some(db) = database().await else {
        eprintln!("WHOSAID_TEST_DATABASE_URL not set, skipping");
        return;
    };

    let alice = user(1, false);
    let bob = user(2, false);
    let robot = user(3, true);
    let outsider = user(4, false);

    // users: single, many, duplicates and empty lists are no-ops
    db.save_user(&alice).await.unwrap();
    db.save_user(&alice).await.unwrap();
    db.save_users(&vec![&alice, &bob, &robot]).await.unwrap();
    db.save_users(&vec![]).await.unwrap();
    // every column decodes into the entity (catches column type drift in migrations)
    let saved = User::find_by_id(1).one(&db.db).await.unwrap().unwrap();
    assert_eq!((saved.name.as_str(), saved.bot), ("user1", Some(false)));

    // PartialGuild has no Default: the guild row is inserted directly
    let guild = entity::guild::ActiveModel {
        id: Set(GUILD as i64),
        name: Set("guild".into()),
        owner_id: Set(1),
    };
    let inserted = Guild::insert(guild)
        .on_conflict(
            OnConflict::column(entity::guild::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(&db.db)
        .await;
    assert!(matches!(inserted, Ok(_) | Err(DbErr::RecordNotInserted)));
    assert_eq!(db.get_guilds().await.unwrap().len(), 1);

    let mut channel = GuildChannel::default();
    channel.id = ChannelId::new(CHANNEL);
    channel.guild_id = GuildId::new(GUILD);
    channel.name = "general".into();
    db.save_channels(&[channel.clone()]).await.unwrap();
    db.save_channels(&[channel]).await.unwrap();
    db.save_channels(&[]).await.unwrap();
    assert!(
        !db.get_channel(ChannelId::new(CHANNEL))
            .await
            .unwrap()
            .backfill_done
    );

    // messages: the outsider is not a guild member, so their message has no author
    let members: HashSet<&DiscordUser> = [&alice, &bob, &robot].into_iter().collect();
    let messages = vec![
        message(100, &alice, "first message from alice", 1_700_000_000),
        message(101, &bob, "a message from bob", 1_700_000_100),
        message(102, &alice, "short", 1_700_000_200),
        message(103, &robot, "a message from the robot", 1_700_000_300),
        message(104, &outsider, "a message from a stranger", 1_700_000_400),
    ];
    db.save_messages(&messages, &members).await.unwrap();
    db.save_messages(&messages, &members).await.unwrap();
    db.save_messages(&[], &members).await.unwrap();
    let stranger = Message::find_by_id(104).one(&db.db).await.unwrap().unwrap();
    assert_eq!(stranger.author_id, None);

    let first = db
        .get_channel_first_message(ChannelId::new(CHANNEL))
        .await
        .unwrap();
    assert_eq!(first.map(|m| m.id), Some(100));
    let last = db
        .get_channel_last_message(ChannelId::new(CHANNEL))
        .await
        .unwrap();
    assert_eq!(last.map(|m| m.id), Some(104));
    assert!(
        db.get_channel_first_message(ChannelId::new(999))
            .await
            .unwrap()
            .is_none()
    );

    db.set_channel_backfilled(ChannelId::new(CHANNEL))
        .await
        .unwrap();
    assert!(
        db.get_channel(ChannelId::new(CHANNEL))
            .await
            .unwrap()
            .backfill_done
    );

    // most active humans: alice (2 messages) before bob (1), the robot excluded
    let active = db
        .get_most_active_users_with_emoji(GuildId::new(GUILD), 10)
        .await
        .unwrap();
    assert_eq!(active.iter().map(|u| u.id).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(active[0].name, "user1");

    // random messages: filtered by guild, minimum length and author
    let random = db
        .get_random_messages(GuildId::new(GUILD), 10, 10, vec![1, 2])
        .await
        .unwrap();
    let mut ids: Vec<i64> = random.iter().map(|m| m.id).collect();
    ids.sort();
    assert_eq!(ids, vec![100, 101]);
    let limited = db
        .get_random_messages(GuildId::new(GUILD), 1, 0, vec![1, 2])
        .await
        .unwrap();
    assert_eq!(limited.len(), 1);

    // emoji: insert, then the conflict updates it
    db.save_user_emoji(UserId::new(1), GuildId::new(GUILD), "🐱")
        .await
        .unwrap();
    db.save_user_emoji(UserId::new(1), GuildId::new(GUILD), "🐶")
        .await
        .unwrap();
    let emoji = UserEmoji::find_by_id((1, GUILD as i64))
        .one(&db.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(emoji.emoji, "🐶");
}
