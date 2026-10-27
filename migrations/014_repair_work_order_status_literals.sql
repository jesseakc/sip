-- Repair values emitted by Debug formatting before canonical status encoding.
UPDATE work_orders SET status = CASE status WHEN 'INPROGRESS' THEN 'IN_PROGRESS' WHEN 'ONHOLD' THEN 'ON_HOLD' END
WHERE status IN ('INPROGRESS','ONHOLD');
UPDATE work_order_status_history SET from_status = CASE from_status WHEN 'INPROGRESS' THEN 'IN_PROGRESS' WHEN 'ONHOLD' THEN 'ON_HOLD' END
WHERE from_status IN ('INPROGRESS','ONHOLD');
UPDATE work_order_status_history SET to_status = CASE to_status WHEN 'INPROGRESS' THEN 'IN_PROGRESS' WHEN 'ONHOLD' THEN 'ON_HOLD' END
WHERE to_status IN ('INPROGRESS','ONHOLD');
