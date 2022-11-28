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

CREATE table user_activity (
    id SERIAL PRIMARY KEY,
    summary TEXT,
    type activity_type,
    local BOOLEAN NOT NULL
);

CREATE TABLE user_follow
(
    id SERIAL PRIMARY KEY,
    user_id     INT REFERENCES users (id),
    follower_id INT REFERENCES users (id)
);

CREATE TABLE organisation
(
    id       SERIAL PRIMARY KEY,
    owner_id INT REFERENCES users (id)
);

CREATE TABLE repository
(
    id       SERIAL PRIMARY KEY,
    name     VARCHAR(100)              NOT NULL,
    owner_id INT REFERENCES users (id) NOT NULL,
    CONSTRAINT Unique_Name_For_Repository UNIQUE (name, owner_id)
);

CREATE TABLE branch
(
    repository_id INT          NOT NULL REFERENCES repository (id),
    name          VARCHAR(255) NOT NULL,
    is_default    BOOLEAN      NOT NULL DEFAULT FALSE,
    PRIMARY KEY (name, repository_id)
);

CREATE TABLE org_repository
(
    id     SERIAL PRIMARY KEY,
    name   VARCHAR(100),
    org_id INT REFERENCES organisation (id)
);


CREATE TABLE public_key
(
    id       SERIAL PRIMARY KEY,
    key      TEXT,
    owner_id INT REFERENCES users (id)
);