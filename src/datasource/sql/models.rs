use super::schema::redirects;

#[derive(Queryable)]
pub struct Redirects {
    pub id: i32,
    pub alias: String,
    pub destination: String,
    pub created_by: String,
}

#[derive(Insertable)]
#[table_name = "redirects"]
pub struct NewRedirects<'a> {
    pub alias: &'a str,
    pub destination: &'a str,
    pub created_by: &'a str,
}