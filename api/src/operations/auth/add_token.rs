use crate::operations::{run_gql, GQLClient};
use graphql_client::*;

//use reqwest;

#[allow(non_camel_case_types,dead_code)] type timestamptz = String;
#[allow(non_camel_case_types,dead_code)] type uuid = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/auth/schema.graphql",
    query_path = "graphql/auth/add_token.gql",
    response_derives = "Debug"
)]
pub struct AddToken;

pub(crate) async fn add_auth_token(
    account_id: &str,
    token: &str,
    cfg: &crate::config::AppConfiguration
) -> crate::Result<()> {
    use add_token::*;
    // this is the important line
    //let request_body = register_cluster::build_query(variables);

    let request_body = AddToken::build_query(Variables {
        account: account_id.to_string(),
        token: token.to_string(),
    });

    let client = GQLClient::get_auth_server_client(cfg);

    let data = run_gql::<AddToken>(
        request_body,
        &client,
    )
    .await?;

    if data.insert_auth_access_token_one.is_some() {
        Ok(())
    } else {
        crate::GraphQLSnafu{
            msg: "Unable to create auth token".to_string()
        }.fail()
    }
}
