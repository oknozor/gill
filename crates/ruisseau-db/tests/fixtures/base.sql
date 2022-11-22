-- Alice
INSERT INTO users (id, username, email)
VALUES (0, 'alice', 'alice@wonder.land');

INSERT INTO repository (name, owner_id)
VALUES ('linux', 0);

-- Okno
INSERT INTO users (id, username, email)
VALUES (1, 'okno', 'oknozor@ruisseau.org');

-- A repository with some branches
INSERT INTO repository (name, owner_id)
VALUES ('ruisseau', 1);

INSERT INTO branch (name, repository_id, is_default)
VALUES ('main', 2, true);

INSERT INTO branch (name, repository_id)
VALUES ('feature', 2);

INSERT INTO branch (name, repository_id)
VALUES ('fix', 2);

