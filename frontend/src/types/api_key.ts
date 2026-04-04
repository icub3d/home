export interface ApiKey {
  id: string;
  name: string;
  user_id: string;
  last_used_at?: string;
  created_at: string;
}

export interface CreatedApiKeyResponse {
  id: string;
  key: string;
  name: string;
}

export interface CreateApiKeyInput {
  name: string;
}
