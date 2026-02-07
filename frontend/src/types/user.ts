export type UserRole = 'admin' | 'member' | 'child';

export interface User {
  id: string;
  username: string;
  name: string;
  birthday: string;
  profile_picture_url?: string;
  role: UserRole;
  track_allowance: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateUserInput {
  username: string;
  name: string;
  password?: string;
  birthday: string;
  role: UserRole;
  track_allowance?: boolean;
}

export interface UpdateUserInput {
  name?: string;
  birthday?: string;
  role?: UserRole;
  profile_picture_url?: string;
  track_allowance?: boolean;
}

export interface AllowanceTransaction {
  id: string;
  user_id: string;
  amount: number;
  balance: number;
  description: string;
  created_at: string;
}

export interface UserBalance {
  user_id: string;
  name: string;
  balance: number;
}
