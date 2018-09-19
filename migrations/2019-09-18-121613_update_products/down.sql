-- This file should undo anything in `up.sql`
ALTER TABLE products DROP COLUMN IF EXISTS pre_order;
ALTER TABLE products DROP COLUMN IF EXISTS pre_order_days;