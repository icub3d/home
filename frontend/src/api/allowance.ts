import type { AxiosInstance } from 'axios';
import type { AllowanceTransaction, UserBalance } from '../types';

export const createAllowanceApi = (client: AxiosInstance) => ({
  getBalances: async (): Promise<UserBalance[]> => {
    const response = await client.get<UserBalance[]>('/allowance/balances');
    return response.data;
  },

  getLedger: async (userId: string): Promise<AllowanceTransaction[]> => {
    const response = await client.get<AllowanceTransaction[]>(`/allowance/${userId}`);
    return response.data;
  },

  addTransaction: async (userId: string, amount: number, description: string): Promise<AllowanceTransaction> => {
    // Amount is in cents
    const response = await client.post<AllowanceTransaction>(`/allowance/${userId}/transaction`, {
      amount,
      description
    });
    return response.data;
  }
});
