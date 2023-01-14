#!/bin/sh

set -e

echo "$GILL_SSH_ECDSA_PUB" > /etc/ssh/ssh_host_ecdsa_key.pub
echo "$GILL_SSH_ECDSA" > /etc/ssh/ssh_host_ecdsa_key
echo "$GILL_SSH_ED25519_PUB" > /etc/ssh/ssh_host_ed25519_key.pub
echo "$GILL_SSH_ED25519" > /etc/ssh/ssh_host_ed25519_key
echo "$GILL_SSH_RSA_PUB" > /etc/ssh/ssh_host_rsa_key.pub
echo "$GILL_SSH_RSA" > /etc/ssh/ssh_host_rsa_key

chmod 600 /etc/ssh/ssh_host_rsa_key

/usr/sbin/sshd

cd /home/git
su git -c "$@"