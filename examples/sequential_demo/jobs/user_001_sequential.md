---
output_dir: src/
output_file: models/user.rs
output_files:
  - src/models/user.rs
  - src/services/user_service.rs
  - src/lib.rs
sequential: true
---

# User Management Module (Sequential)

Generate a complete user management module with model, service, and module exports.

## Files to Generate

### src/models/user.rs
Create a User struct and related types:
- `User` struct with fields: `id: u64`, `name: String`, `email: String`, `active: bool`
- `NewUser` struct for creating users (without id)
- Implement `Display` for `User`
- Add a simple constructor `User::new(id, name, email)` that sets active to true

### src/services/user_service.rs
Create a UserService that manages users:
- `UserService` struct with a `Vec<User>` storage and `next_id: u64` counter
- `new()` constructor
- `create_user(&mut self, new_user: NewUser) -> User` - creates and stores a user
- `get_user(&self, id: u64) -> Option<&User>` - finds user by id
- `list_users(&self) -> &[User]` - returns all users
- `deactivate_user(&mut self, id: u64) -> bool` - sets active to false, returns success

Import the User types from the models module.

### src/lib.rs
Create the library entry point:
- Declare `pub mod models;` and `pub mod services;`
- Re-export `User`, `NewUser` from models
- Re-export `UserService` from services

## Notes
- Keep it simple - no external dependencies
- Each file should be self-contained but reference the previous files appropriately
