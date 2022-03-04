mod data_access;
mod entity;

pub mod prelude {
    pub use crate::data_access::{model::*, DatabaseEngine, DatabaseError, PostgresDb};
}
