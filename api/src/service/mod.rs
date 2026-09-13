use rustboard_domain::{error::service_error::ServiceError, role::Role};

pub mod comment;
pub mod post;
pub mod user;

fn check_ownership(
    post_author_id: i64,
    requester_id: i64,
    requester_role: &Role,
) -> Result<(), ServiceError> {
    if post_author_id == requester_id || *requester_role == Role::Admin {
        Ok(())
    } else {
        Err(ServiceError::Forbidden)
    }
}
