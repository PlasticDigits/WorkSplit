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