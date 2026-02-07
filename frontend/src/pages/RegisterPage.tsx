import { useState } from 'react';
import { useNavigate, Link as RouterLink } from 'react-router-dom';
import { 
  Box, 
  Button, 
  Container, 
  TextField, 
  Typography, 
  Paper, 
  Alert,
  Link,
  Divider,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  IconButton,
  InputAdornment,
} from '@mui/material';
import { 
  Info as InfoIcon,
  OpenInNew as OpenInNewIcon 
} from '@mui/icons-material';
import { authApi } from '../api';
import { useAuth } from '../context/AuthContext';

export default function RegisterPage() {
  const [familyName, setFamilyName] = useState('');
  const [username, setUsername] = useState('');
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  
  // Config state
  const [baseUrl, setBaseUrl] = useState(window.location.origin);
  const [weatherKey, setWeatherKey] = useState('');
  const [googleId, setGoogleId] = useState('');
  const [googleSecret, setGoogleSecret] = useState('');

  // Info Dialog State
  const [weatherInfoOpen, setWeatherKeyInfoOpen] = useState(false);
  const [googleInfoOpen, setGoogleInfoOpen] = useState(false);

  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();
  const { login } = useAuth();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!familyName.trim()) {
      setError('Family Name is required');
      return;
    }

    if (!baseUrl.trim()) {
      setError('Server Base URL is required');
      return;
    }

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 12) {
      setError('Password must be at least 12 characters');
      return;
    }

    setIsLoading(true);
    try {
      const response = await authApi.register({
        family_name: familyName,
        username,
        name,
        password,
        openweather_api_key: weatherKey,
        google_client_id: googleId,
        google_client_secret: googleSecret,
        base_url: baseUrl,
      });
      login(response.user.id, response.user.username, response.user.role, response.token);
      navigate('/dashboard');
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error.response?.data?.error || 'Setup failed. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Container component="main" maxWidth="sm">
      <Box
        sx={{
          marginTop: 4,
          marginBottom: 4,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
        }}
      >
        <Paper sx={{ p: 4, width: '100%' }}>
          <Typography component="h1" variant="h4" align="center" mb={1}>
            {familyName ? `${familyName} Home Setup` : 'System Setup'}
          </Typography>
          <Typography variant="body1" color="text.secondary" align="center" mb={3}>
            Initialize your family's home management system
          </Typography>
          
          {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}
          
          <Box component="form" onSubmit={handleSubmit} noValidate>
            <Typography variant="h6" sx={{ mt: 2, mb: 1 }}>
              1. System Information
            </Typography>
            <TextField
              margin="normal"
              required
              fullWidth
              id="baseUrl"
              label="Server URL"
              name="baseUrl"
              placeholder="https://home.example.com"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              helperText="The external URL used to access this system (e.g. http://localhost:4000). Essential for OAuth redirects."
            />
            <TextField
              margin="normal"
              required
              fullWidth
              id="familyName"
              label="Family Name"
              name="familyName"
              placeholder="e.g. Smith"
              autoFocus
              value={familyName}
              onChange={(e) => setFamilyName(e.target.value)}
              helperText="This will be displayed as the title of your home dashboard"
            />

            <Divider sx={{ my: 4 }} />

            <Typography variant="h6" sx={{ mb: 1 }}>
              2. Administrator Account
            </Typography>
            <TextField
              margin="normal"
              required
              fullWidth
              id="name"
              label="Your Name"
              name="name"
              placeholder="John Smith"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            <TextField
              margin="normal"
              required
              fullWidth
              id="username"
              label="Username"
              name="username"
              placeholder="john"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
            />
            <TextField
              margin="normal"
              required
              fullWidth
              name="password"
              label="Password"
              type="password"
              id="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              helperText="At least 12 characters"
            />
            <TextField
              margin="normal"
              required
              fullWidth
              name="confirmPassword"
              label="Confirm Password"
              type="password"
              id="confirmPassword"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
            />

            <Divider sx={{ my: 4 }} />

            <Typography variant="h6" sx={{ mb: 1 }}>
              3. Optional Configuration
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={2}>
              These can also be configured later in the settings page.
            </Typography>

            <TextField
              margin="normal"
              fullWidth
              id="weatherKey"
              label="OpenWeather API Key"
              name="weatherKey"
              value={weatherKey}
              onChange={(e) => setWeatherKey(e.target.value)}
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
              margin="normal"
              fullWidth
              id="googleId"
              label="Google Client ID"
              name="googleId"
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
              margin="normal"
              fullWidth
              id="googleSecret"
              label="Google Client Secret"
              name="googleSecret"
              type="password"
              value={googleSecret}
              onChange={(e) => setGoogleSecret(e.target.value)}
            />

            <Button
              type="submit"
              fullWidth
              variant="contained"
              size="large"
              sx={{ mt: 4, mb: 2 }}
              disabled={isLoading}
            >
              {isLoading ? 'Setting up...' : 'Complete Setup'}
            </Button>
            
            <Box textAlign="center">
              <Typography variant="body2">
                Already setup?{' '}
                <Link component={RouterLink} to="/login">
                  Sign in
                </Link>
              </Typography>
            </Box>
          </Box>
        </Paper>
      </Box>

      {/* OpenWeather Info Dialog */}
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

      {/* Google Cloud Info Dialog */}
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
    </Container>
  );
}