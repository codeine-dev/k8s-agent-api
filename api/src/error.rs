use rocket::{response::Responder};
use snafu::prelude::*;


#[derive(Debug, Snafu,Responder)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("GraphQL request failed {msg}"))]
    #[response(status = 500, content_type = "plain")]
    GraphQLError {
        msg: String
    },
    #[snafu(display("Reqwest request failed {msg}"))]
    #[response(status = 500, content_type = "plain")]
    ReqwestError {
        msg: String,
        #[response(ignore)]
        source: reqwest::Error 
    },
    #[snafu(display("Unauthorized: {msg} [{detail}]"))]
    #[response(status = 401, content_type = "plain")]
    Unauthorized {
        msg: String,
        #[response(ignore)]
        detail: String
    },
    #[snafu(display("Failed to hash password: {msg}"))]
    #[response(status = 500, content_type = "plain")]
    ArgonHash {
        msg: String
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn encrypt(data: String) -> String {
    // TOOD: encrypt error details
    data
}

/*
pub struct ErrorApiResponse {
    pub msg: String,
    pub status: Status,
    pub details: Option<String>
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> Result<Response<'static>, Status> {

        Response::build_from(Json(ErrorApiResponse{
            msg: format!("{}", self),
            status
        }))
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
*/