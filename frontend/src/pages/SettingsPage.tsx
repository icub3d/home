import React, { useState, useEffect } from 'react';
import {
  Typography,
  Paper,
  Box,
  TextField,
  Button,
  CircularProgress,
  Alert,
  List,
  ListItem,
  ListItemText,
  IconButton,
  Divider,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Stack,
  Link,
  ToggleButton,
  ToggleButtonGroup,
  FormHelperText,
  InputAdornment,
} from '@mui/material';
import { 
  Delete as DeleteIcon, 
  Add as AddIcon, 
  CloudDownload as DownloadIcon, 
  CloudUpload as UploadIcon, 
  PhotoLibrary as PhotoIcon, 
  Link as LinkIcon, 
  CalendarMonth as CalendarIcon,
  Info as InfoIcon,
  OpenInNew as OpenInNewIcon,
  Settings as SettingsIcon,
} from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { settingsApi, calendarApi, displayApi, googlePhotosApi } from '../api';
import type { CreateCalendarInput } from '../types';
import { client } from '../api/client';

const calendarColors = ['primary', 'secondary', 'error', 'warning', 'info', 'success'];

export default function SettingsPage() {
  const queryClient = useQueryClient();
  
  // System Config State
  const [familyName, setFamilyName] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const [weatherKey, setWeatherKey] = useState('');
  const [googleId, setGoogleId] = useState('');
  const [googleSecret, setGoogleSecret] = useState('');
  const [zipCode, setZipCode] = useState('');

  // Info Dialog State
  const [weatherInfoOpen, setWeatherKeyInfoOpen] = useState(false);
  const [googleInfoOpen, setGoogleInfoOpen] = useState(false);

  // Display Token State
  const [isTokenDialogOpen, setIsTokenDialogOpen] = useState(false);
  const [newTokenName, setNewTokenName] = useState('');

  // Google Photos State
  const [isPickerDialogOpen, setIsPickerDialogOpen] = useState(false);
  const [pickerSessionId, setPickerSessionId] = useState('');
  const [pickerUri, setPickerUri] = useState('');

  const [successMessage, setSuccessMessage] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  // Fetch Settings
  const { data: settings, isLoading: isSettingsLoading } = useQuery({
    queryKey: ['settings'],
    queryFn: () => settingsApi.getSettings(),
  });

  // Fetch Calendars
  const { data: calendars, isLoading: isCalLoading } = useQuery({
    queryKey: ['calendars'],
    queryFn: calendarApi.getCalendars,
  });

  // Fetch Display Tokens
  const { data: tokens, isLoading: isTokenLoading } = useQuery({
    queryKey: ['displayTokens'],
    queryFn: displayApi.getTokens,
  });

  useEffect(() => {
    if (settings) {
      setFamilyName(settings.family_name || '');
      setBaseUrl(settings.base_url || '');
      setZipCode(settings.weather_zip_code || '');
      setWeatherKey(settings.openweather_api_key || '');
      setGoogleId(settings.google_client_id || '');
      setGoogleSecret(settings.google_client_secret || '');
    }
  }, [settings]);
  
  // Calendar State
  const [isCalDialogOpen, setIsCalDialogOpen] = useState(false);
  const [calendarType, setCalendarType] = useState<'ical' | 'google'>('ical');
  const [newCalName, setNewCalName] = useState('');
  const [newCalUrl, setNewCalUrl] = useState('');
  const [newCalGoogleId, setNewCalGoogleId] = useState('');
  const [newCalColor, setNewCalColor] = useState('primary');

  // Google Calendars Query
  const { data: googleCalendars, isError: isGoogleCalError } = useQuery({
    queryKey: ['googleCalendars'],
    queryFn: calendarApi.listGoogleCalendars,
    enabled: isCalDialogOpen && calendarType === 'google',
    retry: false,
  });

  // Mutations
  const createTokenMutation = useMutation({
    mutationFn: (name: string) => displayApi.createToken({ name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['displayTokens'] });
      setIsTokenDialogOpen(false);
      setNewTokenName('');
      setSuccessMessage('Display token created.');
    },
    onError: () => setErrorMessage('Failed to create token.')
  });

  const deleteTokenMutation = useMutation({
    mutationFn: (id: string) => displayApi.deleteToken(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['displayTokens'] });
      setSuccessMessage('Display token deleted.');
    }
  });

  const updateSettingsMutation = useMutation({
    mutationFn: async () => {
      await settingsApi.updateSettings({ 
        family_name: familyName,
        base_url: baseUrl,
        weather_zip_code: zipCode,
        openweather_api_key: weatherKey,
        google_client_id: googleId,
        google_client_secret: googleSecret,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      setSuccessMessage('Settings updated.');
      setErrorMessage('');
    },
    onError: (err: any) => {
      setErrorMessage(err.response?.data?.error || 'Failed to update settings.');
      setSuccessMessage('');
    }
  });

  const createCalendarMutation = useMutation({
    mutationFn: (input: CreateCalendarInput) => calendarApi.createCalendar(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendars'] });
      setIsCalDialogOpen(false);
      setNewCalName('');
      setNewCalUrl('');
      setNewCalGoogleId('');
      setNewCalColor('primary');
      setSuccessMessage('Calendar added.');
    },
    onError: () => setErrorMessage('Failed to add calendar.')
  });

  const deleteCalendarMutation = useMutation({
    mutationFn: (id: string) => calendarApi.deleteCalendar(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['calendars'] });
      setSuccessMessage('Calendar removed.');
    }
  });

  const handleAddToken = (e: React.FormEvent) => {
    e.preventDefault();
    createTokenMutation.mutate(newTokenName);
  };

  const handleDeleteToken = (id: string) => {
    if (window.confirm('Are you sure? This will disconnect any display using this token.')) {
      deleteTokenMutation.mutate(id);
    }
  };

  const handleConfigSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    updateSettingsMutation.mutate();
  };

  const handleAddCalendar = (e: React.FormEvent) => {
    e.preventDefault();
    createCalendarMutation.mutate({
      name: newCalName,
      url: calendarType === 'ical' ? newCalUrl : undefined,
      google_id: calendarType === 'google' ? newCalGoogleId : undefined,
      color: newCalColor
    });
  };

  const handleDeleteCalendar = (id: string) => {
    if (window.confirm('Are you sure you want to delete this calendar?')) {
      deleteCalendarMutation.mutate(id);
    }
  };

  const handleExportBackup = async () => {
    try {
      const response = await client.get('/backup/export', { responseType: 'blob' });
      const url = window.URL.createObjectURL(new Blob([response.data]));
      const link = document.createElement('a');
      link.href = url;
      link.setAttribute('download', `home_backup_${new Date().toISOString().split('T')[0]}.json`);
      document.body.appendChild(link);
      link.click();
      link.remove();
    } catch (_error: unknown) {
      setErrorMessage('Failed to export backup.');
    }
  };

  const handleImportBackup = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const json = JSON.parse(e.target?.result as string);
        await client.post('/backup/import', json);
        setSuccessMessage('Backup imported successfully. Reloading...');
        setTimeout(() => window.location.reload(), 1500);
      } catch (_error) {
        setErrorMessage('Failed to import backup. Invalid file or server error.');
      }
    };
    reader.readAsText(file);
  };

  const handleConnectGooglePhotos = async () => {
    try {
      const response = await googlePhotosApi.startOAuth();
      window.location.href = response.auth_url;
    }
    catch (_error) {
      setErrorMessage('Failed to start Google Photos OAuth flow.');
    }
  };

  const handleStartPicking = async () => {
    try {
      setErrorMessage('');
      const session = await googlePhotosApi.createSession();
      setPickerSessionId(session.id);
      setPickerUri(session.pickerUri);
      setIsPickerDialogOpen(true);
      window.open(session.pickerUri, '_blank');
    }
    catch (error: unknown) {
      const err = error as { response?: { data?: { error?: string } }; message?: string };
      setErrorMessage(err.response?.data?.error || err.message || 'Failed to start photo picker.');
    }
  };

  const handleFinishPicking = async () => {
    try {
      await googlePhotosApi.confirmSelection(pickerSessionId);
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      setIsPickerDialogOpen(false);
      setSuccessMessage('Photos selected successfully!');
      setPickerSessionId('');
      setPickerUri('');
    }
    catch (error: unknown) {
      const err = error as { response?: { data?: { error?: string } }; message?: string };
      setErrorMessage(err.response?.data?.error || err.message || 'Failed to confirm selection.');
    }
  };

  const handleDisconnectGooglePhotos = async () => {
    if (window.confirm('Disconnect Google Photos? This will clear all tokens and settings.')) {
      try {
        await googlePhotosApi.disconnect();
        queryClient.invalidateQueries({ queryKey: ['settings'] });
        setSuccessMessage('Google Photos disconnected.');
      } catch (_error) {
        setErrorMessage('Failed to disconnect Google Photos.');
      }
    }
  };

  const isLoading = isSettingsLoading || isCalLoading || isTokenLoading;

  if (isLoading) return <Box display="flex" justifyContent="center" mt={4}><CircularProgress /></Box>;

  return (
    <Box maxWidth="md">
      {successMessage && <Alert severity="success" sx={{ mb: 2 }}>{successMessage}</Alert>}
      {errorMessage && <Alert severity="error" sx={{ mb: 2 }}>{errorMessage}</Alert>}

      {/* App Info Section */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Box display="flex" alignItems="center" gap={2} mb={2}>
          <Box
            component="img"
            src="/home-icon.svg"
            alt="Home"
            sx={{ width: 48, height: 48 }}
          />
          <Box>
            <Typography variant="h5" fontWeight="bold">{settings?.family_name || 'Home'}</Typography>
            <Typography variant="body2" color="text.secondary">
              Family Management System
            </Typography>
          </Box>
        </Box>
      </Paper>

      {/* System Configuration */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Box display="flex" alignItems="center" gap={1} mb={2}>
          <SettingsIcon />
          <Typography variant="h6">System Configuration</Typography>
        </Box>
        <form onSubmit={handleConfigSubmit}>
          <Stack spacing={3}>
            <TextField
              label="Family Name"
              fullWidth
              value={familyName}
              onChange={(e) => setFamilyName(e.target.value)}
              helperText="Displayed as the dashboard title"
            />
            <TextField
              label="Server URL"
              fullWidth
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              helperText="The external URL used to access this system. Critical for OAuth."
            />
            
            <Divider />
            
            <Typography variant="subtitle2" color="text.secondary">API Credentials</Typography>
            
            <TextField
              label="OpenWeather API Key"
              fullWidth
              value={weatherKey}
              onChange={(e) => setWeatherKey(e.target.value)}
              type="password"
              slotProps={{
                input: {
                  endAdornment: (
                    <InputAdornment position="end">
                      <IconButton onClick={() => setWeatherKeyInfoOpen(true)} edge="end">
                        <InfoIcon />
                      </IconButton>
                    </InputAdornment>
                  ),
                }
              }}
            />

            <TextField
              label="Google Client ID"
              fullWidth
              value={googleId}
              onChange={(e) => setGoogleId(e.target.value)}
              slotProps={{
                input: {
                  endAdornment: (
                    <InputAdornment position="end">
                      <IconButton onClick={() => setGoogleInfoOpen(true)} edge="end">
                        <InfoIcon />
                      </IconButton>
                    </InputAdornment>
                  ),
                }
              }}
            />

            <TextField
              label="Google Client Secret"
              fullWidth
              value={googleSecret}
              onChange={(e) => setGoogleSecret(e.target.value)}
              type="password"
            />

            <Box display="flex" justifyContent="flex-end">
              <Button 
                type="submit" 
                variant="contained" 
                disabled={updateSettingsMutation.isPending}
              >
                {updateSettingsMutation.isPending ? 'Saving...' : 'Save Configuration'}
              </Button>
            </Box>
          </Stack>
        </form>
      </Paper>

      {/* Backup & Restore */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>Backup & Restore</Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Export your data to a JSON file or restore from a previous backup. 
          <strong> Warning: Importing will overwrite all existing data.</strong>
        </Typography>
        <Stack direction="row" spacing={2}>
          <Button variant="outlined" startIcon={<DownloadIcon />} onClick={handleExportBackup}>
            Export Backup
          </Button>
          <Button variant="outlined" component="label" startIcon={<UploadIcon />}>
            Import Backup
            <input type="file" hidden accept=".json" onChange={handleImportBackup} />
          </Button>
        </Stack>
      </Paper>

      {/* Weather Configuration */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>Weather Location</Typography>
        <form onSubmit={handleConfigSubmit}>
          <Box display="flex" flexDirection="column" gap={2}>
            <TextField
              label="Zip Code"
              value={zipCode}
              onChange={(e) => setZipCode(e.target.value)}
              helperText="Enter your 5-digit zip code (US) for weather forecasts."
            />
            <Button 
              type="submit" 
              variant="contained" 
              disabled={updateSettingsMutation.isPending}
            >
              {updateSettingsMutation.isPending ? 'Saving...' : 'Save Location'}
            </Button>
          </Box>
        </form>
      </Paper>

      {/* Google Photos Integration */}
      <Paper sx={{ p: 3, mb: 3}}>
        <Typography variant="h6" gutterBottom>Google Account (Photos & Calendar)</Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Connect your Google Photos account and select photos for the display background.
        </Typography>
        
        {settings?.google_connected ? (
          <Box>
            <Alert severity="success" sx={{ mb: 2 }}>
              ✓ Connected to Google Account
            </Alert>
            <Stack direction="row" spacing={2}>
              <Button 
                variant="contained" 
                startIcon={<PhotoIcon />} 
                onClick={handleStartPicking}
              >
                Pick Photos
              </Button>
              <Button variant="outlined" color="error" onClick={handleDisconnectGooglePhotos}>
                Disconnect
              </Button>
            </Stack>
          </Box>
        ) : (
          <Stack direction="row" spacing={2}>
            <Button 
              variant="contained" 
              startIcon={<PhotoIcon />} 
              onClick={handleConnectGooglePhotos}
            >
              Connect Google Account
            </Button>
          </Stack>
        )}
      </Paper>

      {/* Calendar Configuration */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Typography variant="h6">Calendars</Typography>
          <Button startIcon={<AddIcon />} variant="outlined" onClick={() => setIsCalDialogOpen(true)}>
            Add Calendar
          </Button>
        </Box>
        
        <List>
          {calendars?.map((cal) => (
            <React.Fragment key={cal.id}>
              <ListItem
                secondaryAction={
                  <IconButton edge="end" aria-label="delete" onClick={() => handleDeleteCalendar(cal.id)}>
                    <DeleteIcon />
                  </IconButton>
                }
              >
                <ListItemText primary={cal.name} />
              </ListItem>
              <Divider />
            </React.Fragment>
          ))}
          {calendars?.length === 0 && (
            <Typography variant="body2" color="text.secondary">No calendars added.</Typography>
          )}
        </List>
      </Paper>

      {/* Display Tokens */}
      <Paper sx={{ p: 3 }}>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Typography variant="h6">Display Tokens</Typography>
          <Button startIcon={<AddIcon />} variant="outlined" onClick={() => setIsTokenDialogOpen(true)}>
            Create Token
          </Button>
        </Box>
        <List>
          {tokens?.map((token) => (
            <React.Fragment key={token.id}>
              <ListItem
                secondaryAction={
                  <IconButton edge="end" aria-label="delete" onClick={() => handleDeleteToken(token.id)}>
                    <DeleteIcon />
                  </IconButton>
                }
              >
                <ListItemText 
                  primary={token.name} 
                  secondary={token.token}
                  secondaryTypographyProps={{ fontFamily: 'monospace' }}
                />
              </ListItem>
              <Divider />
            </React.Fragment>
          ))}
          {tokens?.length === 0 && (
            <Typography variant="body2" color="text.secondary">No active display tokens.</Typography>
          )}
        </List>
      </Paper>

      {/* Add Token Dialog */}
      <Dialog open={isTokenDialogOpen} onClose={() => setIsTokenDialogOpen(false)} fullWidth maxWidth="sm">
        <form onSubmit={handleAddToken}>
          <DialogTitle>Create Display Token</DialogTitle>
          <DialogContent>
            <Box display="flex" flexDirection="column" gap={2} mt={1}>
              <TextField
                label="Name (e.g. Living Room)"
                required
                fullWidth
                value={newTokenName}
                onChange={(e) => setNewTokenName(e.target.value)}
              />
            </Box>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setIsTokenDialogOpen(false)}>Cancel</Button>
            <Button type="submit" variant="contained">Create</Button>
          </DialogActions>
        </form>
      </Dialog>

      {/* Add Calendar Dialog */}
      <Dialog open={isCalDialogOpen} onClose={() => setIsCalDialogOpen(false)} fullWidth maxWidth="sm">
        <form onSubmit={handleAddCalendar}>
          <DialogTitle>Add Calendar</DialogTitle>
          <DialogContent>
            <Box display="flex" flexDirection="column" gap={2} mt={1}>
              
              <ToggleButtonGroup
                value={calendarType}
                exclusive
                onChange={(_, next) => next && setCalendarType(next)}
                fullWidth
                color="primary"
              >
                <ToggleButton value="ical">
                  <LinkIcon sx={{ mr: 1 }} /> iCal URL
                </ToggleButton>
                <ToggleButton value="google">
                  <CalendarIcon sx={{ mr: 1 }} /> Google Calendar
                </ToggleButton>
              </ToggleButtonGroup>

              <TextField
                label="Name"
                required
                fullWidth
                value={newCalName}
                onChange={(e) => setNewCalName(e.target.value)}
              />

              {calendarType === 'ical' ? (
                <TextField
                  label="iCal URL"
                  required
                  fullWidth
                  value={newCalUrl}
                  onChange={(e) => setNewCalUrl(e.target.value)}
                  helperText="Paste the 'Secret address in iCal format'."
                />
              ) : (
                <FormControl fullWidth>
                  <InputLabel>Select Calendar</InputLabel>
                  <Select
                    value={newCalGoogleId}
                    label="Select Calendar"
                    onChange={(e) => {
                        setNewCalGoogleId(e.target.value);
                        const cal = googleCalendars?.find(c => c.id === e.target.value);
                        if (cal && !newCalName) setNewCalName(cal.summary);
                    }}
                  >
                    {googleCalendars?.map((cal) => (
                      <MenuItem key={cal.id} value={cal.id}>
                        {cal.summary} {cal.primary && "(Primary)"}
                      </MenuItem>
                    ))}
                  </Select>
                  {isGoogleCalError && (
                    <FormHelperText error>
                        Failed to list calendars. Make sure you are connected with Calendar permissions (try reconnecting in Google Photos section).
                    </FormHelperText>
                  )}
                </FormControl>
              )}

              <FormControl fullWidth>
                <InputLabel>Color</InputLabel>
                <Select
                  value={newCalColor}
                  label="Color"
                  onChange={(e) => setNewCalColor(e.target.value)}
                >
                  {calendarColors.map((color) => (
                    <MenuItem key={color} value={color} sx={{ textTransform: 'capitalize' }}>
                      {color}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Box>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setIsCalDialogOpen(false)}>Cancel</Button>
            <Button type="submit" variant="contained">Add</Button>
          </DialogActions>
        </form>
      </Dialog>

      {/* Photo Picker Dialog */}
      <Dialog open={isPickerDialogOpen} onClose={() => setIsPickerDialogOpen(false)} fullWidth maxWidth="sm">
        <DialogTitle>Select Photos</DialogTitle>
        <DialogContent>
          <Typography paragraph>
            A Google Photos Picker window has been opened.
          </Typography>
          <Typography paragraph>
            1. Select the photos you want to display in that window.
            <br />
            2. Click "Done" in the Google window.
            <br />
            3. Come back here and click "Confirm Selection" below.
          </Typography>
          {pickerUri && (
             <Box mt={2}>
                <Typography variant="caption" display="block">
                    If the window didn't open, click here:
                </Typography>
                <Link href={pickerUri} target="_blank" rel="noopener">
                    Open Photo Picker
                </Link>
             </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsPickerDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleFinishPicking} variant="contained" autoFocus>
            Confirm Selection
          </Button>
        </DialogActions>
      </Dialog>

      {/* Info Dialogs (copied from Setup page) */}
      <Dialog open={weatherInfoOpen} onClose={() => setWeatherKeyInfoOpen(false)} maxWidth="xs">
        <DialogTitle>Getting an OpenWeather API Key</DialogTitle>
        <DialogContent>
          <Typography variant="body2" paragraph>
            To display weather information, you need a free API key from OpenWeatherMap.
          </Typography>
          <Box component="ol" sx={{ pl: 2, typography: 'body2' }}>
            <li>Go to <Link href="https://openweathermap.org/api" target="_blank" rel="noopener">OpenWeatherMap API <OpenInNewIcon sx={{ fontSize: 14, verticalAlign: 'middle' }} /></Link></li>
            <li>Sign up for a free account.</li>
            <li>Navigate to the <strong>API keys</strong> tab in your profile.</li>
            <li>Generate a new key or use the default one.</li>
          </Box>
          <Typography variant="caption" color="warning.main">
            Note: It may take up to 2 hours for a new API key to become active.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setWeatherKeyInfoOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      <Dialog open={googleInfoOpen} onClose={() => setGoogleInfoOpen(false)} maxWidth="sm">
        <DialogTitle>Setting up Google Cloud Credentials</DialogTitle>
        <DialogContent>
          <Typography variant="body2" paragraph>
            Google credentials are required to sync your calendars and display Google Photos.
          </Typography>
          <Box component="ol" sx={{ pl: 2, typography: 'body2' }}>
            <li>Go to the <Link href="https://console.cloud.google.com/" target="_blank" rel="noopener">Google Cloud Console <OpenInNewIcon sx={{ fontSize: 14, verticalAlign: 'middle' }} /></Link></li>
            <li>Create a new project.</li>
            <li>Enable the <strong>Photos Picker API</strong> and <strong>Google Calendar API</strong>.</li>
            <li>Go to <strong>APIs & Services &gt; Credentials</strong>.</li>
            <li>Click <strong>Create Credentials &gt; OAuth client ID</strong>.</li>
            <li>Select <strong>Web application</strong> as the type.</li>
            <li>Add <strong>Authorized redirect URIs</strong>:
              <Box component="pre" sx={{ bgcolor: 'action.hover', p: 1, my: 1, borderRadius: 1, overflowX: 'auto' }}>
                {(baseUrl || window.location.origin).replace(/\/+$/, '')}/api/google-photos/callback
              </Box>
            </li>
            <li>Copy the <strong>Client ID</strong> and <strong>Client Secret</strong> into the fields on this page.</li>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setGoogleInfoOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
