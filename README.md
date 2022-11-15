# Legit


## Dev

### Setup local db

```shell
cargo install sqlx-cli
cd crates/api
sqlx migrate run
```

```sql
CREATE role legit WITH PASSWORD legit;
ALTER ROLE legit WITH LOGIN;
CREATE DATABASE legit;
GRANT ALL on database legit to legit;
```
## Todo MVP

### Rest API

- [ ] oauth (see: https://github.com/tokio-rs/axum/blob/93251fa20321b3e93dd6c8c3a229ad005928bdab/examples/oauth/src/main.rs)
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
    - [ ] Create
    - [ ] Read
    - [ ] Update
    - [ ] Delete

### Git server 

- [x] Basic operation (clone, push, fetch, etc)
- [ ] Use git oxide instead of git
- [ ] use AuthorizedKeysCommand instead of `command=` to retrieve user ssh key from database

### Front end

- [ ] Browse repositories by user
- [ ] Browse repositories by org
- [ ] Search repository

### Activity pub