import type { AxiosInstance } from 'axios';
import type { ChoreWithUser, CreateChoreInput, UpdateChoreInput, Chore } from '../types';

export const createChoreApi = (client: AxiosInstance) => ({
  getChores: async (): Promise<ChoreWithUser[]> => {
    const response = await client.get<ChoreWithUser[]>('/chores');
    return response.data;
  },

  createChore: async (input: CreateChoreInput): Promise<Chore> => {
    const response = await client.post<Chore>('/chores', input);
    return response.data;
  },

  updateChore: async (id: string, input: UpdateChoreInput): Promise<Chore> => {
    const response = await client.put<Chore>(`/chores/${id}`, input);
    return response.data;
  },

  deleteChore: async (id: string): Promise<void> => {
    await client.delete(`/chores/${id}`);
  },

  toggleComplete: async (id: string): Promise<Chore> => {
    const response = await client.put<Chore>(`/chores/${id}/toggle`);
    return response.data;
  },
});
