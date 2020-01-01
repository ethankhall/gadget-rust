table! {
    use diesel::sql_types::{Int4, Varchar, Timestamp, Nullable};

    redirects (redirect_id) {
        redirect_id -> Int4,
        public_ref -> Varchar,
        alias -> Varchar,
        destination -> Varchar,
        created_on -> Timestamp,
        created_by -> Nullable<Varchar>,
    }
}

table! {
    usage (usage_id) {
        usage_id -> Int4,
        redirect_id -> Int4,
        clicks -> Int4,
    }
}

joinable!(usage -> redirects (redirect_id));

allow_tables_to_appear_in_same_query!(redirects, usage,);
