FROM alpine:3.4
MAINTAINER Paul Delafosse "paul.delafosse@protonmail.com"

RUN apk add --no-cache \
  openssh \
  bash \
  git

RUN ssh-keygen -A

RUN  adduser -D -s /bin/bash git \
  && echo git:12345 | chpasswd \
  && mkdir /home/git/.ssh \
  && touch /home/git/.ssh/authorized_keys \
  && chown -R git:git /home/git/.ssh \
  && chmod 700 /home/git/.ssh \
  && chmod -R 600 /home/git/.ssh/*

COPY docker/sshd_config /etc/ssh/sshd_config
EXPOSE 22

WORKDIR /home/git
RUN mkdir bin
COPY target/x86_64-unknown-linux-musl/release/api ./bin/api
COPY target/x86_64-unknown-linux-musl/release/git-server ./bin/git-server
COPY .env ./.env
COPY config.toml ./config.toml
COPY crates/api/migrations ./migrations
COPY docker/entrypoint.sh ./entrypoint.sh
COPY docker/gix ./bin/gix

RUN chown git:git ./.env
RUN chown -R git:git ./migrations
RUN chown git:git ./config.toml
RUN chown -R git:git ./bin/api

 CMD ["./entrypoint.sh"]