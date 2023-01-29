# Note that the following build needs binaries to be precompiled for the target
# architectures. Use the `build-all` just recipies to build for all targets.
FROM alpine as arm-builder
COPY ./target/armv7-unknown-linux-musleabihf/release/gill-app /gill-app
COPY ./target/armv7-unknown-linux-musleabihf/release/gill-git-server /gill-git-server
COPY ./target/armv7-unknown-linux-musleabihf/release/post-receive /post-receive

FROM alpine as arm64-builder
COPY ./target/aarch64-unknown-linux-musl/release/gill-app /gill-app
COPY ./target/aarch64-unknown-linux-musl/release/gill-git-server /gill-git-server
COPY ./target/aarch64-unknown-linux-musl/release/post-receive /post-receive

FROM alpine as amd64-builder
COPY ./target/x86_64-unknown-linux-musl/release/gill-app /gill-app
COPY ./target/x86_64-unknown-linux-musl/release/gill-git-server /gill-git-server
COPY ./target/x86_64-unknown-linux-musl/release/post-receive /post-receive

FROM ${TARGETARCH}-builder AS builder

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
COPY --from=builder /gill-app /usr/bin/gill-app
COPY --from=builder /gill-git-server /usr/bin/gill-git-server
COPY --from=builder /post-receive /usr/share/git-core/templates/hooks/post-receive

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
