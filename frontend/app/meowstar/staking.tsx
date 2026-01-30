import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, TextInput, Alert } from 'react-native';
import { Coins, Lock, Unlock, TrendingUp, Clock } from 'lucide-react-native';

interface StakeInfo {
  id: number;
  amount: number;
  lockPeriod: string;
  apy: number;
  startDate: string;
  unlockDate: string;
  earnedRewards: number;
  isUnlocked: boolean;
}

type LockPeriod = 'flexible' | '30days' | '90days' | '180days' | '365days';

const LOCK_OPTIONS: { key: LockPeriod; label: string; apy: number; days: number }[] = [
  { key: 'flexible', label: '灵活', apy: 5, days: 0 },
  { key: '30days', label: '30天', apy: 12, days: 30 },
  { key: '90days', label: '90天', apy: 20, days: 90 },
  { key: '180days', label: '180天', apy: 30, days: 180 },
  { key: '365days', label: '365天', apy: 50, days: 365 },
];

export default function StakingScreen() {
  const [stakeAmount, setStakeAmount] = useState('');
  const [selectedPeriod, setSelectedPeriod] = useState<LockPeriod>('30days');
  const [isStaking, setIsStaking] = useState(false);

  const userBalance = 5000;
  const totalStaked = 2500;
  const totalRewards = 125.5;
  const votePower = 3750;

  const stakes: StakeInfo[] = [
    {
      id: 1,
      amount: 1000,
      lockPeriod: '90天',
      apy: 20,
      startDate: '2026-01-15',
      unlockDate: '2026-04-15',
      earnedRewards: 45.5,
      isUnlocked: false,
    },
    {
      id: 2,
      amount: 1500,
      lockPeriod: '30天',
      apy: 12,
      startDate: '2026-01-20',
      unlockDate: '2026-02-19',
      earnedRewards: 80.0,
      isUnlocked: true,
    },
  ];

  const handleStake = () => {
    const amount = parseFloat(stakeAmount);
    if (isNaN(amount) || amount <= 0) {
      Alert.alert('提示', '请输入有效的质押金额');
      return;
    }
    if (amount > userBalance) {
      Alert.alert('提示', '余额不足');
      return;
    }
    if (amount < 100) {
      Alert.alert('提示', '最小质押金额为 100 COS');
      return;
    }

    const option = LOCK_OPTIONS.find(o => o.key === selectedPeriod);
    setIsStaking(true);
    
    setTimeout(() => {
      setIsStaking(false);
      setStakeAmount('');
      Alert.alert(
        '质押成功',
        `已质押 ${amount} COS，锁定期 ${option?.label}，年化收益 ${option?.apy}%`
      );
    }, 1500);
  };

  const handleClaim = (stakeId: number) => {
    Alert.alert('领取成功', '收益已发放到您的账户');
  };

  const handleUnstake = (stakeId: number) => {
    Alert.alert(
      '确认解除质押',
      '确定要解除这笔质押吗？',
      [
        { text: '取消', style: 'cancel' },
        { text: '确认', onPress: () => Alert.alert('成功', '质押已解除') },
      ]
    );
  };

  return (
    <ScrollView style={styles.container}>
      {/* 总览 */}
      <View style={styles.overviewCard}>
        <View style={styles.overviewRow}>
          <View style={styles.overviewItem}>
            <Coins size={24} color="#F7DC6F" />
            <Text style={styles.overviewValue}>{totalStaked} COS</Text>
            <Text style={styles.overviewLabel}>总质押</Text>
          </View>
          <View style={styles.overviewItem}>
            <TrendingUp size={24} color="#4ECDC4" />
            <Text style={styles.overviewValue}>{totalRewards} COS</Text>
            <Text style={styles.overviewLabel}>累计收益</Text>
          </View>
          <View style={styles.overviewItem}>
            <Lock size={24} color="#BB8FCE" />
            <Text style={styles.overviewValue}>{votePower}</Text>
            <Text style={styles.overviewLabel}>投票权重</Text>
          </View>
        </View>
      </View>

      {/* 新质押 */}
      <View style={styles.stakeCard}>
        <Text style={styles.sectionTitle}>新建质押</Text>
        
        <View style={styles.inputContainer}>
          <TextInput
            style={styles.input}
            placeholder="输入质押金额"
            placeholderTextColor="#666"
            keyboardType="numeric"
            value={stakeAmount}
            onChangeText={setStakeAmount}
          />
          <TouchableOpacity
            style={styles.maxButton}
            onPress={() => setStakeAmount(userBalance.toString())}
          >
            <Text style={styles.maxButtonText}>MAX</Text>
          </TouchableOpacity>
        </View>
        <Text style={styles.balanceText}>可用余额: {userBalance} COS</Text>

        <Text style={styles.subTitle}>选择锁定期</Text>
        <View style={styles.periodContainer}>
          {LOCK_OPTIONS.map((option) => (
            <TouchableOpacity
              key={option.key}
              style={[
                styles.periodOption,
                selectedPeriod === option.key && styles.periodOptionActive,
              ]}
              onPress={() => setSelectedPeriod(option.key)}
            >
              <Text style={[
                styles.periodLabel,
                selectedPeriod === option.key && styles.periodLabelActive,
              ]}>
                {option.label}
              </Text>
              <Text style={[
                styles.periodApy,
                selectedPeriod === option.key && styles.periodApyActive,
              ]}>
                {option.apy}% APY
              </Text>
            </TouchableOpacity>
          ))}
        </View>

        <TouchableOpacity
          style={[styles.stakeButton, isStaking && styles.stakeButtonDisabled]}
          onPress={handleStake}
          disabled={isStaking}
        >
          <Text style={styles.stakeButtonText}>
            {isStaking ? '处理中...' : '确认质押'}
          </Text>
        </TouchableOpacity>
      </View>

      {/* 我的质押 */}
      <Text style={styles.sectionTitle}>我的质押</Text>
      {stakes.map((stake) => (
        <View key={stake.id} style={styles.stakeItem}>
          <View style={styles.stakeHeader}>
            <View style={styles.stakeInfo}>
              <Text style={styles.stakeAmount}>{stake.amount} COS</Text>
              <View style={styles.stakeMeta}>
                <Clock size={12} color="#888" />
                <Text style={styles.stakeMetaText}>{stake.lockPeriod}</Text>
                <Text style={styles.stakeApy}>{stake.apy}% APY</Text>
              </View>
            </View>
            <View style={[
              styles.statusBadge,
              stake.isUnlocked ? styles.statusUnlocked : styles.statusLocked,
            ]}>
              {stake.isUnlocked ? (
                <Unlock size={12} color="#4ECDC4" />
              ) : (
                <Lock size={12} color="#F7DC6F" />
              )}
              <Text style={[
                styles.statusText,
                stake.isUnlocked ? styles.statusTextUnlocked : styles.statusTextLocked,
              ]}>
                {stake.isUnlocked ? '已解锁' : '锁定中'}
              </Text>
            </View>
          </View>
          
          <View style={styles.stakeDates}>
            <Text style={styles.dateText}>开始: {stake.startDate}</Text>
            <Text style={styles.dateText}>解锁: {stake.unlockDate}</Text>
          </View>

          <View style={styles.rewardsRow}>
            <Text style={styles.rewardsLabel}>已赚取收益:</Text>
            <Text style={styles.rewardsValue}>{stake.earnedRewards} COS</Text>
          </View>

          <View style={styles.stakeActions}>
            <TouchableOpacity
              style={styles.claimButton}
              onPress={() => handleClaim(stake.id)}
            >
              <Text style={styles.claimButtonText}>领取收益</Text>
            </TouchableOpacity>
            {stake.isUnlocked && (
              <TouchableOpacity
                style={styles.unstakeButton}
                onPress={() => handleUnstake(stake.id)}
              >
                <Text style={styles.unstakeButtonText}>解除质押</Text>
              </TouchableOpacity>
            )}
          </View>
        </View>
      ))}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
    padding: 16,
  },
  overviewCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 20,
    marginBottom: 20,
  },
  overviewRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  overviewItem: {
    alignItems: 'center',
  },
  overviewValue: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    marginTop: 8,
  },
  overviewLabel: {
    fontSize: 12,
    color: '#888',
    marginTop: 4,
  },
  stakeCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 20,
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 16,
  },
  subTitle: {
    fontSize: 14,
    color: '#888',
    marginBottom: 12,
    marginTop: 16,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  input: {
    flex: 1,
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 16,
    fontSize: 18,
    color: '#fff',
  },
  maxButton: {
    backgroundColor: '#333',
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderRadius: 8,
    marginLeft: 8,
  },
  maxButtonText: {
    color: '#4ECDC4',
    fontWeight: 'bold',
  },
  balanceText: {
    fontSize: 12,
    color: '#888',
    marginTop: 8,
  },
  periodContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  periodOption: {
    width: '48%',
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 12,
    marginBottom: 8,
    marginRight: '2%',
    borderWidth: 2,
    borderColor: 'transparent',
  },
  periodOptionActive: {
    borderColor: '#4ECDC4',
  },
  periodLabel: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#888',
  },
  periodLabelActive: {
    color: '#fff',
  },
  periodApy: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  periodApyActive: {
    color: '#4ECDC4',
  },
  stakeButton: {
    backgroundColor: '#4ECDC4',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    marginTop: 16,
  },
  stakeButtonDisabled: {
    opacity: 0.6,
  },
  stakeButtonText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
  },
  stakeItem: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 16,
    marginBottom: 12,
  },
  stakeHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  stakeInfo: {},
  stakeAmount: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#fff',
  },
  stakeMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 4,
  },
  stakeMetaText: {
    fontSize: 12,
    color: '#888',
    marginLeft: 4,
    marginRight: 8,
  },
  stakeApy: {
    fontSize: 12,
    color: '#4ECDC4',
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 8,
  },
  statusLocked: {
    backgroundColor: '#F7DC6F20',
  },
  statusUnlocked: {
    backgroundColor: '#4ECDC420',
  },
  statusText: {
    fontSize: 12,
    marginLeft: 4,
  },
  statusTextLocked: {
    color: '#F7DC6F',
  },
  statusTextUnlocked: {
    color: '#4ECDC4',
  },
  stakeDates: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  dateText: {
    fontSize: 12,
    color: '#666',
  },
  rewardsRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  rewardsLabel: {
    fontSize: 14,
    color: '#888',
  },
  rewardsValue: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#4ECDC4',
  },
  stakeActions: {
    flexDirection: 'row',
  },
  claimButton: {
    flex: 1,
    backgroundColor: '#333',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    marginRight: 8,
  },
  claimButtonText: {
    color: '#4ECDC4',
    fontWeight: 'bold',
  },
  unstakeButton: {
    flex: 1,
    backgroundColor: '#FF6B6B20',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
  },
  unstakeButtonText: {
    color: '#FF6B6B',
    fontWeight: 'bold',
  },
});
