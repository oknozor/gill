# Ruisseau 

## Dev

### Setup local db

```shell
cargo install sqlx-cli
cd crates/ruisseau-api
sqlx migrate run
```

```sql
CREATE role ruisseau WITH PASSWORD 'ruisseau';
ALTER ROLE ruisseau WITH LOGIN;
CREATE DATABASE ruisseau;
GRANT ALL on database ruisseau to ruisseau;
```

### Create keycloak admin 

```shell
docker exec local_keycloak \
    /opt/jboss/keycloak/bin/add-user-keycloak.sh \
    -u admin \
    -p admin \
&& docker restart local_keycloak
```
## Todo MVP

### Rest API

- [ ] oauth (see: https://github.com/tokio-rs/axum/blob/93251fa20321b3e93dd6c8c3a229ad005928bdab/examples/oauth/src/main.rs)
- [x] oauth resources (REST api using Bearer)
- [ ] database & route
  - [ ] user
    - [x] Create
    - [ ] Read
    - [ ] Update
    - [ ] Delete 
    - [ ] list
      - [ ] ssh keys
        - [ ] Create 
        - [ ] Delete 
  - [ ] repository
    - [x] Create
    - [ ] Read
      - [ ] All
      - [x] All on instance 
      - [ ] One 
    - [ ] Update
    - [ ] Delete

### Git server 

- [x] Basic operation (clone, push, fetch, etc)
- [ ] Use git oxide instead of git
- [ ] use AuthorizedKeysCommand instead of `command=` to retrieve user ssh key from database
- [ ] set up server side hooks
  - [ ] `pre-receive` branch protection, user permission etc.
  - [ ] `post-receive` notification. 

### Front end

- [ ] Browse repositories by user
- [ ] Browse repositories by org
- [ ] Search repository

### Activity pub