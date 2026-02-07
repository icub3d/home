import type { Calendar } from './calendar';
import type { UserBalance } from './user';
import type { ChoreWithUser } from './chore';

export interface DisplayToken {
  id: string;
  name: string;
  token: string;
  created_at: string;
}

export interface CreateTokenInput {
  name: string;
}

export interface DisplayData {
  weather: Record<string, unknown> | null;
  calendars: Calendar[];
  allowances: UserBalance[];
  chores: ChoreWithUser[];
  background_url: string | null;
}
