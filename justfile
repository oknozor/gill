#!/usr/bin/env just --justfile

build:
    # Needs a running database to prepare the offline sqlx build
    docker-compose up postgres -d
    sleep 3 # TODO wait for psql to be up

    cd crates/api \
    && sqlx migrate run \
    && cargo sqlx prepare \
    && CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release

    cross build --package git-server --target x86_64-unknown-linux-musl --release

    docker build -t fserver:alpine -f Dockerfile .

run: build
    docker-compose down
    docker-compose up