use crate::operations::{run_gql, GQLClient};
use graphql_client::*;

//use reqwest;

#[allow(non_camel_case_types,dead_code)] type timestamptz = String;
#[allow(non_camel_case_types,dead_code)] type uuid = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/auth/schema.graphql",
    query_path = "graphql/auth/register_user.gql",
    response_derives = "Debug"
)]
pub struct RegisterUser;

pub(crate) async fn register_user_account(
    email: &str,
    password_hash: &str,
    role: &str,
    token: &str,
    cfg: &crate::config::AppConfiguration
) -> crate::Result<()> {
    use register_user::*;
    // this is the important line
    //let request_body = register_cluster::build_query(variables);

    let request_body = RegisterUser::build_query(Variables {
        email: email.to_lowercase(),
        password: password_hash.to_string(),
        role: role.to_string(),
        token: token.to_string(),
    });

    let client = GQLClient::get_auth_server_client(cfg);

    let data = run_gql::<RegisterUser>(
        request_body,
        &client,
    )
    .await?;

    if data.insert_auth_account_one.is_some() {
        Ok(())
    } else {
        crate::GraphQLSnafu{
            msg: "Unable to create user".to_string()
        }.fail()
    }
}
