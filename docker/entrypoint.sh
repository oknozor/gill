#!/bin/bash

set -e

/usr/sbin/sshd
cd /home/git
su git -c "DATABASE_URL=postgres://ruisseau:ruisseau@postgres/ruisseau ./bin/ruisseau-api"