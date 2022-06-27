#[macro_use] extern crate rocket;

mod api;
mod schema;
mod operations;
mod error;
mod auth;
mod config;

pub use error::*;

use api::{setup, playground, get_graphql_handler, post_graphql_handler};

#[launch]
fn rocket() -> _ {
    let cfg = config::AppConfiguration{
        hasura_endpoint: "https://graphql-engine-k8p-afitzek.cloud.okteto.net/v1/graphql".to_string(),
        internal_token: "some_secret".to_string(),
    };
    setup(rocket::build(), &cfg).mount("/", routes![playground, get_graphql_handler, post_graphql_handler, auth::auth])
}
