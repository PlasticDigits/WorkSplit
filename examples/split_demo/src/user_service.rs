/// User service - a large file that should be split into modules
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct UserQuery {
    pub name_contains: Option<String>,
    pub active_only: bool,
}

#[derive(Debug)]
pub enum ServiceError {
    NotFound(i64),
    DuplicateEmail(String),
    ValidationError(String),
}

pub struct UserService {
    users: HashMap<i64, User>,
    next_id: i64,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    // === CREATE operations ===

    /// Create a new user
    pub fn create_user(&mut self, request: CreateUserRequest) -> Result<User, ServiceError> {
        // Check for duplicate email
        if self.users.values().any(|u| u.email == request.email) {
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
            id: self.next_id,
            name: request.name,
            email: request.email,
            active: true,
        };
        self.next_id += 1;
        self.users.insert(user.id, user.clone());
        Ok(user)
    }

    /// Create multiple users in batch
    pub fn create_users_batch(&mut self, requests: Vec<CreateUserRequest>) -> Result<Vec<User>, ServiceError> {
        let mut created = Vec::new();
        for request in requests {
            created.push(self.create_user(request)?);
        }
        Ok(created)
    }

    // === READ operations ===

    /// Get user by ID
    pub fn get_user(&self, id: i64) -> Result<User, ServiceError> {
        self.users.get(&id)
            .cloned()
            .ok_or(ServiceError::NotFound(id))
    }

    /// Get all users
    pub fn list_users(&self) -> Vec<User> {
        self.users.values().cloned().collect()
    }

    /// Search users by query
    pub fn search_users(&self, query: UserQuery) -> Vec<User> {
        self.users.values()
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

    /// Count total users
    pub fn count_users(&self) -> usize {
        self.users.len()
    }

    /// Count active users
    pub fn count_active_users(&self) -> usize {
        self.users.values().filter(|u| u.active).count()
    }

    // === UPDATE operations ===

    /// Update user by ID
    pub fn update_user(&mut self, id: i64, request: UpdateUserRequest) -> Result<User, ServiceError> {
        let user = self.users.get_mut(&id)
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
            if self.users.values().any(|u| u.id != id && u.email == email) {
                return Err(ServiceError::DuplicateEmail(email));
            }
            user.email = email;
        }
        if let Some(active) = request.active {
            user.active = active;
        }

        Ok(user.clone())
    }

    /// Deactivate user
    pub fn deactivate_user(&mut self, id: i64) -> Result<User, ServiceError> {
        self.update_user(id, UpdateUserRequest {
            name: None,
            email: None,
            active: Some(false),
        })
    }

    /// Activate user
    pub fn activate_user(&mut self, id: i64) -> Result<User, ServiceError> {
        self.update_user(id, UpdateUserRequest {
            name: None,
            email: None,
            active: Some(true),
        })
    }

    // === DELETE operations ===

    /// Delete user by ID
    pub fn delete_user(&mut self, id: i64) -> Result<User, ServiceError> {
        self.users.remove(&id)
            .ok_or(ServiceError::NotFound(id))
    }

    /// Delete all inactive users
    pub fn delete_inactive_users(&mut self) -> Vec<User> {
        let inactive_ids: Vec<i64> = self.users.values()
            .filter(|u| !u.active)
            .map(|u| u.id)
            .collect();
        
        inactive_ids.into_iter()
            .filter_map(|id| self.users.remove(&id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let mut service = UserService::new();
        let user = service.create_user(CreateUserRequest {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        }).unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, "alice@example.com");
        assert!(user.active);
    }

    #[test]
    fn test_get_user() {
        let mut service = UserService::new();
        let created = service.create_user(CreateUserRequest {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        }).unwrap();
        
        let fetched = service.get_user(created.id).unwrap();
        assert_eq!(fetched.name, "Bob");
    }

    #[test]
    fn test_update_user() {
        let mut service = UserService::new();
        let user = service.create_user(CreateUserRequest {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
        }).unwrap();
        
        let updated = service.update_user(user.id, UpdateUserRequest {
            name: Some("Charles".to_string()),
            email: None,
            active: None,
        }).unwrap();
        
        assert_eq!(updated.name, "Charles");
    }

    #[test]
    fn test_delete_user() {
        let mut service = UserService::new();
        let user = service.create_user(CreateUserRequest {
            name: "Dave".to_string(),
            email: "dave@example.com".to_string(),
        }).unwrap();
        
        service.delete_user(user.id).unwrap();
        assert!(service.get_user(user.id).is_err());
    }

    #[test]
    fn test_search_active_users() {
        let mut service = UserService::new();
        service.create_user(CreateUserRequest {
            name: "Active User".to_string(),
            email: "active@example.com".to_string(),
        }).unwrap();
        
        let user2 = service.create_user(CreateUserRequest {
            name: "Inactive User".to_string(),
            email: "inactive@example.com".to_string(),
        }).unwrap();
        service.deactivate_user(user2.id).unwrap();
        
        let active = service.search_users(UserQuery {
            name_contains: None,
            active_only: true,
        });
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Active User");
    }
}
