import type { AxiosInstance } from 'axios';
import type { DisplayToken, CreateTokenInput, DisplayData } from '../types';

export const createDisplayApi = (client: AxiosInstance) => ({
  // Admin methods
  getTokens: async (): Promise<DisplayToken[]> => {
    const response = await client.get<DisplayToken[]>('/display/tokens');
    return response.data;
  },

  createToken: async (input: CreateTokenInput): Promise<DisplayToken> => {
    const response = await client.post<DisplayToken>('/display/tokens', input);
    return response.data;
  },

  deleteToken: async (id: string): Promise<void> => {
    await client.delete(`/display/tokens/${id}`);
  },

  // Public/Token authenticated method
  getDisplayData: async (token: string): Promise<DisplayData> => {
    const response = await client.get<DisplayData>('/display/data', {
        headers: {
            'X-Display-Token': token
        }
    });
    return response.data;
  }
});
