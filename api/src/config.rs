#[derive(Clone, Debug)]
pub(crate) struct AppConfiguration {
    pub hasura_endpoint: String,
    pub internal_token: String
}