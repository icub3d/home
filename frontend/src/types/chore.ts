export interface Chore {
  id: string;
  description: string;
  assigned_to: string;
  completed: boolean;
  created_at: string;
  updated_at: string;
}

export interface ChoreWithUser {
  id: string;
  description: string;
  assigned_to: string;
  assigned_name: string;
  completed: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateChoreInput {
  description: string;
  assigned_to: string;
}

export interface UpdateChoreInput {
  description?: string;
  assigned_to?: string;
  completed?: boolean;
}
