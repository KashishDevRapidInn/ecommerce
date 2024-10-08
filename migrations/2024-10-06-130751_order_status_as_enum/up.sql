CREATE TYPE order_status AS ENUM ('pending', 'shipped', 'delivered');

UPDATE orders
SET status = 'pending' 
WHERE status IS NULL OR status NOT IN ('pending', 'shipped', 'delivered');

ALTER TABLE orders
    ALTER COLUMN status TYPE order_status USING status::order_status;
