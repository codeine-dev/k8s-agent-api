use juniper::{
    graphql_object, graphql_value, EmptySubscription, FieldError, FieldResult, GraphQLObject,
    RootNode,
};

use crate::error::encrypt;

#[derive(Clone, Debug)]
pub(crate) struct Context {
    pub cfg: crate::config::AppConfiguration,
}

impl juniper::Context for Context {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn api_version() -> String {
        "0.0.1".to_string()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn register(ctx: &Context, email: String, password: String) -> FieldResult<String> {
        // TODO: validate email

        // Generate access token
        let (plain, hashed) = crate::auth::new_token(&ctx.cfg).map_err(|e| {
            log::warn!("Failed to create a user token: {}", e);
            let details = encrypt(format!("{}", e));
            FieldError::new("Failed to create a user token", graphql_value!({ "details": details }))
        })?;

        // Create password hash
        let hashed_pwd = crate::auth::hash_password(&password).map_err(|e| {
            log::warn!("Failed to hash user password: {}", e);
            let details = encrypt(format!("{}", e));
            FieldError::new("Failed to hash user password", graphql_value!({ "details": details }))
        })?;

        crate::operations::auth::register_user::register_user_account(
            &email,
            &hashed_pwd,
            "user", // TODO get default roles from cfg
            &hashed,
            &ctx.cfg,
        )
        .await.map_err(|e| {
            log::warn!("Failed to regsiter user: {}", e);
            let details = encrypt(format!("{}", e));
            FieldError::new("Failed to hash user password", graphql_value!({ "details": details }))
        })?;
        Ok(plain)
    }

    async fn login(ctx: &Context, email: String, password: String) -> FieldResult<String> {
        // TODO: validate email

        let error = "User not available, or password not correct";

        let user = crate::operations::auth::get_user_by_email::get_user_from_mail(&email, &ctx.cfg)
            .await
            .map_err(|e| {
                log::warn!("Failed to find user by email: {}", e);
                let details = encrypt(format!("{}", e));
                FieldError::new(error, graphql_value!({ "details": details }))
            })?;

        let ok = crate::auth::validate_password(&password, &user.password).map_err(|e| {
            log::warn!("Failed to validate password for user: {}", e);
            let details = encrypt(format!("{}", e));
            FieldError::new(error, graphql_value!({ "details": details }))
        })?;

        if ok {
            let (plain, hashed) = crate::auth::new_token(&ctx.cfg).map_err(|e| {
                log::warn!("Failed to generate a new token for user: {}", e);
                let details = encrypt(format!("{}", e));
                FieldError::new(error, graphql_value!({ "details": details }))
            })?;

            crate::operations::auth::add_auth_token(&user.id, &hashed, &ctx.cfg)
                .await
                .map_err(|e| {
                    log::warn!("Failed to add a new token for user: {}", e);
                    let details = encrypt(format!("{}", e));
                    FieldError::new(error, graphql_value!({ "details": details }))
                })?;

            Ok(plain.to_string())
        } else {
            let details = encrypt("No specific details here, just the wrong password".to_owned());
            Err(FieldError::new(error, graphql_value!({ "details": details })))
        }
    }
}

pub(crate) type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;
pub(crate) fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}

#[derive(GraphQLObject)]
//#[graphql(description="Information about a cluster registration")]
struct ClusterRegistration {
    #[graphql(name = "name", description = "The cluster name")]
    pub name: String,
    #[graphql(name = "id", description = "The cluster id")]
    pub id: String,
}
