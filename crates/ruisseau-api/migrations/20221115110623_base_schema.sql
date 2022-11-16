-- Add migration script here
CREATE TABLE users
(
    id       SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(255) NOT NULL
);

CREATE TABLE organisation
(
    id       SERIAL PRIMARY KEY,
    owner_id INT REFERENCES users (id)
);

CREATE TABLE repository
(
    id       SERIAL PRIMARY KEY,
    name     VARCHAR(100),
    owner_id INT REFERENCES users (id)
);

CREATE TABLE org_repository
(
    id       SERIAL PRIMARY KEY,
    name     VARCHAR(100),
    org_id INT REFERENCES organisation (id)
);


CREATE TABLE public_key
(
    id       SERIAL PRIMARY KEY,
    key      TEXT,
    owner_id INT REFERENCES users (id)
);