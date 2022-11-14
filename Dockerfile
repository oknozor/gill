FROM alpine:3.4
MAINTAINER Paul Delafosse "paul.delafosse@protonmail.com"

RUN apk add --no-cache \
  openssh \
  git

RUN ssh-keygen -A

WORKDIR /git-server/

# -D flag avoids password generation
# -s flag changes user's shell
RUN mkdir /git-server/keys \
  && adduser -D -s /usr/bin/git-shell git \
  && echo git:12345 | chpasswd \
  && mkdir /home/git/.ssh

COPY git-shell-commands /home/git/git-shell-commands
COPY sshd_config /etc/ssh/sshd_config
EXPOSE 22

COPY target/x86_64-unknown-linux-musl/release/server /git-server/server
COPY start.sh start.sh

ENTRYPOINT ["sh", "start.sh"]
CMD ["/git-server/server"]
