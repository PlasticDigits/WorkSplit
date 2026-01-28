import type { Task, CreateTaskRequest } from './types';
import { ServiceError } from './types';

export function createTask(
  tasks: Map<number, Task>,
  nextIdRef: { value: number },
  request: CreateTaskRequest
): Task {
  // Validate title
  if (!request.title || request.title.trim().length === 0) {
    throw new ServiceError('VALIDATION', 'Title cannot be empty');
  }

  if (request.title.length > 200) {
    throw new ServiceError('VALIDATION', 'Title cannot exceed 200 characters');
  }

  // Check for duplicate titles
  const existingTask = Array.from(tasks.values()).find(
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
    id: nextIdRef.value++,
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

  tasks.set(task.id, task);
  return { ...task };
}

export function createTasksBatch(
  tasks: Map<number, Task>,
  nextIdRef: { value: number },
  requests: CreateTaskRequest[]
): Task[] {
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
    created.push(createTask(tasks, nextIdRef, request));
  }
  return created;
}

export function cloneTask(
  tasks: Map<number, Task>,
  nextIdRef: { value: number },
  id: number,
  newTitle: string
): Task {
  const original = getTask(tasks, id);
  return createTask(tasks, nextIdRef, {
    title: newTitle,
    description: original.description,
    priority: original.priority,
    assignee: original.assignee ?? undefined,
    dueDate: original.dueDate ?? undefined,
    tags: [...original.tags],
  });
}

function getTask(tasks: Map<number, Task>, id: number): Task {
  const task = tasks.get(id);
  if (!task) {
    throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
  }
  return { ...task };
}