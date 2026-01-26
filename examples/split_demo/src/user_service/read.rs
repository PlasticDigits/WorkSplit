use super::{User, UserQuery, ServiceError};
use std::collections::HashMap;

pub(crate) fn get_user(
    users: &HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError> {
    users.get(&id)
        .cloned()
        .ok_or(ServiceError::NotFound(id))
}

pub(crate) fn list_users(
    users: &HashMap<i64, User>,
) -> Vec<User> {
    users.values().cloned().collect()
}

pub(crate) fn search_users(
    users: &HashMap<i64, User>,
    query: UserQuery,
) -> Vec<User> {
    users.values()
        .filter(|u| {
            if query.active_only && !u.active {
                return false;
            }
            if let Some(ref name_contains) = query.name_contains {
                if !u.name.to_lowercase().contains(&name_contains.to_lowercase()) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

pub(crate) fn count_users(users: &HashMap<i64, User>) -> usize {
    users.len()
}

pub(crate) fn count_active_users(users: &HashMap<i64, User>) -> usize {
    users.values().filter(|u| u.active).count()
}