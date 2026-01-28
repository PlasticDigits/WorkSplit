---
mode: split
target_file: src/taskService.ts
output_dir: src/taskService/
output_file: index.ts
output_files:
  - src/taskService/index.ts
  - src/taskService/types.ts
  - src/taskService/create.ts
  - src/taskService/read.ts
  - src/taskService/update.ts
  - src/taskService/delete.ts
---

# Split taskService.ts into CRUD Modules

Split the monolithic taskService.ts into a directory-based module with separate files for each operation type.

## File Structure

- `index.ts`: TaskService class, constructor, and public API that delegates to helper functions. Re-exports all types.
- `types.ts`: All interfaces, types, and the ServiceError class
- `create.ts`: Task creation functions (createTask, createTasksBatch, cloneTask helpers)
- `read.ts`: Task query and lookup functions (getTask, listTasks, searchTasks, getStats helpers)
- `update.ts`: Task update functions (updateTask, status changes, assignment, tags helpers)
- `delete.ts`: Task deletion functions (deleteTask, deleteCompletedTasks, etc. helpers)

## Function Signatures (REQUIRED)

### types.ts
```typescript
export type Priority = 'low' | 'medium' | 'high' | 'critical';
export type TaskStatus = 'pending' | 'in_progress' | 'completed' | 'cancelled';

export interface Task { ... }
export interface CreateTaskRequest { ... }
export interface UpdateTaskRequest { ... }
export interface TaskQuery { ... }
export interface TaskStats { ... }

export class ServiceError extends Error {
  constructor(
    public readonly code: 'NOT_FOUND' | 'VALIDATION' | 'DUPLICATE',
    message: string
  );
}
```

### create.ts
```typescript
import type { Task, CreateTaskRequest } from './types';
import { ServiceError } from './types';

export function createTask(
  tasks: Map<number, Task>,
  nextIdRef: { value: number },
  request: CreateTaskRequest
): Task;

export function createTasksBatch(
  tasks: Map<number, Task>,
  nextIdRef: { value: number },
  requests: CreateTaskRequest[]
): Task[];

export function cloneTask(
  tasks: Map<number, Task>,
  nextIdRef: { value: number },
  id: number,
  newTitle: string
): Task;
```

### read.ts
```typescript
import type { Task, TaskQuery, TaskStats } from './types';
import { ServiceError } from './types';

export function getTask(tasks: Map<number, Task>, id: number): Task;
export function listTasks(tasks: Map<number, Task>): Task[];
export function searchTasks(tasks: Map<number, Task>, query: TaskQuery): Task[];
export function getOverdueTasks(tasks: Map<number, Task>): Task[];
export function countTasks(tasks: Map<number, Task>): number;
export function getStats(tasks: Map<number, Task>): TaskStats;
```

### update.ts
```typescript
import type { Task, UpdateTaskRequest, Priority } from './types';
import { ServiceError } from './types';

export function updateTask(
  tasks: Map<number, Task>,
  id: number,
  request: UpdateTaskRequest
): Task;

export function startTask(tasks: Map<number, Task>, id: number): Task;
export function completeTask(tasks: Map<number, Task>, id: number): Task;
export function cancelTask(tasks: Map<number, Task>, id: number): Task;
export function reopenTask(tasks: Map<number, Task>, id: number): Task;
export function assignTask(tasks: Map<number, Task>, id: number, assignee: string): Task;
export function unassignTask(tasks: Map<number, Task>, id: number): Task;
export function addTag(tasks: Map<number, Task>, id: number, tag: string): Task;
export function removeTag(tasks: Map<number, Task>, id: number, tag: string): Task;
export function setPriority(tasks: Map<number, Task>, id: number, priority: Priority): Task;
export function setDueDate(tasks: Map<number, Task>, id: number, dueDate: Date | null): Task;
```

### delete.ts
```typescript
import type { Task, TaskQuery } from './types';
import { ServiceError } from './types';

export function deleteTask(tasks: Map<number, Task>, id: number): Task;
export function deleteCompletedTasks(tasks: Map<number, Task>): Task[];
export function deleteCancelledTasks(tasks: Map<number, Task>): Task[];
export function deleteTasksByQuery(tasks: Map<number, Task>, query: TaskQuery): Task[];
export function clearAllTasks(tasks: Map<number, Task>, nextIdRef: { value: number }): number;
```

## index.ts Structure

The index.ts should:
1. Import helper functions from submodules
2. Re-export all types from types.ts
3. Keep TaskService class with its impl block
4. Delegate each method to the appropriate helper function

Example pattern:
```typescript
import * as create from './create';
import * as read from './read';
import * as update from './update';
import * as del from './delete';
import type { Task, CreateTaskRequest, UpdateTaskRequest, TaskQuery, TaskStats, Priority } from './types';

// Re-export types
export type { Task, CreateTaskRequest, UpdateTaskRequest, TaskQuery, TaskStats, Priority, TaskStatus } from './types';
export { ServiceError } from './types';

export class TaskService {
  private tasks: Map<number, Task> = new Map();
  private nextId = { value: 1 };

  createTask(request: CreateTaskRequest): Task {
    return create.createTask(this.tasks, this.nextId, request);
  }
  // ... other delegating methods ...
}
```

## Notes

- Tests can be excluded (they're only for demonstration in the original file)
- Each submodule file should be self-contained with proper imports
- Use named exports, not default exports
- Use `export type` for type-only re-exports (TypeScript strict mode)
