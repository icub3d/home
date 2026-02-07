import { useState } from 'react';
import {
  Typography,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Box,
  CircularProgress,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
  Card,
  CardContent
} from '@mui/material';
import { Add as AddIcon, History as HistoryIcon } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { allowanceApi } from '../api';
import type { UserBalance } from '../types';
import { useAuth } from '../context/AuthContext';
import { formatCurrency, parseCurrencyToCents } from '../utils/currency';

export default function AllowancePage() {
  const queryClient = useQueryClient();
  const { isAdmin, userId } = useAuth();
  const [selectedUser, setSelectedUser] = useState<UserBalance | null>(null);
  const [isTxDialogOpen, setIsTxDialogOpen] = useState(false);
  const [txAmount, setTxAmount] = useState('');
  const [txDescription, setTxDescription] = useState('');
  const [viewingLedger, setViewingLedger] = useState<string | null>(null);

  const { data: balances, isLoading: isLoadingBalances } = useQuery({
    queryKey: ['allowanceBalances'],
    queryFn: allowanceApi.getBalances,
  });

  const { data: myLedger, isLoading: isLoadingMyLedger } = useQuery({
    queryKey: ['ledger', userId],
    queryFn: () => allowanceApi.getLedger(userId!),
    enabled: !isAdmin && !!userId,
  });

  const { data: ledger, isLoading: isLoadingLedger } = useQuery({
    queryKey: ['ledger', viewingLedger],
    queryFn: () => allowanceApi.getLedger(viewingLedger!),
    enabled: isAdmin && !!viewingLedger,
  });

  const addTxMutation = useMutation({
    mutationFn: ({ userId, amount, description }: { userId: string, amount: number, description: string }) =>
      allowanceApi.addTransaction(userId, amount, description),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['allowanceBalances'] });
      queryClient.invalidateQueries({ queryKey: ['ledger'] });
      setIsTxDialogOpen(false);
      setTxAmount('');
      setTxDescription('');
    }
  });

  const handleAddFundsClick = (user: UserBalance) => {
    setSelectedUser(user);
    setIsTxDialogOpen(true);
  };

  const handleViewLedgerClick = (userId: string) => {
    if (viewingLedger === userId) {
      setViewingLedger(null); // Toggle off
    } else {
      setViewingLedger(userId);
    }
  };

  const handleTxSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedUser) return;
    
    const amountInCents = parseCurrencyToCents(txAmount);
    
    addTxMutation.mutate({
      userId: selectedUser.user_id,
      amount: amountInCents,
      description: txDescription
    });
  };

  if (isLoadingBalances || (!isAdmin && isLoadingMyLedger)) {
    return <Box display="flex" justifyContent="center" mt={4}><CircularProgress /></Box>;
  }

  // Non-admin view - show only their balance and recent transactions
  if (!isAdmin) {
    const myBalance = balances?.find(b => b.user_id === userId);
    const recentTransactions = myLedger?.slice(0, 10) || [];

    return (
      <Box>
        {/* Balance Card */}
        <Card sx={{ mb: 3, bgcolor: 'primary.main', color: 'primary.contrastText' }}>
          <CardContent sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center' }}>
            <Typography variant="h6" gutterBottom>Current Balance</Typography>
            <Typography variant="h2" fontWeight="bold">
              {myBalance ? formatCurrency(myBalance.balance) : formatCurrency(0)}
            </Typography>
          </CardContent>
        </Card>

        {/* Recent Transactions */}
        <Paper>
          <Box p={2}>
            <Typography variant="h6" mb={2}>Recent Transactions</Typography>
            {recentTransactions.length === 0 ? (
              <Typography color="text.secondary">No transactions yet.</Typography>
            ) : (
              <TableContainer>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Date</TableCell>
                      <TableCell>Description</TableCell>
                      <TableCell align="right">Amount</TableCell>
                      <TableCell align="right">Balance</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {recentTransactions.map((tx) => (
                      <TableRow key={tx.id}>
                        <TableCell>{new Date(tx.created_at).toLocaleDateString()}</TableCell>
                        <TableCell>{tx.description}</TableCell>
                        <TableCell align="right" sx={{ color: tx.amount < 0 ? 'error.main' : 'success.main', fontWeight: 'bold' }}>
                          {formatCurrency(tx.amount)}
                        </TableCell>
                        <TableCell align="right">{formatCurrency(tx.balance)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            )}
          </Box>
        </Paper>
      </Box>
    );
  }

  // Admin view - original functionality
  return (
    <Box>
      <Grid container spacing={3}>
        <Grid size={{ xs: 12, md: viewingLedger ? 6 : 12 }}>
          <TableContainer component={Paper}>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Name</TableCell>
                  <TableCell align="right">Balance</TableCell>
                  <TableCell align="right">Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {balances?.map((user) => (
                  <TableRow key={user.user_id} selected={viewingLedger === user.user_id}>
                    <TableCell>{user.name}</TableCell>
                    <TableCell align="right">
                      <Box
                        component="span"
                        sx={{
                          bgcolor: user.balance < 0 ? 'error.light' : 'success.light',
                          color: user.balance < 0 ? 'error.contrastText' : 'success.contrastText',
                          px: 1.5,
                          py: 0.5,
                          borderRadius: 4,
                          fontWeight: 'bold',
                          display: 'inline-block'
                        }}
                      >
                        {formatCurrency(user.balance)}
                      </Box>
                    </TableCell>
                    <TableCell align="right">
                      <Button 
                        startIcon={<HistoryIcon />} 
                        size="small" 
                        onClick={() => handleViewLedgerClick(user.user_id)}
                        sx={{ mr: 1 }}
                      >
                        History
                      </Button>
                      <Button 
                        startIcon={<AddIcon />} 
                        size="small" 
                        variant="contained"
                        onClick={() => handleAddFundsClick(user)}
                      >
                        Add Funds
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
                {balances?.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={3} align="center">No accounts tracking allowance.</TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </TableContainer>
        </Grid>

        {viewingLedger && (
          <Grid size={{ xs: 12, md: 6 }}>
             <Paper sx={{ p: 2 }}>
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                  <Typography variant="h6">Transaction History</Typography>
                  <Button size="small" onClick={() => setViewingLedger(null)}>Close</Button>
                </Box>
                {isLoadingLedger ? (
                  <CircularProgress size={20} />
                ) : (
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>Date</TableCell>
                        <TableCell>Description</TableCell>
                        <TableCell align="right">Amount</TableCell>
                        <TableCell align="right">Balance</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {ledger?.map((tx) => (
                        <TableRow key={tx.id}>
                          <TableCell>{new Date(tx.created_at).toLocaleDateString()}</TableCell>
                          <TableCell>{tx.description}</TableCell>
                          <TableCell align="right" sx={{ color: tx.amount < 0 ? 'error.main' : 'success.main' }}>
                            {formatCurrency(tx.amount)}
                          </TableCell>
                          <TableCell align="right">{formatCurrency(tx.balance)}</TableCell>
                        </TableRow>
                      ))}
                      {ledger?.length === 0 && (
                        <TableRow>
                          <TableCell colSpan={4} align="center">No transactions yet.</TableCell>
                        </TableRow>
                      )}
                    </TableBody>
                  </Table>
                )}
             </Paper>
          </Grid>
        )}
      </Grid>

      <Dialog open={isTxDialogOpen} onClose={() => setIsTxDialogOpen(false)}>
        <form onSubmit={handleTxSubmit}>
          <DialogTitle>Add Funds for {selectedUser?.name}</DialogTitle>
          <DialogContent>
            <Box display="flex" flexDirection="column" gap={2} mt={1} minWidth={300}>
              <TextField
                label="Amount ($)"
                type="number"
                fullWidth
                required
                value={txAmount}
                onChange={(e) => setTxAmount(e.target.value)}
                inputProps={{ step: "0.01" }}
              />
              <TextField
                label="Description"
                fullWidth
                required
                value={txDescription}
                onChange={(e) => setTxDescription(e.target.value)}
              />
            </Box>
          </DialogContent>
          <DialogActions>
            <Button onClick={() => setIsTxDialogOpen(false)}>Cancel</Button>
            <Button type="submit" variant="contained">Add</Button>
          </DialogActions>
        </form>
      </Dialog>
    </Box>
  );
}
