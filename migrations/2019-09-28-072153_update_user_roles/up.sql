ALTER TABLE user_roles ADD CONSTRAINT role UNIQUE (user_id, name, data);
