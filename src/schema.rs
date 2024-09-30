// @generated automatically by Diesel CLI.

diesel::table! {
    admins (id) {
        id -> Uuid,
        username -> Varchar,
        password_hash -> Varchar,
    }
}

diesel::table! {
    customers (id) {
        id -> Uuid,
        username -> Varchar,
        password_hash -> Varchar,
        email -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    orders (id) {
        id -> Uuid,
        customer_id -> Uuid,
        status -> Varchar,
        created_at -> Timestamp,
        product_id -> Uuid,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        name -> Varchar,
        is_available -> Bool,
        price -> Int4,
    }
}

diesel::joinable!(orders -> customers (customer_id));
diesel::joinable!(orders -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(
    admins,
    customers,
    orders,
    products,
);
