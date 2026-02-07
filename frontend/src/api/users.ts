import type { AxiosInstance } from 'axios';
import type { User, CreateUserInput, UpdateUserInput } from '../types';

export const createUsersApi = (client: AxiosInstance) => ({
  getUsers: async (): Promise<User[]> => {
    const response = await client.get<User[]>('/users');
    return response.data;
  },

  createUser: async (input: CreateUserInput): Promise<User> => {
    const response = await client.post<User>('/users', input);
    return response.data;
  },

  updateUser: async (id: string, input: UpdateUserInput): Promise<User> => {
    const response = await client.put<User>(`/users/${id}`, input);
    return response.data;
  },

  deleteUser: async (id: string): Promise<void> => {
    await client.delete(`/users/${id}`);
  },

  changePassword: async (id: string, password: string): Promise<void> => {
    await client.put(`/users/${id}/password`, { password });
  }
});
