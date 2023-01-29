#!/usr/bin/env just --justfile

export DATABASE_URL := "postgres://postgres:postgres@localhost/gill"

## Dev commands
clean:
    docker compose -f docker-compose.dev.yml down
    cargo clean

reset-db:
    docker-compose -f docker-compose.dev.yml exec gill "pkill" "gill-app" || true
    docker-compose -f docker-compose.dev.yml exec gill-2 "pkill" "gill-app" || true
    docker-compose -f docker-compose.dev.yml up postgres -d
    yes | sqlx database drop
    sqlx database create
    sqlx migrate run --source crates/gill-db/migrations
    yes | sqlx database drop --database-url "postgres://postgres:postgres@localhost/gill_2"
    sqlx database create --database-url "postgres://postgres:postgres@localhost/gill_2"
    sqlx migrate run --source crates/gill-db/migrations  --database-url "postgres://postgres:postgres@localhost/gill_2"
    cargo sqlx prepare --merged

build: reset-db
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    docker build -t gill-app:latest -f Dockerfile .
    docker-compose up -d

reload:
    docker-compose -f docker-compose.dev.yml exec gill "pkill" "gill-app" || true
    docker-compose -f docker-compose.dev.yml exec gill-2 "pkill" "gill-app" || true
    cargo sqlx prepare --merged
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release
    cp target/x86_64-unknown-linux-musl/release/gill-git-server docker/dev/home/bin/gill-git-server
    cp target/x86_64-unknown-linux-musl/release/gill-app docker/dev/home/bin/gill-app
    cp target/x86_64-unknown-linux-musl/release/post-receive docker/dev/home/hooks/post-receive
    cp target/x86_64-unknown-linux-musl/release/gill-git-server docker/dev/home2/bin/gill-git-server
    cp target/x86_64-unknown-linux-musl/release/post-receive docker/dev/home2/hooks/post-receive
    cp target/x86_64-unknown-linux-musl/release/gill-app docker/dev/home2/bin/gill-app
    docker-compose -f docker-compose.dev.yml exec gill ./entrypoint.sh

css-live-reload:
    cd crates/gill-app && tailwindcss -m -i assets/css/style.css -o assets/css/tailwind.min.css --watch


## Docker build
migrate-db:
    docker-compose -f docker-compose.dev.yml up postgres -d
    sqlx migrate run --source crates/gill-db/migrations
    cargo sqlx prepare --merged

build-x86: migrate-db
    CROSS_CONFIG=Cross.toml cross build --target x86_64-unknown-linux-musl --release

build-arm-v7: migrate-db
    CROSS_CONFIG=Cross.toml cross build --target armv7-unknown-linux-musleabihf --release

build-arm-64: migrate-db
    CROSS_CONFIG=Cross.toml cross build --target aarch64-unknown-linux-musl --release

build-docker-image: build-x86
    docker compose build --no-cache

# Helpers
generate-ssh-env:
    mkdir -p /tmp/etc/ssh
    ssh-keygen -A -f /tmp
    echo "GILL_SSH_ECDSA_PUB: '`cat /tmp/etc/ssh/ssh_host_ecdsa_key.pub`'" >> docker/sshd.env
    echo "GILL_SSH_ECDSA: '`cat /tmp/etc/ssh/ssh_host_ecdsa_key`'" >> docker/sshd.env
    echo "GILL_SSH_ED25519_PUB: '`cat /tmp/etc/ssh/ssh_host_ed25519_key.pub`'" >> docker/sshd.env
    echo "GILL_SSH_ED25519: '`cat /tmp/etc/ssh/ssh_host_ed25519_key`'" >> docker/sshd.env
    echo "GILL_SSH_RSA_PUB: '`cat /tmp/etc/ssh/ssh_host_rsa_key.pub`'" >> docker/sshd.env
    echo "GILL_SSH_RSA: '`cat /tmp/etc/ssh/ssh_host_rsa_key`'" >> docker/sshd.env

docker-build: build-arm-64 build-x86 build-arm-v7
    docker buildx build --push --platform linux/amd64 --no-cache --build-arg arch=amd64 . -t gillpub/gill:latest-amd64
    docker buildx build --push --platform linux/arm/v7 --no-cache --build-arg arch=arm32v7 . -t gillpub/gill:latest-arm32v7
    docker buildx build --push --platform linux/arm64/v8 --no-cache --build-arg arch=arm64v8 . -t gillpub/gill:latest-arm64v8

    docker manifest create gillpub/gill:latest \
     --amend gillpub/gill:latest-amd64 \
     --amend gillpub/gill:latest-arm32v7 \
     --amend gillpub/gill:latest-arm64v8

    docker manifest push gillpub/gill:latest

