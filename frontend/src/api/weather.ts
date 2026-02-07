import type { AxiosInstance } from 'axios';

export const createWeatherApi = (client: AxiosInstance) => ({
  getWeather: async (): Promise<Record<string, unknown> | null> => {
    const response = await client.get<Record<string, unknown> | null>('/weather');
    return response.data;
  },
});
