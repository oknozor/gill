## Config 

First create a `config.toml` file and edit it according to your setup.
```toml
# The public domain where gill will be served.
domain = "example.gill.org"
# If set to true gill is expected to be exposed over a non secure HTTP connection.
debug = false
# Internal port exposing the web application.
port = 3000
# Sshd exposed port, depending on your environemnt you might not want to use port 22.  
ssh_port = 2222

# Gill's database endpoint configuration
[database]
host = "gill.example.db"
port = 5432
database = "gill"
user = "postgres"
password = "password"

# Gill's openid connect provider
[oauth_provider]
client_id = "gill"
client_secret = "n5obgGTk855H1Mx3b2YG2JCO8Bc6WGq1"
provider = "https://keycloak.cloud.hoohoot.org"
user_info_url = "/auth/realms/hoohoot/protocol/openid-connect/userinfo"
auth_url = "/auth/realms/hoohoot/protocol/openid-connect/auth"
token_url = "/auth/realms/hoohoot/protocol/openid-connect/token"
```

## Creating the ssh keys

In order to run the sshd server along with gill you need to generate sshd host keys. 

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

## Docker compose

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

## Reverse proxy

Here is an example of exposing gill with nginx: 

```nginx
server {
    server_name home-raspberry.gill.pub;

    listen 443 ssl; # managed by Certbot
    ssl_certificate /etc/letsencrypt/live/home-raspberry.gill.pub/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/home-raspberry.gill.pub/privkey.pem; # managed by Certbot
    include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot

    location / {
        proxy_set_header    X-Real-IP           $remote_addr;
        proxy_set_header    X-Forwarded-For     $proxy_add_x_forwarded_for;
		proxy_set_header    Host $http_host;
		proxy_pass          http://192.168.0.17:3000/;
	}
}


server {
	if ($host = home-raspberry.gill.pub) {
		return 301 https://$host$request_uri;
	} # managed by Certbot


	listen 80;
	server_name home-raspberry.gill.pub;
	return 404; # managed by Certbot
}
```