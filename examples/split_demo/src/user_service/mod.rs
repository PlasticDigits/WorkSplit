mod create;
mod read;
mod update;
mod delete;

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
    users: std::collections::HashMap<i64, User>,
    next_id: i64,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    // === CREATE operations ===

    pub fn create_user(&mut self, request: CreateUserRequest) -> Result<User, ServiceError> {
        create::create_user(&mut self.users, &mut self.next_id, request)
    }

    pub fn create_users_batch(&mut self, requests: Vec<CreateUserRequest>) -> Result<Vec<User>, ServiceError> {
        create::create_users_batch(&mut self.users, &mut self.next_id, requests)
    }

    // === READ operations ===

    pub fn get_user(&self, id: i64) -> Result<User, ServiceError> {
        read::get_user(&self.users, id)
    }

    pub fn list_users(&self) -> Vec<User> {
        read::list_users(&self.users)
    }

    pub fn search_users(&self, query: UserQuery) -> Vec<User> {
        read::search_users(&self.users, query)
    }

    pub fn count_users(&self) -> usize {
        read::count_users(&self.users)
    }

    pub fn count_active_users(&self) -> usize {
        read::count_active_users(&self.users)
    }

    // === UPDATE operations ===

    pub fn update_user(&mut self, id: i64, request: UpdateUserRequest) -> Result<User, ServiceError> {
        update::update_user(&mut self.users, id, request)
    }

    pub fn deactivate_user(&mut self, id: i64) -> Result<User, ServiceError> {
        update::deactivate_user(&mut self.users, id)
    }

    pub fn activate_user(&mut self, id: i64) -> Result<User, ServiceError> {
        update::activate_user(&mut self.users, id)
    }

    // === DELETE operations ===

    pub fn delete_user(&mut self, id: i64) -> Result<User, ServiceError> {
        delete::delete_user(&mut self.users, id)
    }

    pub fn delete_inactive_users(&mut self) -> Vec<User> {
        delete::delete_inactive_users(&mut self.users)
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