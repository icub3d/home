import type { AxiosInstance } from 'axios';

export interface OAuthStartResponse {
  auth_url: string;
}

export interface PickerSession {
  id: string;
  pickerUri: string;
}

export const createGooglePhotosApi = (client: AxiosInstance) => ({
  startOAuth: async (): Promise<OAuthStartResponse> => {
    const response = await client.post<OAuthStartResponse>('/google-photos/start');
    return response.data;
  },

  createSession: async (): Promise<PickerSession> => {
    const response = await client.post<PickerSession>('/google-photos/session');
    return response.data;
  },

  confirmSelection: async (sessionId: string): Promise<void> => {
    await client.post('/google-photos/confirm', { session_id: sessionId });
  },

  disconnect: async (): Promise<void> => {
    await client.post('/google-photos/disconnect');
  },
});