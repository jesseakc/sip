import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { PGlite } from '@electric-sql/pglite';
import { uuid_ossp } from '@electric-sql/pglite/contrib/uuid_ossp';
import { ltree } from '@electric-sql/pglite/contrib/ltree';
import { pg_trgm } from '@electric-sql/pglite/contrib/pg_trgm';
import { createHash } from 'node:crypto';
const migrations = new URL('../../migrations/', import.meta.url);
const checksum = text => createHash('sha384').update(text).digest('hex');
async function apply(db, name) {
  let sql = await readFile(new URL(name, migrations), 'utf8');
  // Only pgvector is substituted in this embedded PostgreSQL harness. Native
  // PostgreSQL + pgvector still needs the normal deployment integration check.
  sql = sql.replace('CREATE EXTENSION IF NOT EXISTS "vector";', '')
    .replace('embedding vector(1024)', 'embedding real[]')
    .replace(/CREATE INDEX idx_emb_vector[\s\S]*?;/, '');
  await db.exec(sql);
}
const fixtures = `
INSERT INTO organizations (id,name,slug) VALUES ('eeeeeeee-0000-0000-0000-000000000001','Regression tenant','regression-tenant');
INSERT INTO users (id,organization_id,email,name,password_hash) VALUES ('eeeeeeee-0000-0000-0000-000000000002','eeeeeeee-0000-0000-0000-000000000001','test@example.invalid','Regression user','unused');
INSERT INTO assets (id,organization_id,asset_type_id,name) VALUES ('eeeeeeee-0000-0000-0000-000000000003','eeeeeeee-0000-0000-0000-000000000001','10000000-0000-0000-0000-000000000001','Regression asset');
INSERT INTO work_orders (id,organization_id,asset_id,title,description,display_number,created_by_id) VALUES ('eeeeeeee-0000-0000-0000-000000000004','eeeeeeee-0000-0000-0000-000000000001','eeeeeeee-0000-0000-0000-000000000003','Regression work','test','WO-TEST','eeeeeeee-0000-0000-0000-000000000002');
INSERT INTO inspections (id,work_order_id) VALUES ('eeeeeeee-0000-0000-0000-000000000005','eeeeeeee-0000-0000-0000-000000000004');
INSERT INTO inspection_checklist_items (inspection_id,question,response_type,result) VALUES ('eeeeeeee-0000-0000-0000-000000000005','Check','NUMERIC','FAIL');
INSERT INTO ai_conversations (id,organization_id,user_id) VALUES ('eeeeeeee-0000-0000-0000-000000000006','eeeeeeee-0000-0000-0000-000000000001','eeeeeeee-0000-0000-0000-000000000002');
INSERT INTO ai_messages (organization_id,conversation_id,role,content) VALUES ('eeeeeeee-0000-0000-0000-000000000001','eeeeeeee-0000-0000-0000-000000000006','ASSISTANT','Retain this value');
INSERT INTO activities (organization_id,entity_type,entity_id,action,actor_type) VALUES ('eeeeeeee-0000-0000-0000-000000000001','asset','eeeeeeee-0000-0000-0000-000000000003','test','AI_AGENT');
`;
test('migration sequence preserves enum values and removes only the demo tenant', async () => {
  const db = new PGlite({ extensions: { 'uuid-ossp': uuid_ossp, ltree, pg_trgm } });
  try {
    const files = (await readdir(migrations)).filter(n => /^\d+.*\.sql$/.test(n)).sort();
    for (const file of files) {
      if (file.startsWith('008')) await db.exec(fixtures);
      await apply(db, file);
    }
    assert.deepEqual((await db.query('SELECT role,content FROM ai_messages')).rows,
      [{role:'ASSISTANT',content:'Retain this value'}]);
    assert.deepEqual((await db.query('SELECT actor_type FROM activities')).rows,[{actor_type:'AI_AGENT'}]);
    assert.deepEqual((await db.query('SELECT response_type,result FROM inspection_checklist_items')).rows,[{response_type:'NUMERIC',result:'FAIL'}]);
    assert.deepEqual((await db.query('SELECT slug FROM organizations')).rows,[{slug:'regression-tenant'}]);
    assert.equal((await db.query('SELECT count(*)::int AS n FROM users')).rows[0].n,1);
    assert.equal((await db.query('SELECT count(*)::int AS n FROM assets')).rows[0].n,1);
    await apply(db,'011_remove_demo_seed_data.sql'); // absent demo tenant is harmless
    const reconciliation = await readFile(new URL('../../scripts/reconcile-reviewed-migrations.sql', import.meta.url),'utf8');
    await db.exec('CREATE TABLE _sqlx_migrations (version bigint PRIMARY KEY, checksum bytea NOT NULL, success boolean NOT NULL)');
    const current = checksum(await readFile(new URL('008_convert_enums_to_text.sql', migrations),'utf8'));
    await db.query('INSERT INTO _sqlx_migrations VALUES (8,decode($1,\'hex\'),true)',[current]);
    await db.exec(reconciliation);
    await db.exec("UPDATE _sqlx_migrations SET checksum = decode('00','hex')");
    await assert.rejects(db.exec(reconciliation), /unknown checksum/);
    await db.exec('ROLLBACK');
    await db.query('UPDATE _sqlx_migrations SET checksum=decode($1,\'hex\')',[current]);
    await db.exec('ALTER TABLE ai_messages DROP COLUMN role');
    await assert.rejects(db.exec(reconciliation), /restore their definitions and values/);
    await db.exec('ROLLBACK');
  } finally { await db.close(); }
});
