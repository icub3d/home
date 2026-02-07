import type { AxiosInstance } from 'axios';
import type { AppSettings, UpdateAppSettingsInput } from '../types';

export const createSettingsApi = (client: AxiosInstance) => ({
  getSettings: async (): Promise<AppSettings> => {
    const response = await client.get<AppSettings>('/settings');
    return response.data;
  },

  updateSettings: async (input: UpdateAppSettingsInput): Promise<AppSettings> => {
    const response = await client.put<AppSettings>('/settings', input);
    return response.data;
  },
});
