// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_status"))]
    pub struct OrderStatus;
}

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
    use diesel::sql_types::*;
    use super::sql_types::OrderStatus;

    orders (id) {
        id -> Uuid,
        customer_id -> Uuid,
        status -> OrderStatus,
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

diesel::allow_tables_to_appear_in_same_query!(admins, customers, orders, products,);
