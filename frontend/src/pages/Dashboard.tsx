import { Typography, Grid, Paper, Box, List, ListItem, ListItemIcon, ListItemText, Divider, useTheme, Checkbox } from '@mui/material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { allowanceApi, weatherApi, calendarApi, choreApi, client } from '../api';
import type { Calendar } from '../types';
import { useEffect, useState } from 'react';
import ICAL from 'ical.js';
import { Event as EventIcon } from '@mui/icons-material';
import { useAuth } from '../context/AuthContext';
import { formatCurrency } from '../utils/currency';

interface EventDisplay {
  summary: string;
  startDate: Date;
  isRecurring: boolean;
  calendarName: string;
  color: string;
}

export default function Dashboard() {
  const { username, userId, isAdmin } = useAuth();
  const theme = useTheme();
  const queryClient = useQueryClient();

  const { data: balances } = useQuery({
    queryKey: ['allowanceBalances'],
    queryFn: allowanceApi.getBalances,
  });

  const { data: chores } = useQuery({
    queryKey: ['chores'],
    queryFn: choreApi.getChores,
  });

  const { data: weatherData } = useQuery({
    queryKey: ['weather'],
    queryFn: weatherApi.getWeather,
  });

  const { data: calendars } = useQuery({
    queryKey: ['calendars'],
    queryFn: calendarApi.getCalendars,
  });

  const [upcomingEvents, setUpcomingEvents] = useState<EventDisplay[]>([]);

  useEffect(() => {
    const fetchAndParseCalendars = async () => {
      if (!calendars || calendars.length === 0) {
        setUpcomingEvents([]);
        return;
      }

      const allEvents: EventDisplay[] = [];
      const now = new Date();
      const endRange = new Date();
      endRange.setDate(now.getDate() + 7);

      await Promise.all(calendars.map(async (cal: Calendar) => {
        try {
          // Use client directly to check headers
          const resp = await client.get(`/calendars/${cal.id}/feed`);
          const contentType = resp.headers['content-type'] || '';

          if (contentType.includes('application/json')) {
             // Handle Google Calendar JSON
             const events: any[] = resp.data;
             events.forEach(event => {
                 const startStr = event.start?.dateTime || event.start?.date;
                 if (!startStr) return;
                 const startDate = new Date(startStr);
                 
                 if (startDate >= now && startDate <= endRange) {
                     allEvents.push({
                         summary: event.summary || 'No Title',
                         startDate: startDate,
                         isRecurring: false,
                         calendarName: cal.name,
                         color: cal.color,
                     });
                 }
             });
          } else {
             // Handle iCal
             const jcalData = ICAL.parse(resp.data);
             const comp = new ICAL.Component(jcalData);
             const vevents = comp.getAllSubcomponents('vevent');
             
             // ICAL.Time objects
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
                        isRecurring: true,
                        calendarName: cal.name,
                        color: cal.color
                      });
                    }
                  }
                } else {
                  if (event.startDate.compare(nowICAL) >= 0 && event.startDate.compare(endRangeICAL) <= 0) {
                    allEvents.push({
                      summary: event.summary || 'No Title',
                      startDate: event.startDate.toJSDate(),
                      isRecurring: false,
                      calendarName: cal.name,
                      color: cal.color
                    });
                  }
                }
             });
          }
        } catch (err) {
          console.error(`Failed to parse calendar ${cal.name}`, err);
        }
      }));

      // Sort by date
      allEvents.sort((a, b) => a.startDate.getTime() - b.startDate.getTime());
      setUpcomingEvents(allEvents.slice(0, 10)); // Show next 10 events
    };

    fetchAndParseCalendars();
  }, [calendars]);

  // For non-admin users, show a simplified welcome dashboard
  if (!isAdmin) {
    const myBalance = balances?.find(b => b.user_id === userId);
    const myChores = chores?.filter(c => c.assigned_to === userId) || [];
    const pendingChores = myChores.filter(c => !c.completed);
    
    const toggleMutation = useMutation({
      mutationFn: choreApi.toggleComplete,
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['chores'] });
      },
    });

    const handleToggle = (id: string) => {
      toggleMutation.mutate(id);
    };
    
    return (
      <Box>
        <Paper sx={{ 
          p: 4, 
          mb: 3,
          background: `linear-gradient(135deg, ${theme.palette.primary.main} 0%, ${theme.palette.secondary.main} 100%)`,
          color: '#fff'
        }}>
          <Typography variant="h3" fontWeight="bold">
            Welcome, {username}!
          </Typography>
        </Paper>

        <Grid container spacing={3}>
          {/* Allowance Balance */}
          {myBalance && (
            <Grid size={{ xs: 12, md: 4 }}>
              <Paper sx={{ p: 3, borderLeft: 6, borderColor: 'success.main' }}>
                <Typography variant="h5" gutterBottom color="success.main" fontWeight="bold">
                  Current Balance
                </Typography>
                <Box display="flex" justifyContent="center" mt={2}>
                  <Typography variant="h3" fontWeight="bold" color="success.main">
                    {formatCurrency(myBalance.balance)}
                  </Typography>
                </Box>
              </Paper>
            </Grid>
          )}

          {/* My Chores */}
          <Grid size={{ xs: 12, md: 4 }}>
            <Paper sx={{ p: 3, borderLeft: 6, borderColor: 'primary.main' }}>
              <Typography variant="h5" gutterBottom color="primary.main" fontWeight="bold">
                Chores ({pendingChores.length} pending)
              </Typography>
              <List dense>
                {myChores.length > 0 ? myChores.slice(0, 5).map((chore) => (
                  <ListItem key={chore.id} sx={{ px: 0 }}>
                    <Checkbox
                      edge="start"
                      checked={chore.completed}
                      onChange={() => handleToggle(chore.id)}
                      size="small"
                      sx={{ mr: 1 }}
                    />
                    <ListItemText 
                      primary={chore.description}
                      sx={{
                        textDecoration: chore.completed ? 'line-through' : 'none',
                        color: chore.completed ? 'text.secondary' : 'text.primary',
                      }}
                    />
                  </ListItem>
                )) : (
                  <Typography variant="body2" color="text.secondary">No chores assigned.</Typography>
                )}
              </List>
            </Paper>
          </Grid>

          {/* Upcoming Events */}
          <Grid size={{ xs: 12, md: 4 }}>
            <Paper sx={{ p: 3, borderLeft: 6, borderColor: 'secondary.main' }}>
              <Typography variant="h6" gutterBottom color="secondary.main">
                Upcoming Events
              </Typography>
              <List dense>
                {upcomingEvents.length > 0 ? upcomingEvents.slice(0, 5).map((event, index) => (
                  <ListItem key={index}>
                    <ListItemIcon sx={{ minWidth: 36 }}>
                      <EventIcon fontSize="small" sx={{ color: `${event.color}.main` }} />
                    </ListItemIcon>
                    <ListItemText 
                      primary={event.summary} 
                      secondary={event.startDate.toLocaleDateString(undefined, { 
                        weekday: 'short', 
                        month: 'short', 
                        day: 'numeric', 
                        hour: 'numeric', 
                        minute: '2-digit' 
                      })}
                    />
                  </ListItem>
                )) : (
                  <Typography variant="body2" color="text.secondary">No upcoming events.</Typography>
                )}
              </List>
            </Paper>
          </Grid>
        </Grid>
      </Box>
    );
  }

  // Admin dashboard - full features
  const weather = weatherData as any;
  return (
    <Box>
      <Grid container spacing={3}>
        <Grid size={{ xs: 12, md: 4 }}>
          <Paper sx={{ 
            p: 3, 
            background: `linear-gradient(135deg, ${theme.palette.primary.main} 0%, ${theme.palette.secondary.main} 100%)`,
            color: '#fff'
          }}>
            <Typography variant="h5" fontWeight="bold">Welcome Home!</Typography>
            <Typography variant="body1" sx={{ opacity: 0.9 }}>Manage your family and home from here.</Typography>
          </Paper>

          <Paper sx={{ p: 3, mt: 3, borderLeft: 6, borderColor: 'info.main' }}>
            <Typography variant="h6" gutterBottom color="info.main">Weather</Typography>
            {weather ? (
              <>
                <Box display="flex" alignItems="center" gap={2}>
                  <img
                    src={`https://openweathermap.org/img/wn/${weather.weather[0].icon}@2x.png`}
                    alt={weather.weather[0].description}
                    style={{ width: 64, height: 64 }}
                  />
                  <Box>
                    <Typography variant="h3">
                      {Math.round(weather.main.temp)}°F
                    </Typography>
                    <Typography variant="body1" sx={{ textTransform: 'capitalize' }}>
                      {weather.weather[0].description}
                    </Typography>
                    <Typography variant="caption">
                      H: {Math.round(weather.main.temp_max)}° L: {Math.round(weather.main.temp_min)}°
                    </Typography>
                  </Box>
                </Box>
                <Typography variant="body2" color="text.secondary" mt={1}>
                  {weather.name}
                </Typography>
              </>
            ) : (
              <Typography variant="body2" color="text.secondary">Weather unavailable.</Typography>
            )}
          </Paper>

          {/* Calendar Widget */}
          <Paper sx={{ p: 2, mt: 3, borderLeft: 6, borderColor: 'secondary.main' }}>
            <Typography variant="h6" gutterBottom color="secondary.main">Upcoming Events</Typography>
            <List dense>
              {upcomingEvents.length > 0 ? upcomingEvents.map((event, index) => (
                <ListItem key={index}>
                  <ListItemIcon sx={{ minWidth: 36 }}>
                    <EventIcon fontSize="small" sx={{ color: `${event.color}.main` }} />
                  </ListItemIcon>
                  <ListItemText 
                    primary={event.summary} 
                    secondary={`${event.startDate.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit' })} • ${event.calendarName}`}
                  />
                </ListItem>
              )) : (
                <Typography variant="body2" color="text.secondary">No upcoming events.</Typography>
              )}
            </List>
          </Paper>
        </Grid>
        
        <Grid size={{ xs: 12, md: 4 }}>
           <Paper sx={{ p: 3, borderLeft: 6, borderColor: 'success.main' }}>
            <Typography variant="h6" gutterBottom color="success.main">Allowance Summary</Typography>
            <List dense>
              {balances?.map((user, index) => (
                <Box key={user.user_id}>
                  <ListItem>
                    <ListItemText 
                      primary={user.name} 
                      primaryTypographyProps={{ fontWeight: 'medium' }}
                    />
                    <Box 
                      sx={{ 
                        bgcolor: user.balance < 0 ? 'error.light' : 'success.light',
                        color: user.balance < 0 ? 'error.contrastText' : 'success.contrastText',
                        px: 1, 
                        py: 0.5, 
                        borderRadius: 1,
                        fontWeight: 'bold',
                        fontSize: '0.875rem'
                      }}
                    >
                      {formatCurrency(user.balance)}
                    </Box>
                  </ListItem>
                  {index < balances.length - 1 && <Divider />}
                </Box>
              ))}
              {(!balances || balances.length === 0) && (
                <Typography variant="body2" color="text.secondary">No active accounts.</Typography>
              )}
            </List>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
}
