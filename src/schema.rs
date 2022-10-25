// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        author -> Text,
        title -> Varchar,
        subtitle -> Varchar,
        body -> Text,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tokens (id) {
        id -> Uuid,
        user -> Uuid,
        expiration -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password -> Text,
        about -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(tokens -> users (user));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    tokens,
    users,
);
