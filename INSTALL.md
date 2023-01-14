### Prerequisites 
- [rust](https://www.rust-lang.org/tools/installsql)
- [sqlx-cli](https://crates.io/crates/sqlx-cli)
- [docker](https://www.docker.com/)
- [just](https://crates.io/crates/just)
- [cross](https://crates.io/crates/cross)

**sqlx-cli:** 
```
cargo install sqlx-cli
```

**just:**
```
cargo install just
```

### Docker build

1. Generate sshd host keys
    ```shell
    just generate-ssh-env
    ```

2. Build the docker image. 
    ```sh
    just build-docker-image
    ```
3. Setup environment:

    Edit the `[docker-compose.yml](docker-compose.yml)` environment.
   
4. Set up the database

   Note that a running postgres database is needed during the build step.
   A postgres container is provided via the `[docker-compose.yml](docker-compose.yml)`
   but you might want to use another postgres instance for production.
   If so you will need to create your database and run the sqlx migrations.
   
   **Example:**
   
   ```sql
   CREATE DATABASE gill;
   GRANT ALL PRIVILEGES ON DATABASE gill TO postgres;
   ```
   
   ```shell
   sqlx migrate run --source crates/gill-db/migrations --database-url "postgres://postgres:postgres@localhost/gill"
   ```
5. Starting the container
   
   If you don't want to use the provided database container run 
   `docker compose up gill -d`, if you do `docker compose up -d`
