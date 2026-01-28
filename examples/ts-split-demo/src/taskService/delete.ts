import type { Task, TaskQuery } from './types';
import { ServiceError } from './types';

export function deleteTask(
  tasks: Map<number, Task>,
  id: number
): Task {
  const task = tasks.get(id);
  if (!task) {
    throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
  }
  tasks.delete(id);
  return { ...task };
}

export function deleteCompletedTasks(
  tasks: Map<number, Task>
): Task[] {
  const completedIds = Array.from(tasks.values())
    .filter((t) => t.status === 'completed')
    .map((t) => t.id);

  const deleted: Task[] = [];
  for (const id of completedIds) {
    const task = tasks.get(id);
    if (task) {
      tasks.delete(id);
      deleted.push({ ...task });
    }
  }
  return deleted;
}

export function deleteCancelledTasks(
  tasks: Map<number, Task>
): Task[] {
  const cancelledIds = Array.from(tasks.values())
    .filter((t) => t.status === 'cancelled')
    .map((t) => t.id);

  const deleted: Task[] = [];
  for (const id of cancelledIds) {
    const task = tasks.get(id);
    if (task) {
      tasks.delete(id);
      deleted.push({ ...task });
    }
  }
  return deleted;
}

export function deleteTasksByQuery(
  tasks: Map<number, Task>,
  query: TaskQuery
): Task[] {
  const matching = searchTasks(tasks, query);
  const deleted: Task[] = [];
  for (const task of matching) {
    tasks.delete(task.id);
    deleted.push(task);
  }
  return deleted;
}

export function clearAllTasks(
  tasks: Map<number, Task>,
  nextIdRef: { value: number }
): number {
  const count = tasks.size;
  tasks.clear();
  nextIdRef.value = 1;
  return count;
}

function searchTasks(
  tasks: Map<number, Task>,
  query: TaskQuery
): Task[] {
  return Array.from(tasks.values())
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