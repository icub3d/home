import { client } from './client';
import { createAllowanceApi } from './allowance';
import { createAuthApi } from './auth';
import { createCalendarApi } from './calendar';
import { createChoreApi } from './chores';
import { createDisplayApi } from './display';
import { createSettingsApi } from './settings';
import { createUsersApi } from './users';
import { createWeatherApi } from './weather';
import { createGooglePhotosApi } from './googlePhotos';
import { createApiKeysApi } from './apiKeys';

export const allowanceApi = createAllowanceApi(client);
export const authApi = createAuthApi(client);
export const calendarApi = createCalendarApi(client);
export const choreApi = createChoreApi(client);
export const displayApi = createDisplayApi(client);
export const settingsApi = createSettingsApi(client);
export const usersApi = createUsersApi(client);
export const weatherApi = createWeatherApi(client);
export const googlePhotosApi = createGooglePhotosApi(client);
export const apiKeysApi = createApiKeysApi(client);

export * from './client';
export * from './auth';
export * from './apiKeys';
