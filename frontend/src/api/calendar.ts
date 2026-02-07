import type { AxiosInstance } from 'axios';
import type { Calendar, CreateCalendarInput, GoogleCalendarEntry } from '../types';

export const createCalendarApi = (client: AxiosInstance) => ({
  getCalendars: async (): Promise<Calendar[]> => {
    const response = await client.get<Calendar[]>('/calendars');
    return response.data;
  },

  listGoogleCalendars: async (): Promise<GoogleCalendarEntry[]> => {
    const response = await client.get<GoogleCalendarEntry[]>('/calendars/google');
    return response.data;
  },

  createCalendar: async (input: CreateCalendarInput): Promise<Calendar> => {
    const response = await client.post<Calendar>('/calendars', input);
    return response.data;
  },

  deleteCalendar: async (id: string): Promise<void> => {
    await client.delete(`/calendars/${id}`);
  },

  getFeed: async (id: string): Promise<string> => {
    const response = await client.get<string>(`/calendars/${id}/feed`);
    return response.data;
  },
});