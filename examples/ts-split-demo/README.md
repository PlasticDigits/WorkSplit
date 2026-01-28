# TypeScript Split Demo

This example demonstrates WorkSplit's **split mode** for breaking a large TypeScript file into a modular directory structure.

## What's Being Split

The `src/taskService.ts` file (~500 lines) contains a complete task management service with:
- Type definitions (Priority, TaskStatus, Task, etc.)
- ServiceError class
- TaskService class with CRUD operations
- Create operations (createTask, createTasksBatch, cloneTask)
- Read operations (getTask, listTasks, searchTasks, getStats)
- Update operations (updateTask, startTask, completeTask, assignTask, tags, etc.)
- Delete operations (deleteTask, deleteCompletedTasks, clearAllTasks)

## Target Structure

After running the split job, the monolithic file will become:

```
src/taskService/
├── index.ts      # TaskService class, re-exports
├── types.ts      # Interfaces and ServiceError
├── create.ts     # Create operation helpers
├── read.ts       # Read operation helpers
├── update.ts     # Update operation helpers
└── delete.ts     # Delete operation helpers
```

## Usage

1. **Ensure Ollama is running** with a capable model (e.g., qwen3)

2. **Validate the job**:
   ```bash
   worksplit validate
   ```

3. **Run the split job**:
   ```bash
   worksplit run
   ```

4. **Check status**:
   ```bash
   worksplit status -v
   ```

## Why Split Mode?

Split mode is ideal when:
- A file has grown too large (300+ lines)
- Functions/methods naturally group into categories (CRUD, etc.)
- You want to improve maintainability and testability
- The file has clear separation of concerns

## Job Configuration

The split job (`jobs/split_001_taskService.md`) specifies:
- `mode: split` - Use split workflow
- `target_file` - The file to split
- `output_files` - The resulting module files
- Function signatures for each output file (helps LLM generate correct interfaces)

## Notes

- The original `taskService.ts` file remains unchanged until you manually remove it
- The split generates standalone helper functions that the class delegates to
- This pattern keeps the public API identical while improving internal organization
