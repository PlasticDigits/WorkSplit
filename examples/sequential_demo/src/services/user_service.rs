use crate::models::{User, NewUser};

/// Manages user operations in memory
pub struct UserService {
    /// Storage for all users
    users: Vec<User>,
    /// Next available user ID
    next_id: u64,
}

impl UserService {
    /// Creates a new user service
    pub fn new() -> Self {
        UserService {
            users: Vec::new(),
            next_id: 1,
        }
    }

    /// Creates a new user with the given details
    /// Returns the created user
    pub fn create_user(&mut self, new_user: NewUser) -> User {
        let user = User::new(self.next_id, new_user.name, new_user.email);
        self.next_id += 1;
        self.users.push(user.clone());
        user
    }

    /// Retrieves a user by ID
    /// Returns Some(&User) if found, None otherwise
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }

    /// Returns a slice of all users
    pub fn list_users(&self) -> &[User] {
        &self.users
    }

    /// Deactivates a user by ID
    /// Returns true if user was found and deactivated, false otherwise
    pub fn deactivate_user(&mut self, id: u64) -> bool {
        if let Some(user) = self.users.iter_mut().find(|u| u.id == id) {
            user.active = false;
            true
        } else {
            false
        }
    }
}