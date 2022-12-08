#!/usr/bin/env just --justfile

export DATABASE_URL := "postgres://postgres:postgres@localhost/gill"

clean:
    docker-compose down
    cargo clean

compile-arm:
    cargo sqlx prepare --merged
    CROSS_CONFIG=Cross.toml cross build --target armv7-unknown-linux-musleabihf --release

scp-arm: compile-arm
 scp target/armv7-unknown-linux-musleabihf/release/gill-api git@192.168.0.17:bin/
 scp target/armv7-unknown-linux-musleabihf/release/gill-apub git@192.168.0.17:bin/
 scp target/armv7-unknown-linux-musleabihf/release/gill-git-server git@192.168.0.17:bin/
 scp target/armv7-unknown-linux-musleabihf/release/post-receive git@192.168.0.17:hooks/
 scp -r crates/gill-api/assets/ git@192.168.0.17:
 scp -r crates/gill-db/migrations/ git@192.168.0.17:

# Prepare build for offline sqlx build
# this is required to build inside cross container
migrate:
    docker-compose up postgres -d
    sqlx migrate run --source crates/gill-db/migrations
    cargo sqlx prepare --merged

build: migrate
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    docker build -t gill-api:latest -f Dockerfile .
    docker build -t gill-apub:latest -f Dockerfile.apub .
    docker-compose up -d

run: build
    docker-compose exec -d gill ./entrypoint.sh

reload:
    docker-compose exec gill "pkill" "gill-api" || true
    docker-compose exec gill "rm" "/tmp/gill-socket" || true
    cargo sqlx prepare --merged
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    cp target/x86_64-unknown-linux-musl/release/gill-git-server docker/rbin/gill-git-server
    cp target/x86_64-unknown-linux-musl/release/post-receive docker/githooks/post-receive
    cp target/x86_64-unknown-linux-musl/release/gill-api docker/rbin/gill-api
    docker-compose exec gill ./entrypoint.sh


css_live:
    cd crates/gill-api && tailwindcss -i assets/css/style.css -o assets/css/tailwind.css --watch
