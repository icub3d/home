import axios from 'axios';
// Import the factory functions
import { createAllowanceApi } from './allowance';
import { createAuthApi } from './auth';
import { createCalendarApi } from './calendar';
import { createChoreApi } from './chores';
import { createDisplayApi } from './display';
import { createSettingsApi } from './settings';
import { createUsersApi } from './users';
import { createWeatherApi } from './weather';
import { createGooglePhotosApi } from './googlePhotos';

export const API_URL = '/api';

export const client = axios.create({
  baseURL: API_URL,
});

client.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Add response interceptor to handle auth errors and sliding session tokens
client.interceptors.response.use(
  (response) => {
    // Check for a new sliding session token
    const newToken = response.headers['x-new-token'];
    if (newToken) {
      localStorage.setItem('token', newToken);
    }
    return response;
  },
  (error) => {
    // If we get a 401 Unauthorized, clear tokens and redirect to login
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('userId');
      localStorage.removeItem('username');
      localStorage.removeItem('userRole');
      
      // Only redirect if we're not already on the login or display page
      // Display page has its own token authentication
      const pathname = window.location.pathname;
      if (pathname !== '/login' && pathname !== '/display') {
        window.location.href = '/login';
      }
    }
    return Promise.reject(error);
  }
);

// Create and export API instances
export const allowanceApi = createAllowanceApi(client);
export const authApi = createAuthApi(client);
export const calendarApi = createCalendarApi(client);
export const choreApi = createChoreApi(client);
export const displayApi = createDisplayApi(client);
export const settingsApi = createSettingsApi(client);
export const usersApi = createUsersApi(client);
export const weatherApi = createWeatherApi(client);
export const googlePhotosApi = createGooglePhotosApi(client);
