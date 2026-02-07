export const formatCurrency = (cents: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(cents / 100);
};

export const parseCurrencyToCents = (amount: string): number => {
  return Math.round(parseFloat(amount) * 100);
};
