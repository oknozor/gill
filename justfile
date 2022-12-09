#!/usr/bin/env just --justfile

export DATABASE_URL := "postgres://postgres:postgres@localhost/gill"

clean:
    docker-compose down
    cargo clean

compile-arm:
    cargo sqlx prepare --merged
    CROSS_CONFIG=Cross.toml cross build --target armv7-unknown-linux-musleabihf --release

reset-db:
    docker-compose exec gill "pkill" "gill-api" || true
    docker-compose exec gill-2 "pkill" "gill-api" || true
    docker-compose up postgres -d
    yes | sqlx database drop
    sqlx database create
    sqlx migrate run --source crates/gill-db/migrations
    yes | sqlx database drop --database-url "postgres://postgres:postgres@localhost/gill_2"
    sqlx database create --database-url "postgres://postgres:postgres@localhost/gill_2"
    sqlx migrate run --source crates/gill-db/migrations  --database-url "postgres://postgres:postgres@localhost/gill_2"
    cargo sqlx prepare --merged

scp-arm: compile-arm
 scp target/armv7-unknown-linux-musleabihf/release/gill-api git@192.168.0.17:bin/
 scp target/armv7-unknown-linux-musleabihf/release/gill-git-server git@192.168.0.17:bin/
 scp target/armv7-unknown-linux-musleabihf/release/post-receive git@192.168.0.17:hooks/
 scp -r crates/gill-api/assets/ git@192.168.0.17:
 scp -r crates/gill-db/migrations/ git@192.168.0.17:

build: reset-db
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    docker build -t gill-api:latest -f Dockerfile .
    docker-compose up -d

run: build
    docker-compose exec -d gill ./entrypoint.sh

reload:
    docker-compose exec gill "pkill" "gill-api" || true
    docker-compose exec gill-2 "pkill" "gill-api" || true
    docker-compose exec gill "rm" "/tmp/gill-socket" || true
    docker-compose exec gill-2 "rm" "/tmp/gill-socket" || true
    cargo sqlx prepare --merged
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    cp target/x86_64-unknown-linux-musl/release/gill-git-server docker/rbin/gill-git-server
    cp target/x86_64-unknown-linux-musl/release/post-receive docker/githooks/post-receive
    cp target/x86_64-unknown-linux-musl/release/gill-api docker/rbin/gill-api
    docker-compose exec gill ./entrypoint.sh

css_live:
    cd crates/gill-api && tailwindcss -i assets/css/style.css -o assets/css/tailwind.css --watch
