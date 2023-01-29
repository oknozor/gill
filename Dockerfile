FROM --platform=$TARGETPLATFORM rust:1.67 as builder
WORKDIR /usr/src/gill
COPY . .
RUN SQLX_OFFLINE=true cargo build --release

FROM alpine
MAINTAINER Paul Delafosse "paul.delafosse@protonmail.com"
RUN apk --no-cache add openssh git

# Setup sshd
COPY docker/sshd_config /etc/ssh/sshd_config

RUN adduser -D -s /bin/sh git \
    && echo git:gill | chpasswd # This is needed so the user is not locked
WORKDIR /home/git
USER git

#Prepare workdir
RUN mkdir .ssh \
  && touch .ssh/authorized_keys \
  && chmod 700 .ssh \
  && chmod -R 600 .ssh/*

# Install binaries
COPY --from=builder /usr/src/gill/target/release/gill-app /usr/bin/gill-app
COPY --from=builder /usr/src/gill/target/release/gill-git-server /usr/bin/gill-git-server
COPY --from=builder /usr/src/gill/target/release/post-receive /usr/share/git-core/templates/hooks/post-receive

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
