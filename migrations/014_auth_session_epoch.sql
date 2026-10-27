-- Session revocation.
--
-- Tokens are stateless, so nothing could invalidate one before its expiry: a
-- deactivated user, or a user who logged out everywhere, kept working until the
-- token aged out. Each user now carries an epoch that is embedded in every token
-- they are issued. Increasing it invalidates every token issued before the bump.
--
-- Default 0 means existing users are unaffected until their epoch is raised, so
-- deploying this does not sign everybody out.

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS auth_epoch INTEGER NOT NULL DEFAULT 0
        CHECK (auth_epoch >= 0);
