#!/bin/bash

set -e

/usr/sbin/sshd
cd /home/git

tail -f /dev/null