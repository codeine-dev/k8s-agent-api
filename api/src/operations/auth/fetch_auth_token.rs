use crate::operations::{run_gql, GQLClient};
use graphql_client::*;

//use reqwest;

#[allow(non_camel_case_types,dead_code)] type timestamptz = String;
#[allow(non_camel_case_types,dead_code)] type uuid = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/auth/schema.graphql",
    query_path = "graphql/auth/fetch_auth_token.gql",
    response_derives = "Debug"
)]
pub struct FetchAuthToken;

pub(crate) async fn resolve_auth_token(
    token: &str,
    cfg: &crate::config::AppConfiguration
) -> crate::Result<Option<crate::auth::AuthResponse>> {
    use fetch_auth_token::*;
    // this is the important line
    //let request_body = register_cluster::build_query(variables);

    let request_body = FetchAuthToken::build_query(Variables {
        token: token.to_string(),
    });

    let client = GQLClient::get_auth_server_client(cfg);

    let data = run_gql::<FetchAuthToken>(
        request_body,
        &client,
    )
    .await?;

    if let Some(token) = data.auth_access_token.get(0) {
        let account_roles: Vec<String> = token
            .account
            .account_has_roles
            .iter()
            .map(|role| role.role_value.clone())
            .collect();

        return Ok(Some(crate::auth::AuthResponse {
            user_id: Some(token.account.id.clone()),
            actor_id: match &token.overwrite_actor_id {
                Some(id) => Some(id.clone()),
                None => Some(token.account.actor_id.clone()),
            },
            role: None,
            cache_control: None,
            allowed_roles: match token.access_token_has_roles.is_empty() {
                true => Some(account_roles),
                false => Some(
                    token
                        .access_token_has_roles
                        .iter()
                        .map(|role| role.role_value.clone())
                        .filter(|role| account_roles.contains(role))
                        .collect(),
                ),
            },
        }))
    }

    Ok(None)
}
