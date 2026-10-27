-- Repair work-order statuses that were written with a non-canonical encoding.
--
-- The API encoded statuses with `format!("{:?}").to_uppercase()`. For the
-- multi-word variants that produces "INPROGRESS" and "ONHOLD", while the parser
-- that reads them back expects "IN_PROGRESS" and "ON_HOLD". Rows written that way
-- could not round-trip: they were read as an unrecognised value rather than the
-- status the user actually set.
--
-- New writes use WorkOrderStatus::as_str(), so this only repairs existing rows.
-- Single-word variants are unaffected because Debug and as_str agree on them.

UPDATE work_orders
SET status = CASE status
                 WHEN 'INPROGRESS' THEN 'IN_PROGRESS'
                 WHEN 'ONHOLD'     THEN 'ON_HOLD'
             END
WHERE status IN ('INPROGRESS', 'ONHOLD');

UPDATE work_order_status_history
SET from_status = CASE from_status
                      WHEN 'INPROGRESS' THEN 'IN_PROGRESS'
                      WHEN 'ONHOLD'     THEN 'ON_HOLD'
                  END
WHERE from_status IN ('INPROGRESS', 'ONHOLD');

UPDATE work_order_status_history
SET to_status = CASE to_status
                    WHEN 'INPROGRESS' THEN 'IN_PROGRESS'
                    WHEN 'ONHOLD'     THEN 'ON_HOLD'
                END
WHERE to_status IN ('INPROGRESS', 'ONHOLD');
