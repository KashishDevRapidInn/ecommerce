use crate::db::PgPool;
use crate::schema::products::dsl as product_dsl;
use diesel::prelude::*;
use uuid::Uuid;

/******************************************/
// Adding seed data to products table
/******************************************/
pub fn seed_products(pool: PgPool) -> Result<(), diesel::result::Error> {
    let data = vec![
        (Uuid::new_v4(), "Laptop".to_string(), true, 50000),
        (Uuid::new_v4(), "Smart Phone".to_string(), true, 20000),
        (Uuid::new_v4(), "Dress".to_string(), true, 5000),
        (Uuid::new_v4(), "Bottle".to_string(), true, 1000),
        (Uuid::new_v4(), "Cap".to_string(), true, 500),
    ];
    let mut conn = pool.get().expect("Failed to get db connection from Pool");
    for (id, name, is_available, price) in data {
        diesel::insert_into(product_dsl::products)
            .values((
                product_dsl::id.eq(id),
                product_dsl::name.eq(name),
                product_dsl::is_available.eq(is_available),
                product_dsl::price.eq(price),
            ))
            .execute(&mut conn)?;
    }

    println!("successfully added products");
    Ok(())
}
