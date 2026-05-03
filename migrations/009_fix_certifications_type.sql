-- Fix certifications column type: JSONB[] -> JSONB for sqlx compatibility
ALTER TABLE users ALTER COLUMN certifications DROP DEFAULT;
ALTER TABLE users ALTER COLUMN certifications TYPE jsonb USING to_jsonb(certifications);
ALTER TABLE users ALTER COLUMN certifications SET DEFAULT '[]'::jsonb;
