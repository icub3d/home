import type { AxiosInstance } from 'axios';
import type { ApiKey, CreateApiKeyInput, CreatedApiKeyResponse } from '../types/api_key';

export const createApiKeysApi = (client: AxiosInstance) => ({
  listKeys: async (): Promise<ApiKey[]> => {
    const response = await client.get<ApiKey[]>('/api-keys');
    return response.data;
  },

  createKey: async (input: CreateApiKeyInput): Promise<CreatedApiKeyResponse> => {
    const response = await client.post<CreatedApiKeyResponse>('/api-keys', input);
    return response.data;
  },

  deleteKey: async (id: string): Promise<void> => {
    await client.delete(`/api-keys/${id}`);
  },
});
