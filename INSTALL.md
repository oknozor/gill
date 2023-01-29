## Using docker

### Config 

```toml
domain = "example.gill.org"
debug = false
port = 3000
ssh_port = 2222

[database]
host = "gill.example.db"
port = 5432
database = "gill"
user = "postgres"
password = "password"

[oauth_provider]
client_id = "gill"
client_secret = "n5obgGTk855H1Mx3b2YG2JCO8Bc6WGq1"
provider = "https://keycloak.cloud.hoohoot.org"
user_info_url = "/auth/realms/hoohoot/protocol/openid-connect/userinfo"
auth_url = "/auth/realms/hoohoot/protocol/openid-connect/auth"
token_url = "/auth/realms/hoohoot/protocol/openid-connect/token"
```

### Docker setup

#### Creating the ssh keys

```shell
    mkdir -p /tmp/etc/ssh
    ssh-keygen -A -f /tmp
    echo "GILL_SSH_ECDSA_PUB: '`cat /tmp/etc/ssh/ssh_host_ecdsa_key.pub`'" >> sshd.env
    echo "GILL_SSH_ECDSA: '`cat /tmp/etc/ssh/ssh_host_ecdsa_key`'" >> sshd.env
    echo "GILL_SSH_ED25519_PUB: '`cat /tmp/etc/ssh/ssh_host_ed25519_key.pub`'" >> sshd.env
    echo "GILL_SSH_ED25519: '`cat /tmp/etc/ssh/ssh_host_ed25519_key`'" >> sshd.env
    echo "GILL_SSH_RSA_PUB: '`cat /tmp/etc/ssh/ssh_host_rsa_key.pub`'" >> sshd.env
    echo "GILL_SSH_RSA: '`cat /tmp/etc/ssh/ssh_host_rsa_key`'" >> sshd.env
```

#### Running with docker

```shell
docker run --env-file sshd.env   \ 
  --port 3000:3000 -p 2222:22    \
  --volume ./gill_data:/home/git \ 
  --volume ./config.toml:/opt/gill/config.toml
```

#### Running with docker compose

```yaml
version: '3.9'
services:
  gill:
    image: "gillpub/gill:latest"
    restart: unless-stopped
    container_name: gill
    env_file:
      - sshd.env
    ports:
      - "3000:3000"
      - "2222:22"
    volumes:
      - ./gill_data:/home/git
      - ./config.toml:/opt/gill/config.toml
    networks:
      - gill

volumes:
  gill_data:
```

**Run:**

`docker compose up -d`