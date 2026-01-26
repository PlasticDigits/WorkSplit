use std::fmt;

/// Represents a user in the system
#[derive(Debug, Clone)]
pub struct User {
    /// Unique identifier for the user
    pub id: u64,
    /// User's full name
    pub name: String,
    /// User's email address
    pub email: String,
    /// Indicates if the user account is active
    pub active: bool,
}

/// Represents a new user to be created
#[derive(Debug, Clone)]
pub struct NewUser {
    /// User's full name
    pub name: String,
    /// User's email address
    pub email: String,
}

impl User {
    /// Creates a new user with the specified id, name, and email
    /// The user is automatically marked as active
    pub fn new(id: u64, name: String, email: String) -> Self {
        User {
            id,
            name,
            email,
            active: true,
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User {{ id: {}, name: {}, email: {}, active: {} }}",
            self.id, self.name, self.email, self.active
        )
    }
}