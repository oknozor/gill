CREATE TABLE users
(
    id              SERIAL PRIMARY KEY,
    username        VARCHAR(255) NOT NULL,
    domain          VARCHAR(255) NOT NULL,
    email           VARCHAR(255) NOT NULL,
    public_key      TEXT         NOT NULL,
    private_key     TEXT,
    activity_pub_id VARCHAR(255) NOT NULL UNIQUE,
    inbox_url       VARCHAR(255) NOT NULL UNIQUE,
    outbox_url      VARCHAR(255) NOT NULL UNIQUE,
    followers_url   VARCHAR(255) NOT NULL UNIQUE,
    is_local        BOOLEAN      NOT NULL
);

CREATE TYPE activity_type AS ENUM ('sad', 'ok', 'happy');

CREATE table user_activity
(
    id      SERIAL PRIMARY KEY,
    summary TEXT,
    type    activity_type,
    local   BOOLEAN NOT NULL
);

CREATE TABLE user_follow
(
    id          SERIAL PRIMARY KEY,
    user_id     INT REFERENCES users (id) NOT NULL,
    follower_id INT REFERENCES users (id) NOT NULL
);

CREATE TABLE organisation
(
    id       SERIAL PRIMARY KEY,
    owner_id INT REFERENCES users (id) NOT NULL
);

CREATE TABLE repository
(
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(100)              NOT NULL,
    description TEXT,
    private     BOOLEAN DEFAULT false     NOT NULL,
    owner_id    INT REFERENCES users (id) NOT NULL,
    CONSTRAINT Unique_Name_For_Repository UNIQUE (name, owner_id)
);

CREATE TABLE branch
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    name          VARCHAR(255)                   NOT NULL,
    is_default    BOOLEAN                        NOT NULL DEFAULT FALSE,
    PRIMARY KEY (name, repository_id)
);

CREATE TABLE repository_star
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    starred_by    VARCHAR(255) REFERENCES users (activity_pub_id),
    PRIMARY KEY (starred_by, repository_id)
);

CREATE TABLE repository_fork
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    forked_by     VARCHAR(255) REFERENCES users (activity_pub_id),
    PRIMARY KEY (forked_by, repository_id)
);

CREATE TABLE repository_watch
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    watched_by    VARCHAR(255) REFERENCES users (activity_pub_id),
    PRIMARY KEY (watched_by, repository_id)
);

CREATE TABLE org_repository
(
    id     SERIAL PRIMARY KEY,
    name   VARCHAR(100),
    org_id INT REFERENCES organisation (id) NOT NULL
);


CREATE TABLE public_key
(
    id       SERIAL PRIMARY KEY,
    key      TEXT,
    owner_id INT REFERENCES users (id) NOT NULL
);