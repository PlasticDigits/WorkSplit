/**
 * Task Service - A large file that should be split into modules
 *
 * This file contains a complete task management service with CRUD operations,
 * filtering, and statistics. It's intentionally large to demonstrate
 * WorkSplit's split functionality.
 */

// =============================================================================
// Types and Interfaces
// =============================================================================

/** Priority levels for tasks */
export type Priority = 'low' | 'medium' | 'high' | 'critical';

/** Status of a task */
export type TaskStatus = 'pending' | 'in_progress' | 'completed' | 'cancelled';

/** A task item */
export interface Task {
  id: number;
  title: string;
  description: string;
  priority: Priority;
  status: TaskStatus;
  assignee: string | null;
  dueDate: Date | null;
  tags: string[];
  createdAt: Date;
  updatedAt: Date;
}

/** Request to create a new task */
export interface CreateTaskRequest {
  title: string;
  description?: string;
  priority?: Priority;
  assignee?: string;
  dueDate?: Date;
  tags?: string[];
}

/** Request to update an existing task */
export interface UpdateTaskRequest {
  title?: string;
  description?: string;
  priority?: Priority;
  status?: TaskStatus;
  assignee?: string | null;
  dueDate?: Date | null;
  tags?: string[];
}

/** Query parameters for searching tasks */
export interface TaskQuery {
  titleContains?: string;
  status?: TaskStatus;
  priority?: Priority;
  assignee?: string;
  hasTag?: string;
  dueBefore?: Date;
  dueAfter?: Date;
}

/** Statistics about tasks */
export interface TaskStats {
  total: number;
  byStatus: Record<TaskStatus, number>;
  byPriority: Record<Priority, number>;
  overdue: number;
  unassigned: number;
}

/** Service error types */
export class ServiceError extends Error {
  constructor(
    public readonly code: 'NOT_FOUND' | 'VALIDATION' | 'DUPLICATE',
    message: string
  ) {
    super(message);
    this.name = 'ServiceError';
  }
}

// =============================================================================
// Task Service Implementation
// =============================================================================

export class TaskService {
  private tasks: Map<number, Task> = new Map();
  private nextId: number = 1;

  constructor() {
    this.tasks = new Map();
    this.nextId = 1;
  }

  // ===========================================================================
  // CREATE Operations
  // ===========================================================================

  /**
   * Create a new task
   * @param request - The task creation request
   * @returns The created task
   * @throws ServiceError if validation fails
   */
  createTask(request: CreateTaskRequest): Task {
    // Validate title
    if (!request.title || request.title.trim().length === 0) {
      throw new ServiceError('VALIDATION', 'Title cannot be empty');
    }

    if (request.title.length > 200) {
      throw new ServiceError('VALIDATION', 'Title cannot exceed 200 characters');
    }

    // Check for duplicate titles
    const existingTask = Array.from(this.tasks.values()).find(
      (t) => t.title.toLowerCase() === request.title.toLowerCase()
    );
    if (existingTask) {
      throw new ServiceError('DUPLICATE', `Task with title "${request.title}" already exists`);
    }

    // Validate due date if provided
    if (request.dueDate && request.dueDate < new Date()) {
      throw new ServiceError('VALIDATION', 'Due date cannot be in the past');
    }

    const now = new Date();
    const task: Task = {
      id: this.nextId++,
      title: request.title.trim(),
      description: request.description?.trim() ?? '',
      priority: request.priority ?? 'medium',
      status: 'pending',
      assignee: request.assignee?.trim() ?? null,
      dueDate: request.dueDate ?? null,
      tags: request.tags?.map((t) => t.trim().toLowerCase()) ?? [],
      createdAt: now,
      updatedAt: now,
    };

    this.tasks.set(task.id, task);
    return { ...task };
  }

  /**
   * Create multiple tasks in a batch
   * @param requests - Array of task creation requests
   * @returns Array of created tasks
   * @throws ServiceError if any validation fails (no tasks created on failure)
   */
  createTasksBatch(requests: CreateTaskRequest[]): Task[] {
    // Validate all first before creating any
    const titles = new Set<string>();
    for (const request of requests) {
      if (!request.title || request.title.trim().length === 0) {
        throw new ServiceError('VALIDATION', 'All tasks must have a title');
      }
      const normalizedTitle = request.title.toLowerCase();
      if (titles.has(normalizedTitle)) {
        throw new ServiceError('DUPLICATE', `Duplicate title in batch: "${request.title}"`);
      }
      titles.add(normalizedTitle);
    }

    // Create all tasks
    const created: Task[] = [];
    for (const request of requests) {
      created.push(this.createTask(request));
    }
    return created;
  }

  /**
   * Clone an existing task with a new title
   * @param id - The ID of the task to clone
   * @param newTitle - The title for the cloned task
   * @returns The cloned task
   */
  cloneTask(id: number, newTitle: string): Task {
    const original = this.getTask(id);
    return this.createTask({
      title: newTitle,
      description: original.description,
      priority: original.priority,
      assignee: original.assignee ?? undefined,
      dueDate: original.dueDate ?? undefined,
      tags: [...original.tags],
    });
  }

  // ===========================================================================
  // READ Operations
  // ===========================================================================

  /**
   * Get a task by ID
   * @param id - The task ID
   * @returns The task
   * @throws ServiceError if not found
   */
  getTask(id: number): Task {
    const task = this.tasks.get(id);
    if (!task) {
      throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
    }
    return { ...task };
  }

  /**
   * Get all tasks
   * @returns Array of all tasks
   */
  listTasks(): Task[] {
    return Array.from(this.tasks.values()).map((t) => ({ ...t }));
  }

  /**
   * Search tasks by query
   * @param query - The search query
   * @returns Array of matching tasks
   */
  searchTasks(query: TaskQuery): Task[] {
    return Array.from(this.tasks.values())
      .filter((task) => {
        // Filter by title
        if (query.titleContains) {
          const searchTerm = query.titleContains.toLowerCase();
          if (!task.title.toLowerCase().includes(searchTerm)) {
            return false;
          }
        }

        // Filter by status
        if (query.status && task.status !== query.status) {
          return false;
        }

        // Filter by priority
        if (query.priority && task.priority !== query.priority) {
          return false;
        }

        // Filter by assignee
        if (query.assignee !== undefined) {
          if (task.assignee !== query.assignee) {
            return false;
          }
        }

        // Filter by tag
        if (query.hasTag) {
          const tag = query.hasTag.toLowerCase();
          if (!task.tags.includes(tag)) {
            return false;
          }
        }

        // Filter by due date range
        if (query.dueBefore && task.dueDate) {
          if (task.dueDate > query.dueBefore) {
            return false;
          }
        }
        if (query.dueAfter && task.dueDate) {
          if (task.dueDate < query.dueAfter) {
            return false;
          }
        }

        return true;
      })
      .map((t) => ({ ...t }));
  }

  /**
   * Get tasks by status
   * @param status - The status to filter by
   * @returns Array of tasks with the given status
   */
  getTasksByStatus(status: TaskStatus): Task[] {
    return this.searchTasks({ status });
  }

  /**
   * Get tasks by priority
   * @param priority - The priority to filter by
   * @returns Array of tasks with the given priority
   */
  getTasksByPriority(priority: Priority): Task[] {
    return this.searchTasks({ priority });
  }

  /**
   * Get tasks assigned to a specific person
   * @param assignee - The assignee name
   * @returns Array of tasks assigned to the person
   */
  getTasksByAssignee(assignee: string): Task[] {
    return this.searchTasks({ assignee });
  }

  /**
   * Get overdue tasks
   * @returns Array of overdue tasks
   */
  getOverdueTasks(): Task[] {
    const now = new Date();
    return Array.from(this.tasks.values())
      .filter((task) => {
        return (
          task.dueDate !== null &&
          task.dueDate < now &&
          task.status !== 'completed' &&
          task.status !== 'cancelled'
        );
      })
      .map((t) => ({ ...t }));
  }

  /**
   * Count total tasks
   * @returns The total number of tasks
   */
  countTasks(): number {
    return this.tasks.size;
  }

  /**
   * Get task statistics
   * @returns Statistics about all tasks
   */
  getStats(): TaskStats {
    const tasks = Array.from(this.tasks.values());
    const now = new Date();

    const byStatus: Record<TaskStatus, number> = {
      pending: 0,
      in_progress: 0,
      completed: 0,
      cancelled: 0,
    };

    const byPriority: Record<Priority, number> = {
      low: 0,
      medium: 0,
      high: 0,
      critical: 0,
    };

    let overdue = 0;
    let unassigned = 0;

    for (const task of tasks) {
      byStatus[task.status]++;
      byPriority[task.priority]++;

      if (task.assignee === null) {
        unassigned++;
      }

      if (
        task.dueDate !== null &&
        task.dueDate < now &&
        task.status !== 'completed' &&
        task.status !== 'cancelled'
      ) {
        overdue++;
      }
    }

    return {
      total: tasks.length,
      byStatus,
      byPriority,
      overdue,
      unassigned,
    };
  }

  // ===========================================================================
  // UPDATE Operations
  // ===========================================================================

  /**
   * Update a task
   * @param id - The task ID
   * @param request - The update request
   * @returns The updated task
   * @throws ServiceError if not found or validation fails
   */
  updateTask(id: number, request: UpdateTaskRequest): Task {
    const task = this.tasks.get(id);
    if (!task) {
      throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
    }

    // Validate and apply title update
    if (request.title !== undefined) {
      if (request.title.trim().length === 0) {
        throw new ServiceError('VALIDATION', 'Title cannot be empty');
      }
      if (request.title.length > 200) {
        throw new ServiceError('VALIDATION', 'Title cannot exceed 200 characters');
      }
      // Check for duplicate
      const existing = Array.from(this.tasks.values()).find(
        (t) => t.id !== id && t.title.toLowerCase() === request.title!.toLowerCase()
      );
      if (existing) {
        throw new ServiceError('DUPLICATE', `Task with title "${request.title}" already exists`);
      }
      task.title = request.title.trim();
    }

    // Apply other updates
    if (request.description !== undefined) {
      task.description = request.description.trim();
    }

    if (request.priority !== undefined) {
      task.priority = request.priority;
    }

    if (request.status !== undefined) {
      task.status = request.status;
    }

    if (request.assignee !== undefined) {
      task.assignee = request.assignee?.trim() ?? null;
    }

    if (request.dueDate !== undefined) {
      task.dueDate = request.dueDate;
    }

    if (request.tags !== undefined) {
      task.tags = request.tags.map((t) => t.trim().toLowerCase());
    }

    task.updatedAt = new Date();
    return { ...task };
  }

  /**
   * Start working on a task (set status to in_progress)
   * @param id - The task ID
   * @returns The updated task
   */
  startTask(id: number): Task {
    const task = this.getTask(id);
    if (task.status === 'completed') {
      throw new ServiceError('VALIDATION', 'Cannot start a completed task');
    }
    if (task.status === 'cancelled') {
      throw new ServiceError('VALIDATION', 'Cannot start a cancelled task');
    }
    return this.updateTask(id, { status: 'in_progress' });
  }

  /**
   * Complete a task
   * @param id - The task ID
   * @returns The updated task
   */
  completeTask(id: number): Task {
    const task = this.getTask(id);
    if (task.status === 'cancelled') {
      throw new ServiceError('VALIDATION', 'Cannot complete a cancelled task');
    }
    return this.updateTask(id, { status: 'completed' });
  }

  /**
   * Cancel a task
   * @param id - The task ID
   * @returns The updated task
   */
  cancelTask(id: number): Task {
    return this.updateTask(id, { status: 'cancelled' });
  }

  /**
   * Reopen a completed or cancelled task
   * @param id - The task ID
   * @returns The updated task
   */
  reopenTask(id: number): Task {
    return this.updateTask(id, { status: 'pending' });
  }

  /**
   * Assign a task to someone
   * @param id - The task ID
   * @param assignee - The assignee name
   * @returns The updated task
   */
  assignTask(id: number, assignee: string): Task {
    if (!assignee || assignee.trim().length === 0) {
      throw new ServiceError('VALIDATION', 'Assignee cannot be empty');
    }
    return this.updateTask(id, { assignee });
  }

  /**
   * Unassign a task
   * @param id - The task ID
   * @returns The updated task
   */
  unassignTask(id: number): Task {
    return this.updateTask(id, { assignee: null });
  }

  /**
   * Add a tag to a task
   * @param id - The task ID
   * @param tag - The tag to add
   * @returns The updated task
   */
  addTag(id: number, tag: string): Task {
    const task = this.getTask(id);
    const normalizedTag = tag.trim().toLowerCase();
    if (task.tags.includes(normalizedTag)) {
      return task; // Tag already exists, no-op
    }
    return this.updateTask(id, { tags: [...task.tags, normalizedTag] });
  }

  /**
   * Remove a tag from a task
   * @param id - The task ID
   * @param tag - The tag to remove
   * @returns The updated task
   */
  removeTag(id: number, tag: string): Task {
    const task = this.getTask(id);
    const normalizedTag = tag.trim().toLowerCase();
    return this.updateTask(id, { tags: task.tags.filter((t) => t !== normalizedTag) });
  }

  /**
   * Set task priority
   * @param id - The task ID
   * @param priority - The new priority
   * @returns The updated task
   */
  setPriority(id: number, priority: Priority): Task {
    return this.updateTask(id, { priority });
  }

  /**
   * Set task due date
   * @param id - The task ID
   * @param dueDate - The new due date
   * @returns The updated task
   */
  setDueDate(id: number, dueDate: Date | null): Task {
    return this.updateTask(id, { dueDate });
  }

  // ===========================================================================
  // DELETE Operations
  // ===========================================================================

  /**
   * Delete a task by ID
   * @param id - The task ID
   * @returns The deleted task
   * @throws ServiceError if not found
   */
  deleteTask(id: number): Task {
    const task = this.tasks.get(id);
    if (!task) {
      throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
    }
    this.tasks.delete(id);
    return { ...task };
  }

  /**
   * Delete all completed tasks
   * @returns Array of deleted tasks
   */
  deleteCompletedTasks(): Task[] {
    const completedIds = Array.from(this.tasks.values())
      .filter((t) => t.status === 'completed')
      .map((t) => t.id);

    const deleted: Task[] = [];
    for (const id of completedIds) {
      const task = this.tasks.get(id);
      if (task) {
        this.tasks.delete(id);
        deleted.push({ ...task });
      }
    }
    return deleted;
  }

  /**
   * Delete all cancelled tasks
   * @returns Array of deleted tasks
   */
  deleteCancelledTasks(): Task[] {
    const cancelledIds = Array.from(this.tasks.values())
      .filter((t) => t.status === 'cancelled')
      .map((t) => t.id);

    const deleted: Task[] = [];
    for (const id of cancelledIds) {
      const task = this.tasks.get(id);
      if (task) {
        this.tasks.delete(id);
        deleted.push({ ...task });
      }
    }
    return deleted;
  }

  /**
   * Delete all tasks matching a query
   * @param query - The query to match tasks
   * @returns Array of deleted tasks
   */
  deleteTasksByQuery(query: TaskQuery): Task[] {
    const matching = this.searchTasks(query);
    const deleted: Task[] = [];
    for (const task of matching) {
      this.tasks.delete(task.id);
      deleted.push(task);
    }
    return deleted;
  }

  /**
   * Clear all tasks
   * @returns The number of deleted tasks
   */
  clearAllTasks(): number {
    const count = this.tasks.size;
    this.tasks.clear();
    this.nextId = 1;
    return count;
  }
}
