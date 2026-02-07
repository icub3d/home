import { useState, useEffect, useRef } from 'react';
import {
  Typography,
  Paper,
  Box,
  TextField,
  Button,
  CircularProgress,
  List,
  ListItem,
  ListItemText,
  Divider,
  ListItemIcon,
  IconButton,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  type SxProps,
  type Theme
} from '@mui/material';
import {
  Event as EventIcon,
  AssignmentTurnedIn as ChoreIcon,
  Cloud as CloudIcon,
  AttachMoney as MoneyIcon,
  Settings as SettingsIcon,
  QrCode as QrCodeIcon
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { displayApi, API_URL, client } from '../api';
import ICAL from 'ical.js';
import { formatCurrency } from '../utils/currency';
import { QRCodeSVG } from 'qrcode.react';

interface EventDisplay {
  summary: string;
  startDate: Date;
  calendarName: string;
  color: string;
}

// Helper to resolve image URL
const getImageUrl = (url?: string, token?: string) => {
  if (!url) return undefined;
  if (url.startsWith('http')) return url;
  
  try {
    let fullUrl: string;
    if (API_URL.startsWith('http')) {
      const urlObj = new URL(API_URL);
      fullUrl = urlObj.origin + url;
    } else {
      fullUrl = window.location.origin + url;
    }
    
    if (token) {
      fullUrl += (fullUrl.includes('?') ? '&' : '?') + 'token=' + encodeURIComponent(token);
    }
    
    return fullUrl;
  } catch (e) {
    console.error('Failed to resolve image URL', e);
    return url;
  }
};

// --- Extracted Components ---

const WeatherCard = ({ data, sx }: { data: any, sx: SxProps<Theme> }) => (
  <Paper elevation={0} sx={[...(Array.isArray(sx) ? sx : [sx]), { border: 4, borderColor: 'info.main' }]}>
    <Typography variant="h2" gutterBottom color="info.main" fontWeight="bold" sx={{ fontSize: '4.2rem', textShadow: '0 2px 4px rgba(0,0,0,0.1)', mb: 4, display: 'flex', alignItems: 'center', gap: 2 }}>
      <CloudIcon fontSize="inherit" />
      Weather
    </Typography>
    <Box flex={1} display="flex" flexDirection="column" justifyContent="center" alignItems="center">
      <img
        src={`https://openweathermap.org/img/wn/${data.weather[0].icon}@4x.png`}
        alt={data.weather[0].description}
        style={{ width: 180, height: 180 }}
      />
      <Typography fontWeight="500" sx={{ fontSize: '7rem', lineHeight: 1, textShadow: '0 4px 8px rgba(0,0,0,0.15)' }}>
        {Math.round(data.main.temp)}°
      </Typography>
      <Typography variant="h4" sx={{ fontSize: '2.4rem', textTransform: 'capitalize', color: 'text.secondary' }}>
        {data.weather[0].description}
      </Typography>
      <Typography variant="h6" color="text.secondary" mt={2} sx={{ fontSize: '1.5rem' }}>
        {data.name}
      </Typography>
      <Box display="flex" gap={5} mt={4}>
        <Box textAlign="center">
          <Typography variant="body1" color="text.secondary" sx={{ fontSize: '1.1rem' }}>Humidity</Typography>
          <Typography variant="h6" sx={{ fontSize: '1.5rem' }}>{data.main.humidity}%</Typography>
        </Box>
        <Box textAlign="center">
          <Typography variant="body1" color="text.secondary" sx={{ fontSize: '1.1rem' }}>Wind</Typography>
          <Typography variant="h6" sx={{ fontSize: '1.5rem' }}>{Math.round(data.wind.speed)} mph</Typography>
        </Box>
      </Box>
    </Box>
  </Paper>
);

const AllowanceCard = ({ data, sx }: { data: any[], sx: SxProps<Theme> }) => (
  <Paper elevation={0} sx={[...(Array.isArray(sx) ? sx : [sx]), { border: 4, borderColor: 'success.main' }]}>
    <Typography variant="h2" gutterBottom color="success.main" fontWeight="bold" sx={{ fontSize: '4.2rem', textShadow: '0 2px 4px rgba(0,0,0,0.1)', mb: 4, display: 'flex', alignItems: 'center', gap: 2 }}>
      <MoneyIcon fontSize="inherit" />
      Allowance
    </Typography>
    <List sx={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
      {data.map((user, index) => (
        <Box key={user.user_id}>
          <ListItem sx={{ py: 3 }}>
            <ListItemText
              primary={user.name}
              primaryTypographyProps={{ variant: 'h4', fontWeight: 'medium', sx: { fontSize: '2.4rem', textShadow: '0 1px 2px rgba(0,0,0,0.1)' } }}
            />
            <Box
              sx={{
                bgcolor: user.balance < 0 ? 'error.main' : 'success.main',
                color: 'white',
                px: 3, py: 1, borderRadius: 4,
                fontWeight: 'bold',
                fontSize: '2.2rem',
                boxShadow: 2
              }}
            >
              {formatCurrency(user.balance)}
            </Box>
          </ListItem>
          {index < (data.length || 0) - 1 && <Divider />}
        </Box>
      ))}
    </List>
  </Paper>
);

const ChoresCard = ({ data, sx }: { data: any[], sx: SxProps<Theme> }) => {
  // Group chores by assigned_name
  const groups = data.reduce((acc, chore) => {
    if (!acc[chore.assigned_name]) {
      acc[chore.assigned_name] = [];
    }
    acc[chore.assigned_name].push(chore);
    return acc;
  }, {} as Record<string, any[]>);

  return (
    <Paper elevation={0} sx={[...(Array.isArray(sx) ? sx : [sx]), { border: 4, borderColor: 'warning.main' }]}>
      <Typography variant="h2" gutterBottom color="warning.main" fontWeight="bold" sx={{ fontSize: '4.2rem', textShadow: '0 2px 4px rgba(0,0,0,0.1)', mb: 4, display: 'flex', alignItems: 'center', gap: 2 }}>
        <ChoreIcon fontSize="inherit" />
        Chores
      </Typography>
      <List sx={{ width: '100%', overflowY: 'auto', flex: 1 }} data-card-content>
        {(Object.entries(groups) as [string, any[]][]).map(([name, chores], gIndex) => (
          <Box key={name} sx={{ mb: gIndex < Object.keys(groups).length - 1 ? 4 : 0 }}>
            <Typography variant="h5" color="text.primary" fontWeight="bold" sx={{ fontSize: '1.7rem', mb: 1, px: 2 }}>
              {name}
            </Typography>
            {chores.map((chore: any, cIndex: number) => (
              <Box key={chore.id}>
                <ListItem alignItems="flex-start" sx={{ py: 1 }}>
                  <ListItemIcon sx={{ minWidth: 40, mt: 0.5 }}>
                    <ChoreIcon fontSize="medium" color="warning" />
                  </ListItemIcon>
                  <ListItemText
                    primary={
                      <Typography variant="h6" component="div" sx={{ fontSize: '1.4rem', textShadow: '0 1px 2px rgba(0,0,0,0.05)' }}>
                        {chore.description}
                      </Typography>
                    }
                  />
                </ListItem>
                {cIndex < chores.length - 1 && <Divider variant="inset" component="li" sx={{ ml: 7 }} />}
              </Box>
            ))}
          </Box>
        ))}
      </List>
    </Paper>
  );
};

const EventsCard = ({ events, sx }: { events: EventDisplay[], sx: SxProps<Theme> }) => (
  <Paper elevation={0} sx={[...(Array.isArray(sx) ? sx : [sx]), { border: 4, borderColor: 'primary.main' }]}>
    <Typography variant="h2" gutterBottom color="primary.main" fontWeight="bold" sx={{ fontSize: '4.2rem', textShadow: '0 2px 4px rgba(0,0,0,0.1)', mb: 4, display: 'flex', alignItems: 'center', gap: 2 }}>
      <EventIcon fontSize="inherit" />
      Events
    </Typography>
    <List sx={{ width: '100%', overflowY: 'auto', flex: 1 }} data-card-content>
      {events.length > 0 ? (
        events.map((event, index) => (
          <Box key={index}>
            <ListItem alignItems="flex-start" sx={{ py: 2 }}>
              <ListItemIcon sx={{ minWidth: 50, mt: 1 }}>
                <EventIcon fontSize="large" sx={{ color: `${event.color}.main` }} />
              </ListItemIcon>
              <ListItemText
                primary={
                  <Typography variant="h5" component="div" gutterBottom sx={{ fontSize: '1.8rem', textShadow: '0 1px 2px rgba(0,0,0,0.1)' }}>
                    {event.summary}
                  </Typography>
                }
                secondaryTypographyProps={{ component: 'div' }}
                secondary={
                  <Box>
                    <Typography variant="h6" component="span" color="text.primary" sx={{ fontSize: '1.4rem' }}>
                      {event.startDate.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' })}
                    </Typography>
                    <Typography variant="h6" component="span" color="text.secondary" sx={{ ml: 2, fontSize: '1.4rem' }}>
                      {event.startDate.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })}
                    </Typography>
                    <Typography variant="body1" color="text.secondary" sx={{ mt: 0.5, fontSize: '1.1rem' }}>
                      {event.calendarName}
                    </Typography>
                  </Box>
                }
              />
            </ListItem>
            {index < events.length - 1 && <Divider component="li" />}
          </Box>
        ))
      ) : (
        <Box display="flex" justifyContent="center" alignItems="center" height="200px">
          <Typography variant="h5" color="text.secondary" sx={{ fontSize: '1.7rem' }}>No upcoming events.</Typography>
        </Box>
      )}
    </List>
  </Paper>
);

export default function DisplayPage() {
  const [token, setToken] = useState<string>(() => localStorage.getItem('display_token') || '');
  const [inputToken, setInputToken] = useState('');
  
  // Settings Menu State
  const [settingsAnchorEl, setSettingsAnchorEl] = useState<null | HTMLElement>(null);
  const [qrDialogOpen, setQrDialogOpen] = useState(false);
  
  // Data State
  const [upcomingEvents, setUpcomingEvents] = useState<EventDisplay[]>([]);

  // Scroll ref for container
  const scrollRef = useRef<HTMLDivElement>(null);

  // Screensaver State
  const [isScreensaverActive, setIsScreensaverActive] = useState(false);
  const activityTimerRef = useRef<number | null>(null);

  // Activity Monitor
  useEffect(() => {
    const resetTimer = () => {
      setIsScreensaverActive(false);
      if (activityTimerRef.current) {
        clearTimeout(activityTimerRef.current);
      }
      activityTimerRef.current = setTimeout(() => {
        setIsScreensaverActive(true);
      }, 60000); // 1 minute
    };

    const events = ['mousemove', 'mousedown', 'touchstart', 'click', 'scroll', 'keydown'];
    events.forEach(event => window.addEventListener(event, resetTimer));
    resetTimer();

    return () => {
      if (activityTimerRef.current) {
        clearTimeout(activityTimerRef.current);
      }
      events.forEach(event => window.removeEventListener(event, resetTimer));
    };
  }, []);

  // Fetch Display Data using the token
  const { data: displayData, error, isLoading } = useQuery({
    queryKey: ['displayData', token],
    queryFn: () => displayApi.getDisplayData(token),
    enabled: !!token,
    refetchInterval: 1000 * 60,
    retry: false
  });

  const handleSettingsClick = (event: React.MouseEvent<HTMLElement>) => {
    setSettingsAnchorEl(event.currentTarget);
  };

  const handleSettingsClose = () => {
    setSettingsAnchorEl(null);
  };

  const handleQrDialogOpen = () => {
    setQrDialogOpen(true);
    handleSettingsClose();
  };

  const handleQrDialogClose = () => {
    setQrDialogOpen(false);
  };

  const frontendUrl = `${window.location.origin}/login`;

  const handleTokenSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setToken(inputToken);
    localStorage.setItem('display_token', inputToken);
  };

  const weatherData = displayData?.weather as Record<string, any> | undefined;

  useEffect(() => {
    const fetchAndParseCalendars = async () => {
      if (!displayData?.calendars || displayData.calendars.length === 0) {
        setUpcomingEvents([]);
        return;
      }

      const allEvents: EventDisplay[] = [];
      const now = new Date();
      const endRange = new Date();
      endRange.setDate(now.getDate() + 7);

      await Promise.all(
        displayData.calendars.map(async (cal) => {
          try {
            const resp = await client.get(
              `/calendars/${cal.id}/feed`,
              {
                headers: {
                  'X-Display-Token': token,
                },
              },
            );

            const contentType = resp.headers['content-type'] || '';

            if (contentType.includes('application/json')) {
                const events: any[] = resp.data;
                events.forEach(event => {
                    const startStr = event.start?.dateTime || event.start?.date;
                    if (!startStr) return;
                    
                    const startDate = new Date(startStr);
                    if (startDate >= now && startDate <= endRange) {
                        allEvents.push({
                            summary: event.summary || 'No Title',
                            startDate: startDate,
                            calendarName: cal.name,
                            color: cal.color,
                        });
                    }
                });

            } else {
                const jcalData = ICAL.parse(resp.data);
                const comp = new ICAL.Component(jcalData);
                const vevents = comp.getAllSubcomponents('vevent');
                
                const nowICAL = ICAL.Time.now();
                const endRangeICAL = ICAL.Time.now();
                endRangeICAL.day += 7;

                vevents.forEach((vevent) => {
                  const event = new ICAL.Event(vevent);

                  if (event.isRecurring()) {
                    const expand = event.iterator();
                    let next;
                    while ((next = expand.next())) {
                      if (next.compare(nowICAL) >= 0) {
                        if (next.compare(endRangeICAL) > 0) break;

                        const details = event.getOccurrenceDetails(next);
                        allEvents.push({
                          summary: details.item?.summary || event.summary || 'No Title',
                          startDate: details.startDate.toJSDate(),
                          calendarName: cal.name,
                          color: cal.color,
                        });
                      }
                    }
                  } else {
                    if (event.startDate.compare(nowICAL) >= 0 && event.startDate.compare(endRangeICAL) <= 0) {
                      allEvents.push({
                        summary: event.summary || 'No Title',
                        startDate: event.startDate.toJSDate(),
                        calendarName: cal.name,
                        color: cal.color,
                      });
                    }
                  }
                });
            }

          } catch (err) {
            console.error(`Failed to parse calendar ${cal.name}`, err);
          }
        }),
      );

      allEvents.sort((a, b) => a.startDate.getTime() - b.startDate.getTime());
      setUpcomingEvents(allEvents.slice(0, 10));
    };

    fetchAndParseCalendars();
  }, [displayData, token]);

  const handleWheel = (e: React.WheelEvent) => {
    if (!scrollRef.current) return;
    const target = e.target as HTMLElement;
    const isInsideScrollableContent = target.closest('[data-card-content]');
    if (isInsideScrollableContent) return;
    
    e.preventDefault();
    e.stopPropagation();
    
    const scrollAmount = e.deltaY + e.deltaX;
    scrollRef.current.scrollBy({
      left: scrollAmount,
      behavior: 'auto'
    });
  };

  if (!token || error) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight="100vh" bgcolor="background.default">
        <Paper sx={{ p: 4, width: '100%', maxWidth: 400 }}>
          <Typography variant="h5" mb={3} textAlign="center">Display Login</Typography>
          {error && <Typography color="error" mb={2} textAlign="center">Invalid Token</Typography>}
          <form onSubmit={handleTokenSubmit}>
            <TextField
              label="Display Token"
              fullWidth
              value={inputToken}
              onChange={(e) => setInputToken(e.target.value)}
              margin="normal"
            />
            <Button type="submit" variant="contained" fullWidth size="large" sx={{ mt: 2 }}>
              Start Display
            </Button>
          </form>
        </Paper>
      </Box>
    );
  }

  if (isLoading) return <Box display="flex" justifyContent="center" mt={4}><CircularProgress /></Box>;

  const cardSx = {
    minWidth: { xs: '85vw', md: '500px' },
    height: '80vh',
    bgcolor: 'rgba(255, 255, 255, 0.55)',
    backdropFilter: 'blur(16px)',
    p: 4,
    borderRadius: 6,
    display: 'flex',
    flexDirection: 'column',
    boxShadow: '0 8px 32px 0 rgba(0, 0, 0, 0.3)',
    border: '1px solid rgba(255, 255, 255, 0.18)',
    overflow: 'hidden',
    userSelect: 'none',
  };

  const cards: React.ReactNode[] = [];
  if (weatherData) {
    cards.push(<WeatherCard key="weather" data={weatherData} sx={cardSx} />);
  }
  if (displayData?.allowances && displayData.allowances.length > 0) {
    cards.push(<AllowanceCard key="allowance" data={displayData.allowances} sx={cardSx} />);
  }
  if (displayData?.chores && displayData.chores.length > 0) {
    cards.push(<ChoresCard key="chores" data={displayData.chores} sx={cardSx} />);
  }
  cards.push(<EventsCard key="events" events={upcomingEvents} sx={cardSx} />);

  return (
    <Box 
      sx={{ 
        height: '100vh', 
        width: '100vw', 
        overflow: 'hidden', 
        position: 'relative',
        bgcolor: 'black',
      }}
    >
      {displayData?.background_url && (
        <Box
          sx={{
            position: 'absolute',
            top: -40, left: -40, right: -40, bottom: -40,
            backgroundImage: `url('${getImageUrl(displayData.background_url, token)}')`,
            backgroundSize: 'cover',
            backgroundPosition: 'center',
            filter: 'blur(30px) brightness(0.5)',
            zIndex: 0,
          }}
        />
      )}

      {displayData?.background_url && (
        <Box
          sx={{
            position: 'absolute',
            top: 0, left: 0, right: 0, bottom: 0,
            backgroundImage: `url('${getImageUrl(displayData.background_url, token)}')`,
            backgroundSize: 'contain',
            backgroundRepeat: 'no-repeat',
            backgroundPosition: 'center',
            zIndex: 0,
            filter: 'drop-shadow(0 0 20px rgba(0,0,0,0.5))'
          }}
        />
      )}

      <Box
        ref={scrollRef}
        onWheel={handleWheel}
        sx={{
          display: 'flex',
          flexDirection: 'row',
          overflowX: 'scroll',
          overflowY: 'hidden',
          height: '100%',
          alignItems: 'center',
          gap: { xs: 2, md: 6 },
          px: { xs: 2, md: 6 },
          position: 'relative',
          zIndex: 1,
          scrollbarWidth: 'thin',
          '&::-webkit-scrollbar': { 
            height: '8px',
          },
          '&::-webkit-scrollbar-track': {
            background: 'rgba(255, 255, 255, 0.1)',
            borderRadius: '4px',
          },
          '&::-webkit-scrollbar-thumb': {
            background: 'rgba(255, 255, 255, 0.3)',
            borderRadius: '4px',
            '&:hover': {
              background: 'rgba(255, 255, 255, 0.5)',
            },
          },
          opacity: isScreensaverActive ? 0 : 1,
          transition: 'opacity 0.5s ease-in-out',
          pointerEvents: isScreensaverActive ? 'none' : 'auto',
        }}
      >
        {cards}
      </Box>

      <IconButton
        onClick={handleSettingsClick}
        sx={{
          position: 'absolute',
          top: 16,
          right: 16,
          zIndex: 10,
          color: 'white',
          bgcolor: 'rgba(0, 0, 0, 0.2)',
          backdropFilter: 'blur(8px)',
          opacity: isScreensaverActive ? 0 : 0.3,
          transition: 'opacity 0.3s ease-in-out, background-color 0.2s',
          '&:hover': {
            bgcolor: 'rgba(0, 0, 0, 0.4)',
            opacity: 0.9,
          },
        }}
      >
        <SettingsIcon fontSize="large" />
      </IconButton>

      <Menu
        anchorEl={settingsAnchorEl}
        open={Boolean(settingsAnchorEl)}
        onClose={handleSettingsClose}
        anchorOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
      >
        <MenuItem onClick={handleQrDialogOpen}>
          <ListItemIcon>
            <QrCodeIcon />
          </ListItemIcon>
          <ListItemText>Change Settings</ListItemText>
        </MenuItem>
      </Menu>

      <Dialog 
        open={qrDialogOpen} 
        onClose={handleQrDialogClose}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          <Box display="flex" alignItems="center" gap={1}>
            <QrCodeIcon />
            <Typography variant="h6">Scan to Configure</Typography>
          </Box>
        </DialogTitle>
        <DialogContent>
          <Box 
            display="flex" 
            flexDirection="column" 
            alignItems="center" 
            gap={2}
            py={2}
          >
            <Typography variant="body1" color="text.secondary" textAlign="center">
              Scan this QR code with your phone to access the settings page
            </Typography>
            <Box 
              p={2} 
              bgcolor="white" 
              borderRadius={2}
              display="flex"
              justifyContent="center"
            >
              <QRCodeSVG 
                value={frontendUrl} 
                size={256}
                level="H"
                includeMargin
              />
            </Box>
            <Typography variant="caption" color="text.secondary">
              {frontendUrl}
            </Typography>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={handleQrDialogClose}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
