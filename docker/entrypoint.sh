#!/bin/bash

set -e

/usr/sbin/sshd
cd /home/git
su git -c "./bin/ruisseau-api"