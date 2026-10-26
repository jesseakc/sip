-- Remove the demonstration tenant that 004_seed_data.sql ships.
--
-- 004 creates a complete sample organisation: eight user accounts sharing one
-- placeholder password hash, plus sample locations, assets, parts, documents and
-- work orders. A deployment must start empty — the operator creates their own
-- accounts and imports their own data — so the sample tenant is deleted here
-- rather than left in place for someone to discover later.
--
-- Deliberately a forward migration, not an edit to 004: rewriting an applied
-- migration changes its checksum, and `sqlx migrate run` then refuses to run on
-- every database that has already applied it.
--
-- Scoped to the seeded organisation's fixed identifier. `TRUNCATE ... CASCADE`
-- would be shorter but would wipe every tenant on a database that also holds real
-- data, so it is not used here.
--
-- Deletion order is resolved at runtime rather than hard-coded. Not every foreign
-- key cascades — `part_usage.used_by_id`, `documents.uploaded_by_id`,
-- `teams.lead_id` and others do not — and `locations.parent_id` is RESTRICT, so a
-- fixed order would break the moment the schema gains a reference. The loop below
-- repeatedly deletes whatever is currently deletable and stops when a full pass
-- removes nothing, which is order-independent.

DO $$
DECLARE
    demo_org CONSTANT uuid := '00000000-0000-0000-0000-000000000001';
    tbl      text;
    removed  integer;
    pass     integer := 0;
BEGIN
    -- Join tables that carry no organization_id of their own, keyed through their
    -- parent. Removing them first unblocks the tables below.
    DELETE FROM part_usage
    WHERE work_order_id IN (SELECT id FROM work_orders WHERE organization_id = demo_org)
       OR used_by_id     IN (SELECT id FROM users      WHERE organization_id = demo_org);

    DELETE FROM team_members
    WHERE team_id IN (SELECT id FROM teams WHERE organization_id = demo_org)
       OR user_id IN (SELECT id FROM users WHERE organization_id = demo_org);

    DELETE FROM inspection_checklist_items
    WHERE inspection_id IN (
        SELECT i.id FROM inspections i
        JOIN work_orders w ON w.id = i.work_order_id
        WHERE w.organization_id = demo_org
    );

    DELETE FROM inspections
    WHERE work_order_id IN (SELECT id FROM work_orders WHERE organization_id = demo_org);

    -- Everything else that carries organization_id. Tables blocked by a child
    -- constraint raise a foreign-key error, which is swallowed so the next pass can
    -- retry them once the child is gone.
    LOOP
        pass := pass + 1;
        removed := 0;

        FOR tbl IN
            SELECT c.relname
            FROM pg_class c
            JOIN pg_attribute a ON a.attrelid = c.oid
            WHERE c.relkind = 'r'
              AND a.attname = 'organization_id'
              AND NOT a.attisdropped
              AND c.relname <> 'organizations'
            ORDER BY c.relname
        LOOP
            BEGIN
                EXECUTE format('DELETE FROM %I WHERE organization_id = $1', tbl)
                    USING demo_org;
                GET DIAGNOSTICS removed = ROW_COUNT;
            EXCEPTION WHEN foreign_key_violation THEN
                NULL;
            END;
        END LOOP;

        EXIT WHEN removed = 0;
        EXIT WHEN pass > 20;  -- safety valve; a tenant this tangled should be manual
    END LOOP;

    DELETE FROM organizations WHERE id = demo_org;
END
$$;
