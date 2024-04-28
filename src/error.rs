use http::status::StatusCode;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
    #[error("Room Does Not Exist")]
    InvalidPath,
    #[error("Websocket Connection Failed")]
    WebSocketConnectionFailed,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidPath => StatusCode::NOT_FOUND,
            Self::WebSocketConnectionFailed => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Room Does Not Exist")]
    RoomDoesNotExist,
    #[cfg(feature = "ssr")]
    #[error("Database Error: {0}")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("User Has Already Joined the Channel")]
    AddChannelError,
    #[error("Unable To Remove User From The Channel")]
    RemoveChannelError,
    #[error("User Does Not Exist")]
    UserDoesNotExist,
    #[error("Email Has Been Taken")]
    EmailTaken,
    #[error("Other: {0}")]
    Other(String),
}
