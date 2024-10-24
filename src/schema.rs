// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use crate::code::CodeType as Passcode;

    users (id) {
        id -> Int8,
        uuid -> Uuid,
        code -> Passcode,
    }
}
