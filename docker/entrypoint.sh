#!/bin/bash

set -e

/usr/sbin/sshd
cd /home/git
su git -c "DATABASE_URL=postgres://legit:legit@postgres/legit ./bin/api"