import type { Task, UpdateTaskRequest, Priority } from './types';
import { ServiceError } from './types';

export function updateTask(
  tasks: Map<number, Task>,
  id: number,
  request: UpdateTaskRequest
): Task {
  const task = tasks.get(id);
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
    const existing = Array.from(tasks.values()).find(
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

export function startTask(tasks: Map<number, Task>, id: number): Task {
  const task = getTask(tasks, id);
  if (task.status === 'completed') {
    throw new ServiceError('VALIDATION', 'Cannot start a completed task');
  }
  if (task.status === 'cancelled') {
    throw new ServiceError('VALIDATION', 'Cannot start a cancelled task');
  }
  return updateTask(tasks, id, { status: 'in_progress' });
}

export function completeTask(tasks: Map<number, Task>, id: number): Task {
  const task = getTask(tasks, id);
  if (task.status === 'cancelled') {
    throw new ServiceError('VALIDATION', 'Cannot complete a cancelled task');
  }
  return updateTask(tasks, id, { status: 'completed' });
}

export function cancelTask(tasks: Map<number, Task>, id: number): Task {
  return updateTask(tasks, id, { status: 'cancelled' });
}

export function reopenTask(tasks: Map<number, Task>, id: number): Task {
  return updateTask(tasks, id, { status: 'pending' });
}

export function assignTask(tasks: Map<number, Task>, id: number, assignee: string): Task {
  if (!assignee || assignee.trim().length === 0) {
    throw new ServiceError('VALIDATION', 'Assignee cannot be empty');
  }
  return updateTask(tasks, id, { assignee });
}

export function unassignTask(tasks: Map<number, Task>, id: number): Task {
  return updateTask(tasks, id, { assignee: null });
}

export function addTag(tasks: Map<number, Task>, id: number, tag: string): Task {
  const task = getTask(tasks, id);
  const normalizedTag = tag.trim().toLowerCase();
  if (task.tags.includes(normalizedTag)) {
    return task; // Tag already exists, no-op
  }
  return updateTask(tasks, id, { tags: [...task.tags, normalizedTag] });
}

export function removeTag(tasks: Map<number, Task>, id: number, tag: string): Task {
  const task = getTask(tasks, id);
  const normalizedTag = tag.trim().toLowerCase();
  return updateTask(tasks, id, { tags: task.tags.filter((t) => t !== normalizedTag) });
}

export function setPriority(tasks: Map<number, Task>, id: number, priority: Priority): Task {
  return updateTask(tasks, id, { priority });
}

export function setDueDate(tasks: Map<number, Task>, id: number, dueDate: Date | null): Task {
  return updateTask(tasks, id, { dueDate });
}

function getTask(tasks: Map<number, Task>, id: number): Task {
  const task = tasks.get(id);
  if (!task) {
    throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
  }
  return { ...task };
}