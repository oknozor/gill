FROM alpine:3.4
MAINTAINER Paul Delafosse "paul.delafosse@protonmail.com"

RUN apk add --no-cache \
  openssh \
  bash \
  git \
  curl

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
EXPOSE 3000

WORKDIR /home/git
RUN mkdir bin
COPY target/x86_64-unknown-linux-musl/release/ruisseau-api ./bin/ruisseau-api
COPY target/x86_64-unknown-linux-musl/release/ruisseau-git-server ./bin/ruisseau-git-server
RUN echo "DATABASE_URL=postgres://ruisseau:ruisseau@postgres/ruisseau" > .env
COPY config.toml ./config.toml
COPY crates/ruisseau-api/migrations ./migrations
COPY docker/entrypoint.sh ./entrypoint.sh
COPY docker/gix ./bin/gix

RUN chown git:git ./.env
RUN chown -R git:git ./migrations
RUN chown git:git ./config.toml
RUN chown -R git:git ./bin/ruisseau-api

ENTRYPOINT ["./entrypoint.sh"]