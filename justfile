#!/usr/bin/env just --justfile

clean:
    docker-compose down
    cargo clean

# Prepare build for offline sqlx build
# this is required to build inside cross container
migrate:
    docker-compose up postgres -d
    sqlx migrate run
    cargo sqlx prepare --merged

build:
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    docker build --no-cache -t fserver:alpine -f Dockerfile .
    docker-compose up -d

run: build
    docker-compose exec ruisseau ./entrypoint.sh

reload:
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    cp target/x86_64-unknown-linux-musl/release/ruisseau-git-server docker/rbin/ruisseau-git-server
    cp target/x86_64-unknown-linux-musl/release/ruisseau-api docker/rbin/post-receive-hook
    cp target/x86_64-unknown-linux-musl/release/ruisseau-api docker/rbin/ruisseau-api

css_live:
    cd crates/ruisseau-api && tailwindcss -i assets/css/style.css -o assets/css/tailwind.css --watch
