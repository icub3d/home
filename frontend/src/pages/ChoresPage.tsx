import { useState } from 'react';
import {
  Typography,
  Paper,
  Box,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Checkbox,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Divider,
  Chip
} from '@mui/material';
import { Add as AddIcon, Delete as DeleteIcon, Edit as EditIcon } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { choreApi } from '../api';
import { usersApi } from '../api';
import type { ChoreWithUser, UpdateChoreInput } from '../types';
import type { User } from '../types';
import { useAuth } from '../context/AuthContext';

export default function ChoresPage() {
  const queryClient = useQueryClient();
  const { isAdmin } = useAuth();
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editingChore, setEditingChore] = useState<ChoreWithUser | null>(null);
  const [description, setDescription] = useState('');
  const [assignedTo, setAssignedTo] = useState('');

  const { data: chores, isLoading: isLoadingChores } = useQuery({
    queryKey: ['chores'],
    queryFn: choreApi.getChores,
  });

  const { data: users } = useQuery({
    queryKey: ['users'],
    queryFn: usersApi.getUsers,
    enabled: isAdmin,
  });

  const createMutation = useMutation({
    mutationFn: choreApi.createChore,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['chores'] });
      handleCloseDialog();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateChoreInput }) =>
      choreApi.updateChore(id, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['chores'] });
      handleCloseDialog();
    },
  });

  const deleteMutation = useMutation({
    mutationFn: choreApi.deleteChore,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['chores'] });
    },
  });

  const toggleMutation = useMutation({
    mutationFn: choreApi.toggleComplete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['chores'] });
    },
  });

  const handleAddClick = () => {
    setEditingChore(null);
    setDescription('');
    setAssignedTo('');
    setIsDialogOpen(true);
  };

  const handleEditClick = (chore: ChoreWithUser) => {
    setEditingChore(chore);
    setDescription(chore.description);
    setAssignedTo(chore.assigned_to);
    setIsDialogOpen(true);
  };

  const handleCloseDialog = () => {
    setIsDialogOpen(false);
    setEditingChore(null);
    setDescription('');
    setAssignedTo('');
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (editingChore) {
      updateMutation.mutate({
        id: editingChore.id,
        input: { description, assigned_to: assignedTo },
      });
    } else {
      createMutation.mutate({ description, assigned_to: assignedTo });
    }
  };

  const handleToggle = (id: string) => {
    toggleMutation.mutate(id);
  };

  const handleDelete = (id: string) => {
    if (window.confirm('Are you sure you want to delete this chore?')) {
      deleteMutation.mutate(id);
    }
  };

  if (isLoadingChores) return <Box display="flex" justifyContent="center" mt={4}>Loading...</Box>;

  // Group chores by user
  const choresByUser: { [key: string]: ChoreWithUser[] } = {};
  chores?.forEach((chore) => {
    if (!choresByUser[chore.assigned_name]) {
      choresByUser[chore.assigned_name] = [];
    }
    choresByUser[chore.assigned_name].push(chore);
  });

  return (
    <Box>
      {isAdmin && (
        <Box display="flex" justifyContent="flex-end" alignItems="center" mb={3}>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={handleAddClick}
          >
            Add Chore
          </Button>
        </Box>
      )}

      {Object.keys(choresByUser).length === 0 ? (
        <Paper sx={{ p: 3 }}>
          <Typography color="text.secondary">No chores assigned yet.</Typography>
        </Paper>
      ) : !isAdmin ? (
        <Paper sx={{ p: 2 }}>
          <Typography variant="h6" gutterBottom>
            My Chores
            <Chip 
              label={`${chores?.filter(c => !c.completed).length || 0} pending`} 
              size="small" 
              sx={{ ml: 2 }}
              color={chores?.filter(c => !c.completed).length === 0 ? 'success' : 'default'}
            />
          </Typography>
          <Divider sx={{ mb: 1 }} />
          <List dense>
            {chores?.map((chore) => (
              <ListItem key={chore.id}>
                <Checkbox
                  edge="start"
                  checked={chore.completed}
                  onChange={() => handleToggle(chore.id)}
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
            ))}
          </List>
        </Paper>
      ) : (
        Object.keys(choresByUser).sort().map((userName) => (
          <Paper key={userName} sx={{ mb: 2, p: 2 }}>
            <Typography variant="h6" gutterBottom>
              {userName}
              <Chip 
                label={`${choresByUser[userName].filter(c => !c.completed).length} pending`} 
                size="small" 
                sx={{ ml: 2 }}
                color={choresByUser[userName].filter(c => !c.completed).length === 0 ? 'success' : 'default'}
              />
            </Typography>
            <Divider sx={{ mb: 1 }} />
            <List dense>
              {choresByUser[userName].map((chore) => (
                <ListItem key={chore.id}>
                  <Checkbox
                    edge="start"
                    checked={chore.completed}
                    onChange={() => handleToggle(chore.id)}
                    sx={{ mr: 1 }}
                  />
                  <ListItemText
                    primary={chore.description}
                    sx={{
                      textDecoration: chore.completed ? 'line-through' : 'none',
                      color: chore.completed ? 'text.secondary' : 'text.primary',
                    }}
                  />
                  {isAdmin && (
                    <ListItemSecondaryAction>
                      <IconButton edge="end" size="small" onClick={() => handleEditClick(chore)} sx={{ mr: 1 }}>
                        <EditIcon fontSize="small" />
                      </IconButton>
                      <IconButton edge="end" size="small" onClick={() => handleDelete(chore.id)} color="error">
                        <DeleteIcon fontSize="small" />
                      </IconButton>
                    </ListItemSecondaryAction>
                  )}
                </ListItem>
              ))}
            </List>
          </Paper>
        ))
      )}

      {/* Add/Edit Dialog */}
      <Dialog open={isDialogOpen} onClose={handleCloseDialog} maxWidth="sm" fullWidth>
        <form onSubmit={handleSubmit}>
          <DialogTitle>{editingChore ? 'Edit Chore' : 'Add Chore'}</DialogTitle>
          <DialogContent>
            <Box display="flex" flexDirection="column" gap={2} mt={1}>
              <TextField
                label="Description"
                fullWidth
                required
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                autoFocus
              />
              <FormControl fullWidth required>
                <InputLabel>Assign To</InputLabel>
                <Select
                  value={assignedTo}
                  label="Assign To"
                  onChange={(e) => setAssignedTo(e.target.value)}
                >
                  {users?.map((user: User) => (
                    <MenuItem key={user.id} value={user.id}>
                      {user.name}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Box>
          </DialogContent>
          <DialogActions>
            <Button onClick={handleCloseDialog}>Cancel</Button>
            <Button type="submit" variant="contained">
              {editingChore ? 'Update' : 'Add'}
            </Button>
          </DialogActions>
        </form>
      </Dialog>
    </Box>
  );
}
