---
mode: split
target_file: src/user_service.rs
output_dir: src/user_service/
output_file: mod.rs
output_files:
  - src/user_service/mod.rs
  - src/user_service/create.rs
  - src/user_service/read.rs
  - src/user_service/update.rs
  - src/user_service/delete.rs
---

# Split user_service.rs into CRUD Modules

Split the monolithic user_service.rs into a directory-based module with separate files for each CRUD operation type.

## File Structure

- `mod.rs`: User struct, types, errors, UserService struct, and public API that delegates to helpers
- `create.rs`: User creation functions
- `read.rs`: User query and lookup functions  
- `update.rs`: User update functions
- `delete.rs`: User deletion functions

## Function Signatures (REQUIRED)

### create.rs
```rust
use super::{User, CreateUserRequest, ServiceError};
use std::collections::HashMap;

pub(crate) fn create_user(
    users: &mut HashMap<i64, User>,
    next_id: &mut i64,
    request: CreateUserRequest,
) -> Result<User, ServiceError>

pub(crate) fn create_users_batch(
    users: &mut HashMap<i64, User>,
    next_id: &mut i64,
    requests: Vec<CreateUserRequest>,
) -> Result<Vec<User>, ServiceError>
```

### read.rs
```rust
use super::{User, UserQuery, ServiceError};
use std::collections::HashMap;

pub(crate) fn get_user(
    users: &HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError>

pub(crate) fn list_users(
    users: &HashMap<i64, User>,
) -> Vec<User>

pub(crate) fn search_users(
    users: &HashMap<i64, User>,
    query: UserQuery,
) -> Vec<User>

pub(crate) fn count_users(users: &HashMap<i64, User>) -> usize

pub(crate) fn count_active_users(users: &HashMap<i64, User>) -> usize
```

### update.rs
```rust
use super::{User, UpdateUserRequest, ServiceError};
use std::collections::HashMap;

pub(crate) fn update_user(
    users: &mut HashMap<i64, User>,
    id: i64,
    request: UpdateUserRequest,
) -> Result<User, ServiceError>

pub(crate) fn deactivate_user(
    users: &mut HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError>

pub(crate) fn activate_user(
    users: &mut HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError>
```

### delete.rs
```rust
use super::{User, ServiceError};
use std::collections::HashMap;

pub(crate) fn delete_user(
    users: &mut HashMap<i64, User>,
    id: i64,
) -> Result<User, ServiceError>

pub(crate) fn delete_inactive_users(
    users: &mut HashMap<i64, User>,
) -> Vec<User>
```

## mod.rs Structure

The mod.rs should:
1. Define User, CreateUserRequest, UpdateUserRequest, UserQuery, ServiceError
2. Import and use the helper functions from submodules
3. Keep UserService struct with its impl block
4. Delegate each method to the appropriate helper function

Example:
```rust
mod create;
mod read;
mod update;
mod delete;

// ... types ...

impl UserService {
    pub fn create_user(&mut self, request: CreateUserRequest) -> Result<User, ServiceError> {
        create::create_user(&mut self.users, &mut self.next_id, request)
    }
    // ... other delegating methods ...
}
```

## Notes

- Tests should remain in mod.rs (copy from original)
- Each submodule file should be self-contained
- Use `pub(crate)` visibility for helper functions
