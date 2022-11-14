#!/usr/bin/env just --justfile

build:
    cross build --package server --target x86_64-unknown-linux-musl --release
    cross build --package gitserve --target x86_64-unknown-linux-musl --release
    docker build --no-cache -t fserver:alpine -f Dockerfile .

run: build
    docker-compose down
    docker-compose up