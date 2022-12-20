FROM docker.io/alpine:3.4
MAINTAINER Paul Delafosse "paul.delafosse@protonmail.com"

RUN apk add --no-cache \
  openssh \
  bash \
  git \
  curl

# Setup sshd
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

# Prepare workdir
WORKDIR /home/git
RUN mkdir bin

COPY target/x86_64-unknown-linux-musl/release/gill-app ./bin/gill-app
COPY target/x86_64-unknown-linux-musl/release/gill-git-server ./bin/gill-git-server
COPY target/x86_64-unknown-linux-musl/release/post-receive /usr/share/git-core/templates/hooks/post-receive
COPY crates/gill-app/assets/ ./
COPY docker/home/* ./

RUN chown -R git:git ./

ENTRYPOINT ["./entrypoint.sh"]