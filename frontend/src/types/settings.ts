export interface AppSettings {
  family_name: string;
  base_url: string;
  weather_zip_code: string;
  background_url: string;
  openweather_api_key: string;
  google_client_id: string;
  google_client_secret: string;
  google_connected: boolean;
  google_photos_picked_items?: string;
}

export interface UpdateAppSettingsInput {
  family_name?: string;
  base_url?: string;
  weather_zip_code?: string;
  background_url?: string;
  openweather_api_key?: string;
  google_client_id?: string;
  google_client_secret?: string;
}
