import { chainService } from '@/src/services/chain';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useEffect, useState } from 'react';
import { ActivityIndicator, Pressable, StyleSheet, Text, View } from 'react-native';

interface BalanceDisplayProps {
  compact?: boolean;
  showRefresh?: boolean;
}

export function BalanceDisplay({ compact = false, showRefresh = true }: BalanceDisplayProps) {
  const { address } = useAuthStore();
  const { isConnected } = useChainStore();
  const [balance, setBalance] = useState<string>('0');
  const [loading, setLoading] = useState(false);

  const fetchBalance = async () => {
    if (!address || !isConnected) return;
    
    setLoading(true);
    try {
      const result = await chainService.getBalance(address);
      const formatted = (parseFloat(result) / 1e10).toFixed(4);
      setBalance(formatted);
    } catch (error) {
      console.error('Failed to fetch balance:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchBalance();
  }, [address, isConnected]);

  if (compact) {
    return (
      <View style={styles.compactContainer}>
        <Text style={styles.compactLabel}>余额</Text>
        {loading ? (
          <ActivityIndicator size="small" color="#6D28D9" />
        ) : (
          <Text style={styles.compactValue}>{balance} STAR</Text>
        )}
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.label}>账户余额</Text>
        {showRefresh && (
          <Pressable onPress={fetchBalance} disabled={loading}>
            <Text style={styles.refreshText}>{loading ? '刷新中...' : '🔄 刷新'}</Text>
          </Pressable>
        )}
      </View>
      
      <View style={styles.balanceRow}>
        {loading ? (
          <ActivityIndicator size="large" color="#6D28D9" />
        ) : (
          <>
            <Text style={styles.value}>{balance}</Text>
            <Text style={styles.unit}>STAR</Text>
          </>
        )}
      </View>
      
      {!isConnected && (
        <Text style={styles.offlineHint}>⚠️ 链未连接</Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#6D28D9',
    borderRadius: 16,
    padding: 20,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  label: {
    fontSize: 14,
    color: 'rgba(255,255,255,0.8)',
  },
  refreshText: {
    fontSize: 13,
    color: 'rgba(255,255,255,0.7)',
  },
  balanceRow: {
    flexDirection: 'row',
    alignItems: 'baseline',
  },
  value: {
    fontSize: 36,
    fontWeight: '700',
    color: '#fff',
  },
  unit: {
    fontSize: 18,
    color: 'rgba(255,255,255,0.8)',
    marginLeft: 8,
  },
  offlineHint: {
    fontSize: 12,
    color: '#fbbf24',
    marginTop: 8,
  },
  compactContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  compactLabel: {
    fontSize: 13,
    color: '#6b7280',
  },
  compactValue: {
    fontSize: 15,
    fontWeight: '600',
    color: '#6D28D9',
  },
});
