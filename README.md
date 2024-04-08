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

Generate entities:
```shell
sea-orm-cli generate entity -o entity/src
```