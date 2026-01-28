import type { Task, TaskQuery, TaskStats } from './types';
import { ServiceError } from './types';

export function getTask(tasks: Map<number, Task>, id: number): Task {
  const task = tasks.get(id);
  if (!task) {
    throw new ServiceError('NOT_FOUND', `Task with ID ${id} not found`);
  }
  return { ...task };
}

export function listTasks(tasks: Map<number, Task>): Task[] {
  return Array.from(tasks.values()).map((t) => ({ ...t }));
}

export function searchTasks(tasks: Map<number, Task>, query: TaskQuery): Task[] {
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

export function getTasksByStatus(tasks: Map<number, Task>, status: TaskStatus): Task[] {
  return searchTasks(tasks, { status });
}

export function getTasksByPriority(tasks: Map<number, Task>, priority: Priority): Task[] {
  return searchTasks(tasks, { priority });
}

export function getTasksByAssignee(tasks: Map<number, Task>, assignee: string): Task[] {
  return searchTasks(tasks, { assignee });
}

export function getOverdueTasks(tasks: Map<number, Task>): Task[] {
  const now = new Date();
  return Array.from(tasks.values())
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

export function countTasks(tasks: Map<number, Task>): number {
  return tasks.size;
}

export function getStats(tasks: Map<number, Task>): TaskStats {
  const tasksList = Array.from(tasks.values());
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

  for (const task of tasksList) {
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
    total: tasksList.length,
    byStatus,
    byPriority,
    overdue,
    unassigned,
  };
}