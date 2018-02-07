-- Your SQL goes here
CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    role VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
);

CREATE UNIQUE INDEX user_roles_user_id_idx ON user_roles (user_id);

SELECT diesel_manage_updated_at('user_roles');
