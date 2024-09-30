CREATE TABLE orders (
    id uuid PRIMARY KEY NOT NULL,
    customer_id uuid REFERENCES customers(id),
    status VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
