import React, { useEffect, useState } from 'react';
import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
  MenuItem,
  Box,
  FormControlLabel,
  Checkbox
} from '@mui/material';
import type { User, CreateUserInput, UserRole } from '../types';

interface Props {
  open: boolean;
  onClose: () => void;
  onSave: (user: CreateUserInput | Partial<User>) => void;
  user?: User | null;
}

const roles: UserRole[] = ['admin', 'member', 'child'];

export default function UserDialog({ open, onClose, onSave, user }: Props) {
  const [formData, setFormData] = useState<CreateUserInput>({
    username: '',
    name: '',
    birthday: '',
    role: 'member' as UserRole,
    password: '',
    track_allowance: false,
  });

  useEffect(() => {
    if (open) {
      if (user) {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setFormData({
          username: user.username,
          name: user.name,
          birthday: user.birthday,
          role: user.role,
          password: '', // Password not shown for existing users
          track_allowance: user.track_allowance,
        });
      } else {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setFormData({
          username: '',
          name: '',
          birthday: '',
          role: 'member',
          password: '',
          track_allowance: false,
        });
      }
    }
  }, [user, open]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, checked, type } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(formData);
  };

  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
      <form onSubmit={handleSubmit}>
        <DialogTitle>{user ? 'Edit User' : 'Add Family Member'}</DialogTitle>
        <DialogContent>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 1 }}>
            <TextField
              name="username"
              label="Username"
              fullWidth
              required
              value={formData.username}
              onChange={handleChange}
              disabled={!!user}
            />
            <TextField
              name="name"
              label="Full Name"
              fullWidth
              required
              value={formData.name}
              onChange={handleChange}
            />
            {!user && (
              <TextField
                name="password"
                label="Password"
                type="password"
                fullWidth
                required
                value={formData.password}
                onChange={handleChange}
              />
            )}
            <TextField
              name="birthday"
              label="Birthday"
              type="date"
              fullWidth
              required
              InputLabelProps={{ shrink: true }}
              value={formData.birthday}
              onChange={handleChange}
            />
            <TextField
              name="role"
              label="Role"
              select
              fullWidth
              required
              value={formData.role}
              onChange={handleChange}
            >
              {roles.map((role) => (
                <MenuItem key={role} value={role}>
                  {role.charAt(0).toUpperCase() + role.slice(1)}
                </MenuItem>
              ))}
            </TextField>
            <FormControlLabel
              control={
                <Checkbox
                  name="track_allowance"
                  checked={formData.track_allowance}
                  onChange={handleChange}
                />
              }
              label="Track Allowance"
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={onClose}>Cancel</Button>
          <Button type="submit" variant="contained" color="primary">
            Save
          </Button>
        </DialogActions>
      </form>
    </Dialog>
  );
}