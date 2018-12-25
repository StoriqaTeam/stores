ALTER TABLE base_products ADD COLUMN store_status VARCHAR NOT NULL DEFAULT 'draft';

UPDATE base_products SET store_status=bp_store.status FROM (SELECT id, status FROM stores) AS bp_store WHERE bp_store.id=base_products.store_id;
