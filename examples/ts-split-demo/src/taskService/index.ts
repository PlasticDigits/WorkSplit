import * as create from './create';
import * as read from './read';
import * as update from './update';
import * as del from './delete';
import type { Task, CreateTaskRequest, UpdateTaskRequest, TaskQuery, TaskStats, Priority, TaskStatus } from './types';

// Re-export types
export type { Task, CreateTaskRequest, UpdateTaskRequest, TaskQuery, TaskStats, Priority, TaskStatus } from './types';
export { ServiceError } from './types';

export class TaskService {
  private tasks: Map<number, Task> = new Map();
  private nextId = { value: 1 };

  constructor() {
    this.tasks = new Map();
    this.nextId = { value: 1 };
  }

  createTask(request: CreateTaskRequest): Task {
    return create.createTask(this.tasks, this.nextId, request);
  }

  createTasksBatch(requests: CreateTaskRequest[]): Task[] {
    return create.createTasksBatch(this.tasks, this.nextId, requests);
  }

  cloneTask(id: number, newTitle: string): Task {
    return create.cloneTask(this.tasks, this.nextId, id, newTitle);
  }

  getTask(id: number): Task {
    return read.getTask(this.tasks, id);
  }

  listTasks(): Task[] {
    return read.listTasks(this.tasks);
  }

  searchTasks(query: TaskQuery): Task[] {
    return read.searchTasks(this.tasks, query);
  }

  getTasksByStatus(status: TaskStatus): Task[] {
    return this.searchTasks({ status });
  }

  getTasksByPriority(priority: Priority): Task[] {
    return this.searchTasks({ priority });
  }

  getTasksByAssignee(assignee: string): Task[] {
    return this.searchTasks({ assignee });
  }

  getOverdueTasks(): Task[] {
    return read.getOverdueTasks(this.tasks);
  }

  countTasks(): number {
    return read.countTasks(this.tasks);
  }

  getStats(): TaskStats {
    return read.getStats(this.tasks);
  }

  updateTask(id: number, request: UpdateTaskRequest): Task {
    return update.updateTask(this.tasks, id, request);
  }

  startTask(id: number): Task {
    return update.startTask(this.tasks, id);
  }

  completeTask(id: number): Task {
    return update.completeTask(this.tasks, id);
  }

  cancelTask(id: number): Task {
    return update.cancelTask(this.tasks, id);
  }

  reopenTask(id: number): Task {
    return update.reopenTask(this.tasks, id);
  }

  assignTask(id: number, assignee: string): Task {
    return update.assignTask(this.tasks, id, assignee);
  }

  unassignTask(id: number): Task {
    return update.unassignTask(this.tasks, id);
  }

  addTag(id: number, tag: string): Task {
    return update.addTag(this.tasks, id, tag);
  }

  removeTag(id: number, tag: string): Task {
    return update.removeTag(this.tasks, id, tag);
  }

  setPriority(id: number, priority: Priority): Task {
    return update.setPriority(this.tasks, id, priority);
  }

  setDueDate(id: number, dueDate: Date | null): Task {
    return update.setDueDate(this.tasks, id, dueDate);
  }

  deleteTask(id: number): Task {
    return del.deleteTask(this.tasks, id);
  }

  deleteCompletedTasks(): Task[] {
    return del.deleteCompletedTasks(this.tasks);
  }

  deleteCancelledTasks(): Task[] {
    return del.deleteCancelledTasks(this.tasks);
  }

  deleteTasksByQuery(query: TaskQuery): Task[] {
    return del.deleteTasksByQuery(this.tasks, query);
  }

  clearAllTasks(): number {
    return del.clearAllTasks(this.tasks, this.nextId);
  }
}