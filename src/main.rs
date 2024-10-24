use diesel::dsl::*;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};

use rand::rngs::OsRng;
use rand::RngCore;

use uuid::Uuid;

use dotenvy::dotenv;

mod code;
mod schema;

use code::Code;
use schema::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i64,
    pub uuid: Uuid,
    pub code: Code,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct Generated {
    pub uuid: Uuid,
    pub code: Code,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    assert!(dotenv().is_ok());

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let mut db = AsyncPgConnection::establish(&database_url).await.unwrap();

    let mut rng = OsRng;

    let mut bytes = [0u8; 16];
    rng.try_fill_bytes(&mut bytes).unwrap();
    let uuid = Uuid::from_bytes(bytes);

    let code = code::generate(&mut rng);

    let user = insert_into(users::table)
        .values(Generated { uuid, code })
        .returning(User::as_returning())
        .get_result(&mut db)
        .await // generates error here
        .expect("error while generating user");

    dbg!(user);
}
