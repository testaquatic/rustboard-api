use rustboard_domain::role::Role;

use crate::service::error::ServiceError;

pub mod comments;
pub mod error;
pub mod posts;
pub mod user;

fn check_ownership(
    resource_author_id: i64,
    requester_id: i64,
    requester_role: &Role,
) -> Result<(), ServiceError> {
    if resource_author_id == requester_id || *requester_role == Role::Admin {
        Ok(())
    } else {
        Err(ServiceError::Forbidden)
    }
}
