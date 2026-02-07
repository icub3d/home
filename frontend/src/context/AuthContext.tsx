import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import type { UserRole } from '../types';

interface AuthContextType {
  userId: string | null;
  username: string | null;
  userRole: UserRole | null;
  token: string | null;
  isAdmin: boolean;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (userId: string, username: string, userRole: UserRole, token: string) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [userId, setUserId] = useState<string | null>(null);
  const [username, setUsername] = useState<string | null>(null);
  const [userRole, setUserRole] = useState<UserRole | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const storedUserId = localStorage.getItem('userId');
    const storedUsername = localStorage.getItem('username');
    const storedUserRole = localStorage.getItem('userRole') as UserRole | null;
    const storedToken = localStorage.getItem('token');
    
    if (storedUserId && storedUsername && storedUserRole && storedToken) {
      setUserId(storedUserId);
      setUsername(storedUsername);
      setUserRole(storedUserRole);
      setToken(storedToken);
    }
    setIsLoading(false);
  }, []);

  const login = (newUserId: string, newUsername: string, newUserRole: UserRole, newToken: string) => {
    localStorage.setItem('token', newToken);
    localStorage.setItem('userId', newUserId);
    localStorage.setItem('username', newUsername);
    localStorage.setItem('userRole', newUserRole);
    setUserId(newUserId);
    setUsername(newUsername);
    setUserRole(newUserRole);
    setToken(newToken);
  };

  const logout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('userId');
    localStorage.removeItem('username');
    localStorage.removeItem('userRole');
    setUserId(null);
    setUsername(null);
    setUserRole(null);
    setToken(null);
  };

  const value: AuthContextType = {
    userId,
    username,
    userRole,
    token,
    isAdmin: userRole === 'admin' || userRole === 'owner',
    isAuthenticated: !!token,
    isLoading,
    login,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
