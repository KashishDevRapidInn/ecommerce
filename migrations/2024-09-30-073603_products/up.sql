CREATE TABLE products (
    id uuid PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    is_available BOOLEAN NOT NULL,
    price INTEGER NOT NULL
);
