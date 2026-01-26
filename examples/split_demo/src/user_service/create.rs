use super::{User, CreateUserRequest, ServiceError};
use std::collections::HashMap;

pub(crate) fn create_user(
    users: &mut HashMap<i64, User>,
    next_id: &mut i64,
    request: CreateUserRequest,
) -> Result<User, ServiceError> {
    // Check for duplicate email
    if users.values().any(|u| u.email == request.email) {
        return Err(ServiceError::DuplicateEmail(request.email));
    }

    // Validate
    if request.name.is_empty() {
        return Err(ServiceError::ValidationError("Name cannot be empty".to_string()));
    }
    if !request.email.contains('@') {
        return Err(ServiceError::ValidationError("Invalid email format".to_string()));
    }

    let user = User {
        id: *next_id,
        name: request.name,
        email: request.email,
        active: true,
    };
    *next_id += 1;
    users.insert(user.id, user.clone());
    Ok(user)
}

pub(crate) fn create_users_batch(
    users: &mut HashMap<i64, User>,
    next_id: &mut i64,
    requests: Vec<CreateUserRequest>,
) -> Result<Vec<User>, ServiceError> {
    let mut created = Vec::new();
    for request in requests {
        created.push(create_user(users, next_id, request)?);
    }
    Ok(created)
}