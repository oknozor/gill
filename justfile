#!/usr/bin/env just --justfile

clean:
    docker-compose down
    cargo clean

build-setup:
    docker-compose down
    # Needs a running database to prepare the offline sqlx build
    docker-compose up postgres -d
    sleep 5 # TODO wait for psql to be up

build: build-setup
    cd crates/ruisseau-api \
    && sqlx migrate run \
    && cargo sqlx prepare \
    && CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    cross build --package ruisseau-git-server --target x86_64-unknown-linux-musl --release
    docker build --no-cache -t fserver:alpine -f Dockerfile .

run: build
    docker-compose down
    docker-compose up -d
    xdg-open http://localhost:8080

rerun:
    cd crates/ruisseau-api \
    && sqlx migrate run \
    && cargo sqlx prepare \
    && CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    cross build --package ruisseau-git-server --target x86_64-unknown-linux-musl --release
    cp target/x86_64-unknown-linux-musl/release/ruisseau-git-server docker/rbin/ruisseau-git-server
    cp target/x86_64-unknown-linux-musl/release/ruisseau-api docker/rbin/ruisseau-api
    docker-compose exec ruisseau ./entrypoint.sh

css_live:
    cd crates/ruisseau-api && tailwindcss -i assets/css/style.css -o assets/css/tailwind.css --watch
