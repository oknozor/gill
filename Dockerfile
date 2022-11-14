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

COPY sshd_config /etc/ssh/sshd_config
EXPOSE 22

WORKDIR /home/git
COPY target/x86_64-unknown-linux-musl/release/server /usr/bin/server
COPY target/x86_64-unknown-linux-musl/release/gitserve /usr/bin/gitserve
COPY docker/entrypoint.sh entrypoint.sh
COPY docker/gix /usr/bin/gix

CMD ["./entrypoint.sh"]