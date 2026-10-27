-- Restore the four columns destroyed by the previous version of migration 008.
--
-- Old 008 ended with `DROP TYPE ... CASCADE`. CASCADE drops dependent columns, and
-- these four were never converted, so they were removed along with their data:
--   activities.actor_type
--   ai_messages.role
--   inspection_checklist_items.response_type
--   inspection_checklist_items.result
--
-- 008 itself is now catalog-driven and cannot repeat the mistake, so this repairs
-- only databases that already ran the old version. On a fresh database the columns
-- still exist and every statement below is a no-op.
--
-- Data that was in these columns is gone; they come back with defaults, not values.
-- Anything that must be recovered has to come from an authoritative source.

ALTER TABLE activities
    ADD COLUMN IF NOT EXISTS actor_type TEXT NOT NULL DEFAULT 'HUMAN';

ALTER TABLE inspection_checklist_items
    ADD COLUMN IF NOT EXISTS response_type TEXT NOT NULL DEFAULT 'PASS_FAIL';

ALTER TABLE inspection_checklist_items
    ADD COLUMN IF NOT EXISTS result TEXT;

-- `role` had no default in the original schema, so it is added nullable, backfilled
-- for any surviving rows, then constrained — which works whether or not the table
-- has rows.
ALTER TABLE ai_messages
    ADD COLUMN IF NOT EXISTS role TEXT;

UPDATE ai_messages SET role = 'USER' WHERE role IS NULL;

ALTER TABLE ai_messages
    ALTER COLUMN role SET NOT NULL;
