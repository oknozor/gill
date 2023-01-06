CREATE TABLE users
(
    id              SERIAL PRIMARY KEY,
    username        VARCHAR(255) NOT NULL,
    domain          VARCHAR(255) NOT NULL,
    email           VARCHAR(255),
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
    id                SERIAL PRIMARY KEY,
    activity_pub_id   VARCHAR(255) NOT NULL UNIQUE,
    name              VARCHAR(100) NOT NULL,
    summary           TEXT,
    private           BOOLEAN               DEFAULT false NOT NULL,
    inbox_url         VARCHAR(255) NOT NULL UNIQUE,
    outbox_url        VARCHAR(255) NOT NULL UNIQUE,
    followers_url     VARCHAR(255) NOT NULL UNIQUE,
    attributed_to     VARCHAR(255) NOT NULL,
    clone_uri         VARCHAR(255) NOT NULL,
    public_key        TEXT         NOT NULL,
    private_key       TEXT,
    published         TIMESTAMP    NOT NULL DEFAULT now(),
    ticket_tracked_by VARCHAR(255) NOT NULL,
    send_patches_to   VARCHAR(255) NOT NULL,
    domain            VARCHAR(255) NOT NULL,
    is_local          BOOLEAN      NOT NULL,
    item_count        INT          NOT NULL DEFAULT 0,
    CONSTRAINT Unique_Name_For_Repository UNIQUE (name, attributed_to)
);

CREATE TABLE branch
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    name          VARCHAR(255)                   NOT NULL,
    is_default    BOOLEAN                        NOT NULL DEFAULT FALSE,
    PRIMARY KEY (name, repository_id)
);

CREATE TYPE pull_request_state AS ENUM ('Open', 'Closed', 'Merged');

CREATE TABLE pull_request
(
    number        INT                            NOT NULL,
    repository_id INT REFERENCES repository (id) NOT NULL,
    opened_by     INT REFERENCES users (id)      NOT NULL,
    title         VARCHAR(255)                   NOT NULL,
    description   TEXT,
    base          VARCHAR(255)                   NOT NULL,
    compare       VARCHAR(255)                   NOT NULL,
    state         pull_request_state             NOT NULL DEFAULT 'Open',
    CONSTRAINT base_key FOREIGN KEY (base, repository_id) REFERENCES branch (name, repository_id),
    PRIMARY KEY (number, repository_id)
);

CREATE TABLE pull_request_comment
(
    id            SERIAL,
    number        INT                            NOT NULL,
    repository_id INT REFERENCES repository (id) NOT NULL,
    created_by    INT REFERENCES users (id)      NOT NULL,
    content       TEXT                           NOT NULL,
    CONSTRAINT pull_request_key FOREIGN KEY (number, repository_id) REFERENCES pull_request (number, repository_id)
);

CREATE TYPE issue_state AS ENUM ('Open', 'Closed');

CREATE TABLE issue
(
    -- Local
    number          INT                                             NOT NULL,
    repository_id   INT REFERENCES repository (id)                  NOT NULL,
    title           VARCHAR(255)                                    NOT NULL,
    content         TEXT                                            NOT NULL,
    state           issue_state                                     NOT NULL DEFAULT 'Open',
    opened_by       INT REFERENCES users (id)                       NOT NULL,

    -- Federated
    activity_pub_id VARCHAR(255)                                    NOT NULL UNIQUE,
    context         VARCHAR(255)                                    NOT NULL,
    attributed_to   VARCHAR(255) references users (activity_pub_id) NOT NULL,
    media_type      VARCHAR(255)                                    NOT NULL,
    published       TIMESTAMP                                       NOT NULL DEFAULT now(),
    followers_url   VARCHAR(255)                                    NOT NULL,
    team            VARCHAR(255)                                    NOT NULL,
    replies         VARCHAR(255)                                    NOT NULL,
    history         VARCHAR(255)                                    NOT NULL,
    dependants      VARCHAR(255)                                    NOT NULL,
    dependencies    VARCHAR(255)                                    NOT NULL,
    resolved_by     VARCHAR(255) REFERENCES users (activity_pub_id),
    resolved        TIMESTAMP,
    is_local        BOOLEAN                                         NOT NULL,
    PRIMARY KEY (number, repository_id)
);

CREATE TABLE issue_comment
(
    id            SERIAL,
    number        INT                            NOT NULL,
    repository_id INT REFERENCES repository (id) NOT NULL,
    created_by    INT REFERENCES users (id)      NOT NULL,
    content       TEXT                           NOT NULL,
    CONSTRAINT issue_key FOREIGN KEY (number, repository_id) REFERENCES issue (number, repository_id)
);

CREATE TABLE issue_subscriber
(
    number        INT                       NOT NULL,
    repository_id INT                       NOT NULL,
    subscriber    INT REFERENCES users (id) NOT NULL,
    CONSTRAINT issue_key FOREIGN KEY (number, repository_id) REFERENCES issue (number, repository_id),
    PRIMARY KEY (number, repository_id, subscriber)
);

CREATE TABLE repository_star
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    starred_by    INT REFERENCES users (id)      NOT NULL,
    PRIMARY KEY (starred_by, repository_id)
);

CREATE TABLE repository_fork
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    fork_id       INT REFERENCES repository (id) NOT NULL,
    forked_by     INT REFERENCES users (id)      NOT NULL,
    PRIMARY KEY (forked_by, repository_id)
);

CREATE TABLE repository_watch
(
    repository_id INT REFERENCES repository (id) NOT NULL,
    watched_by    INT REFERENCES users (id)      NOT NULL,
    PRIMARY KEY (watched_by, repository_id)
);

CREATE TABLE ssh_key
(
    id       SERIAL PRIMARY KEY,
    key      TEXT                      NOT NULL,
    name     VARCHAR(255)              NOT NULL UNIQUE,
    key_type VARCHAR(255)              NOT NULL,
    owner_id INT REFERENCES users (id) NOT NULL
);


