use super::{User, UpdateUserRequest, ServiceError};
use std::collections::HashMap;

pub(crate) fn update_user(
    users: &mut HashMap<i64, User>,
    id: i64,
    request: UpdateUserRequest,
) -> Result<User, ServiceError> {
    let user = users.get_mut(&id)
        .ok_or(ServiceError::NotFound(id))?;

    if let Some(name) = request.name {
        if name.is_empty() {
            return Err(ServiceError::ValidationError("Name cannot be empty".to_string()));
        }
        user.name = name;
    }
    if let Some(email) = request.email {
        if !email.contains('@') {
            return Err(ServiceError::ValidationError("Invalid email format".to_string()));
        }
        // Check for duplicate
        if users.values().any(|u| u.id != id && u.email == email) {
            return Err(ServiceError::DuplicateEmail(email));
        }
        user.email = email;
    }
    if let Some(active) = request.active {
        user.active = active;
    }

    Ok(user.clone())
}

pub(crate) fn deactivate_user(
    users: &mut HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError> {
    update_user(users, id, UpdateUserRequest {
        name: None,
        email: None,
        active: Some(false),
    })
}

pub(crate) fn activate_user(
    users: &mut HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError> {
    update_user(users, id, UpdateUserRequest {
        name: None,
        email: None,
        active: Some(true),
    })
}