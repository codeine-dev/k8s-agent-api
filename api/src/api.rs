use rocket::{response::content, State, Rocket, Build};
use crate::schema::{schema, Schema, Context};


pub(crate) fn setup(rocket: Rocket<Build>, cfg: &crate::config::AppConfiguration) -> Rocket<Build> {
    rocket
        .manage(Context{
            cfg: cfg.to_owned()
        })
        .manage(schema())
        .manage(cfg.to_owned())
}

#[rocket::get("/")]
pub(crate) fn playground() -> content::RawHtml<String> {
    juniper_rocket::playground_source("/graphql", None)
}

#[rocket::get("/graphql?<request>")]
pub(crate) async fn get_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&*schema, &*context).await
}

#[rocket::post("/graphql", data = "<request>")]
pub(crate) async fn post_graphql_handler(
    context: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&*schema, &*context).await
}