use crate::operations::{run_gql, GQLClient};
use graphql_client::*;

//use reqwest;

#[allow(non_camel_case_types,dead_code)] type timestamptz = String;
#[allow(non_camel_case_types,dead_code)] type uuid = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/auth/schema.graphql",
    query_path = "graphql/auth/get_user_by_email.gql",
    response_derives = "Debug"
)]
pub struct GetUserByEmail;

pub(crate) struct UserEntry {
    pub id: String,
    #[allow(dead_code)] pub email: String,
    pub password: String,
    #[allow(dead_code)] pub roles: Vec<String>,
}

impl From<&get_user_by_email::GetUserByEmailAuthAccount> for UserEntry {
    fn from(source: &get_user_by_email::GetUserByEmailAuthAccount) -> Self {
        UserEntry {
            id: source.id.to_owned(),
            email: source.email.to_owned(),
            password: source.password.to_owned(),
            roles: source
                .account_has_roles
                .iter()
                .map(|r| r.role_value.to_owned())
                .collect(),
        }
    }
}

pub(crate) async fn get_user_from_mail(
    email: &str,
    cfg: &crate::config::AppConfiguration,
) -> crate::Result<UserEntry> {
    use get_user_by_email::*;
    // this is the important line
    //let request_body = register_cluster::build_query(variables);

    let request_body = GetUserByEmail::build_query(Variables {
        email: email.to_lowercase(),
    });

    let client = GQLClient::get_auth_server_client(cfg);

    let data = run_gql::<GetUserByEmail>(request_body, &client).await?;

    if let Some(user) = data.auth_account.get(0) {
        Ok(user.into())
    } else {
        crate::GraphQLSnafu {
            msg: "Unable to find user from email".to_string(),
        }
        .fail()
    }
}
