import type { AxiosInstance } from 'axios';
import type { User } from '../types';

export interface LoginResponse {
  token: string;
  user: User;
}

export interface RegisterInput {
  username: string;
  name: string;
  password: string;
  family_name: string;
  base_url: string;
  openweather_api_key?: string;
  google_client_id?: string;
  google_client_secret?: string;
}

export interface SystemStatus {
  initialized: boolean;
}

export const createAuthApi = (client: AxiosInstance) => ({
  getStatus: async (): Promise<SystemStatus> => {
    const response = await client.get<SystemStatus>('/auth/status');
    return response.data;
  },
  login: async (username: string, password: string): Promise<LoginResponse> => {
    const response = await client.post<LoginResponse>('/auth/login', { username, password });
    return response.data;
  },
  register: async (input: RegisterInput): Promise<LoginResponse> => {
    const response = await client.post<LoginResponse>('/auth/register', input);
    return response.data;
  },
});
