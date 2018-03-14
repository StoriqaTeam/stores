INSERT INTO user_roles (user_id, role) VALUES (1, 'superuser') ON CONFLICT (user_id) DO NOTHING;
