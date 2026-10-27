-- Log out all previously issued tokens by advancing the user session epoch.
ALTER TABLE users ADD COLUMN auth_epoch INTEGER NOT NULL DEFAULT 0 CHECK (auth_epoch >= 0);
