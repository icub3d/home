import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ThemeProvider } from '@mui/material/styles';
import { CssBaseline } from '@mui/material';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import UsersPage from './pages/UsersPage';
import AllowancePage from './pages/AllowancePage';
import ChoresPage from './pages/ChoresPage';
import SettingsPage from './pages/SettingsPage';
import DisplayPage from './pages/DisplayPage';
import LoginPage from './pages/LoginPage';
import RegisterPage from './pages/RegisterPage';
import ProtectedRoute from './components/ProtectedRoute';
import { theme } from './theme';
import { AuthProvider } from './context/AuthContext';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <ThemeProvider theme={theme} defaultMode="light" noSsr>
          <CssBaseline />
          <Router>
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/login" element={<LoginPage />} />
              <Route path="/setup" element={<RegisterPage />} />
              <Route path="/display" element={<DisplayPage />} />
              <Route path="/dashboard" element={
                <ProtectedRoute>
                  <Layout>
                    <Dashboard />
                  </Layout>
                </ProtectedRoute>
              } />
              <Route path="/users" element={
                <ProtectedRoute requireAdmin={true}>
                  <Layout>
                    <UsersPage />
                  </Layout>
                </ProtectedRoute>
              } />
              <Route path="/allowance" element={
                <ProtectedRoute>
                  <Layout>
                    <AllowancePage />
                  </Layout>
                </ProtectedRoute>
              } />
              <Route path="/chores" element={
                <ProtectedRoute>
                  <Layout>
                    <ChoresPage />
                  </Layout>
                </ProtectedRoute>
              } />
              <Route path="/settings" element={
                <ProtectedRoute requireAdmin={true}>
                  <Layout>
                    <SettingsPage />
                  </Layout>
                </ProtectedRoute>
              } />
              {/* Legacy redirect */}
              <Route path="/register" element={<Navigate to="/setup" replace />} />
            </Routes>
          </Router>
        </ThemeProvider>
      </AuthProvider>
    </QueryClientProvider>
  );
}

export default App;
