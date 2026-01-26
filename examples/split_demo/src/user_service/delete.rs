use super::{User, ServiceError};
use std::collections::HashMap;

pub(crate) fn delete_user(
    users: &mut HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError> {
    users.remove(&id)
        .ok_or(ServiceError::NotFound(id))
}

pub(crate) fn delete_inactive_users(
    users: &mut HashMap<i64, User>,
) -> Vec<User> {
    let inactive_ids: Vec<i64> = users.values()
        .filter(|u| !u.active)
        .map(|u| u.id)
        .collect();
    
    inactive_ids.into_iter()
        .filter_map(|id| users.remove(&id))
        .collect()
}