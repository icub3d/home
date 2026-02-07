export interface Calendar {
  id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface CreateCalendarInput {
  name: string;
  url?: string;
  google_id?: string;
  color?: string;
}

export interface GoogleCalendarEntry {
  id: string;
  summary: string;
  backgroundColor?: string;
  primary?: boolean;
}