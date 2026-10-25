# Whosaid rs

A discord bot for playing "Who said ... ?"
It works by scrapping all accessible guild messages

## Installation


## Play the game

First, all players need to setup their own emoji

## Development

### Migrations and database management

Migrations are handled by [Sea ORM](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/):

Apply migrations:

```shell
(cd migration && cargo run)
```

Check migration status (`DATABASE_URL` points to the database):

```shell
(cd migration && cargo run -- status)
```

Generate entities:
```shell
sea-orm-cli generate entity -o entity/src
```

### Tests

The database tests run every query against a real Postgres and drop its tables first, so use a scratch database:

```shell
docker run --rm -d --name whosaid-test-pg -p 25432:5432 -e POSTGRES_PASSWORD=test postgres:16
WHOSAID_TEST_DATABASE_URL=postgres://postgres:test@localhost:25432/postgres cargo test
```

Without `WHOSAID_TEST_DATABASE_URL` they are skipped. CI runs them against a Postgres service.
