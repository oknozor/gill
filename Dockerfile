FROM docker.io/alpine:3.4
MAINTAINER Paul Delafosse "paul.delafosse@protonmail.com"

RUN apk --no-cache add openssh git

# Setup sshd
COPY docker/sshd_config /etc/ssh/sshd_config

RUN adduser -D -s /bin/sh git

WORKDIR /home/git
USER git

#Prepare workdir
RUN mkdir .ssh \
  && touch .ssh/authorized_keys \
  && chmod 700 .ssh \
  && chmod -R 600 .ssh/*
RUN mkdir bin

# Install binaries
COPY target/x86_64-unknown-linux-musl/release/gill-app /usr/bin/gill-app
COPY target/x86_64-unknown-linux-musl/release/gill-git-server /usr/bin/gill-git-server
COPY target/x86_64-unknown-linux-musl/release/post-receive /usr/share/git-core/templates/hooks/post-receive

# Install assets
COPY crates/gill-db/migrations /opt/gill/migrations
COPY crates/gill-app/assets/ /opt/gill/assets

EXPOSE 22
EXPOSE 3000

USER root

RUN mkdir /root/.ssh \
    && chmod 700 /root/.ssh

COPY ./docker/entrypoint.sh /entrypoint.sh

CMD ["gill-app"]
ENTRYPOINT ["/entrypoint.sh"]
