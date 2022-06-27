use crate::{GraphQLSnafu, ReqwestSnafu};
use graphql_client::Response;
use snafu::{ResultExt};

pub mod auth;

#[derive(Debug)]
pub(crate) struct GQLClient {
    pub endpoint: String,
    pub headers: std::vec::Vec<(String, String)>,
}

impl GQLClient {
    pub fn get_auth_server_client(cfg: &crate::config::AppConfiguration) -> Self {
        GQLClient {
            endpoint: cfg.hasura_endpoint.to_owned(),
            headers: vec![
                ("x-hasura-admin-secret".to_string(), "p6vM/s18zDP2Z5zszd6+wWzrIeU=".to_string()),
                ("x-hasura-role".to_string(), "auth-server".to_string())
            ],
        }
    }

    pub fn get_request_builder(&self, client: Option<reqwest::Client>) -> reqwest::RequestBuilder {
        let client = match client {
            Some(c) => c,
            None => reqwest::Client::new(),
        };

        let mut builder = client.post(&self.endpoint);

        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }

        builder
    }
}

pub(crate) async fn run_gql<T: graphql_client::GraphQLQuery>(
    query: graphql_client::QueryBody<T::Variables>,
    gclient: &GQLClient,
) -> crate::Result<T::ResponseData> {
    let builder = gclient.get_request_builder(None);

    let resp = builder
        .json(&query)
        .send()
        .await
        .with_context(|e| ReqwestSnafu {
            msg: format!("GraphQL Request failed: '{}'", e),
        })?;

    if !resp.status().is_success() {
        return GraphQLSnafu {
            msg: format!(
                "GraphQL query to {} failed with {}",
                gclient.endpoint,
                resp.status()
            ),
        }
        .fail();
    }

    let graphql_response: Response<T::ResponseData> =
        resp.json().await.with_context(|e| ReqwestSnafu {
            msg: format!("GraphQL reading response failed: '{}'", e),
        })?;

    if graphql_response.errors.is_some() {
        return GraphQLSnafu {
            msg: format!(
                "GraphQL query to {} failed with {:?}",
                gclient.endpoint, graphql_response.errors
            ),
        }
        .fail();
    }

    if let Some(data) = graphql_response.data {
        return Ok(data);
    }

    return GraphQLSnafu {
        msg: format!(
            "GraphQL query to {} failed returned no errors and an empty data section",
            gclient.endpoint
        ),
    }
    .fail();
}
