/// Re-exports the user management module
pub mod models;
pub mod services;

/// Re-export User type from models
pub use models::User;
/// Re-export NewUser type from models
pub use models::NewUser;
/// Re-export UserService from services
pub use services::UserService;