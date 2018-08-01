-- This file should undo anything in `up.sql`
ALTER TABLE base_products DROP COLUMN IF EXISTS kafka_update_no;
