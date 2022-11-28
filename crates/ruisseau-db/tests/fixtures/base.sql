-- Alice
INSERT INTO users (id, username, email, domain, is_local, followers_url, inbox_url, outbox_url, private_key, public_key,
                   activity_pub_id)
VALUES (0, 'alice', 'alice@wonder.land', 'myinstance.org', true, 'https://myinstance.org/alice/followsers/',
        'https://myinstance.org/alice/inbox/', 'https://myinstance.org/alice/outbox/', 'private_key', 'public_key',
        'https://myinstance.org/alice');

INSERT INTO repository (name, owner_id)
VALUES ('linux', 0);

-- Okno
INSERT INTO users (id, username, email, domain, is_local, followers_url, inbox_url, outbox_url, private_key, public_key,
                   activity_pub_id)
VALUES (1, 'okno', 'oknozor@ruisseau.org', 'myinstance.org', false, 'https://okno.org/oknozor/followsers/',
        'https://okno.org/oknozor/inbox/', 'https://okno.org/oknozor/outbox/', 'private_key', 'public_key',
        'https://okno.org/oknozor');

-- A repository with some branches
INSERT INTO repository (name, owner_id)
VALUES ('ruisseau', 1);

INSERT INTO branch (name, repository_id, is_default)
VALUES ('main', 2, true);

INSERT INTO branch (name, repository_id)
VALUES ('feature', 2);

INSERT INTO branch (name, repository_id)
VALUES ('fix', 2);

INSERT INTO users (id, username, email, domain, is_local, followers_url, inbox_url, outbox_url, private_key, public_key,
                   activity_pub_id)
VALUES (2, 'tom', 'tom@wonder.land', 'myinstance.org', true, 'https://myinstance.org/tom/followsers/',
        'https://myinstance.org/tom/inbox/', 'https://myinstance.org/tom/outbox/', 'private_key', 'public_key',
        'https://myinstance.org/tom');

INSERT INTO users (id, username, email, domain, is_local, followers_url, inbox_url, outbox_url, private_key, public_key,
                   activity_pub_id)
VALUES (3, 'jerry', 'jerry@wonder.land', 'myinstance.org', true, 'https://myinstance.org/jerry/followsers/',
        'https://myinstance.org/jerry/inbox/', 'https://myinstance.org/jerry/outbox/', 'private_key', 'public_key',
        'https://myinstance.org/jerry');

INSERT INTO user_followers(user_id, follower_id)
VALUES (2, 0);
INSERT INTO user_followers(user_id, follower_id)
VALUES (3, 0);


