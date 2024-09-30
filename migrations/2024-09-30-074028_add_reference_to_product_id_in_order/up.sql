ALTER TABLE orders DROP COLUMN product_id;
ALTER TABLE orders ADD COLUMN product_id UUID;
ALTER TABLE orders
ADD CONSTRAINT fk_product
FOREIGN KEY (product_id)
REFERENCES products(id);
