/**
 * 星尘玄鉴 - 转账页面
 * 发送代币到其他地址
 * 主题色：金棕色 #B2955D
 */

import { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  TextInput,
  ScrollView,
  Alert,
  KeyboardAvoidingView,
  Platform,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useWalletStore } from '@/stores';
import { BottomNavBar } from '@/components/BottomNavBar';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

export default function TransferPage() {
  const router = useRouter();
  const { address } = useWalletStore();

  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [memo, setMemo] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  // 模拟余额
  const balance = '0.00';

  const isValidAddress = (addr: string) => {
    // Substrate 地址以 5 开头，长度 48
    return addr.startsWith('5') && addr.length === 48;
  };

  const handleTransfer = async () => {
    // 验证收款地址
    if (!recipient) {
      Alert.alert('提示', '请输入收款地址');
      return;
    }
    if (!isValidAddress(recipient)) {
      Alert.alert('提示', '收款地址格式不正确');
      return;
    }
    if (recipient === address) {
      Alert.alert('提示', '不能转账给自己');
      return;
    }

    // 验证金额
    const amountNum = parseFloat(amount);
    if (!amount || isNaN(amountNum) || amountNum <= 0) {
      Alert.alert('提示', '请输入有效的转账金额');
      return;
    }
    if (amountNum > parseFloat(balance)) {
      Alert.alert('提示', '余额不足');
      return;
    }

    // 确认转账
    Alert.alert(
      '确认转账',
      `确定要向 ${recipient.slice(0, 8)}...${recipient.slice(-8)} 转账 ${amount} DUST 吗？`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: async () => {
            setIsLoading(true);
            try {
              // TODO: 实现实际转账逻辑
              await new Promise(resolve => setTimeout(resolve, 2000));
              Alert.alert('提示', '转账功能即将上线，敬请期待！');
            } catch (error) {
              Alert.alert('转账失败', '请稍后重试');
            } finally {
              setIsLoading(false);
            }
          },
        },
      ]
    );
  };

  const handleScanQR = () => {
    Alert.alert('提示', '二维码扫描功能即将上线');
  };

  const handleMaxAmount = () => {
    setAmount(balance);
  };

  return (
    <KeyboardAvoidingView
      style={styles.container}
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      {/* 顶部导航 */}
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>转账</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.scrollView} contentContainerStyle={styles.content} showsVerticalScrollIndicator={false}>
        {/* 余额显示 */}
        <View style={styles.balanceCard}>
          <Text style={styles.balanceLabel}>可用余额</Text>
          <Text style={styles.balanceAmount}>{balance} DUST</Text>
        </View>

        {/* 表单卡片 */}
        <View style={styles.formCard}>
          {/* 收款地址 */}
          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>收款地址</Text>
            <View style={styles.inputContainer}>
              <TextInput
                style={styles.input}
                placeholder="输入 Substrate 地址 (以 5 开头)"
                placeholderTextColor="#999"
                value={recipient}
                onChangeText={setRecipient}
                autoCapitalize="none"
                autoCorrect={false}
              />
              <Pressable onPress={handleScanQR} style={styles.scanButton}>
                <Ionicons name="scan-outline" size={20} color={THEME_COLOR} />
              </Pressable>
            </View>
          </View>

          {/* 转账金额 */}
          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>转账金额</Text>
            <View style={styles.inputContainer}>
              <TextInput
                style={styles.input}
                placeholder="0.00"
                placeholderTextColor="#999"
                value={amount}
                onChangeText={setAmount}
                keyboardType="decimal-pad"
              />
              <Pressable onPress={handleMaxAmount} style={styles.maxButton}>
                <Text style={styles.maxButtonText}>MAX</Text>
              </Pressable>
              <Text style={styles.currencyLabel}>DUST</Text>
            </View>
          </View>

          {/* 备注 */}
          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>备注（可选）</Text>
            <View style={styles.inputContainer}>
              <TextInput
                style={styles.input}
                placeholder="添加转账备注"
                placeholderTextColor="#999"
                value={memo}
                onChangeText={setMemo}
                maxLength={100}
              />
            </View>
          </View>

          {/* 手续费提示 */}
          <View style={styles.feeInfo}>
            <Ionicons name="information-circle-outline" size={16} color="#999" />
            <Text style={styles.feeText}>预估手续费: 0.001 DUST</Text>
          </View>
        </View>

        {/* 转账按钮 */}
        <Pressable
          style={[styles.primaryButton, isLoading && styles.disabledButton]}
          onPress={handleTransfer}
          disabled={isLoading}
        >
          {isLoading ? (
            <ActivityIndicator color="#FFF" />
          ) : (
            <>
              <Ionicons name="send" size={20} color="#FFF" />
              <Text style={styles.primaryButtonText}>确认转账</Text>
            </>
          )}
        </Pressable>

        {/* 安全提示 */}
        <View style={styles.tips}>
          <View style={styles.tipItem}>
            <Ionicons name="shield-checkmark-outline" size={18} color="#27AE60" />
            <Text style={styles.tipText}>请仔细核对收款地址</Text>
          </View>
          <View style={styles.tipItem}>
            <Ionicons name="alert-circle-outline" size={18} color="#F39C12" />
            <Text style={styles.tipText}>转账后无法撤销</Text>
          </View>
        </View>
      </ScrollView>

      {/* 底部导航栏 */}
      <BottomNavBar activeTab="profile" />
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#333',
  },
  placeholder: {
    width: 32,
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  balanceCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 20,
    marginBottom: 16,
    alignItems: 'center',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  balanceLabel: {
    fontSize: 14,
    color: '#999',
    marginBottom: 8,
  },
  balanceAmount: {
    fontSize: 32,
    fontWeight: 'bold',
    color: THEME_COLOR,
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    color: '#8B6914',
    marginBottom: 8,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#FAFAFA',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    paddingHorizontal: 14,
    height: 48,
    gap: 8,
  },
  input: {
    flex: 1,
    fontSize: 15,
    color: '#333',
  },
  scanButton: {
    padding: 6,
  },
  maxButton: {
    backgroundColor: THEME_COLOR + '20',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 4,
  },
  maxButtonText: {
    fontSize: 12,
    color: THEME_COLOR,
    fontWeight: '600',
  },
  currencyLabel: {
    fontSize: 14,
    color: '#999',
  },
  feeInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  feeText: {
    fontSize: 13,
    color: '#999',
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 16,
    borderRadius: 8,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 8,
    marginBottom: 20,
  },
  primaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: '#FFF',
  },
  disabledButton: {
    opacity: 0.6,
  },
  tips: {
    gap: 12,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  tipText: {
    fontSize: 13,
    color: '#666',
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: '50%',
    transform: [{ translateX: -207 }],
    width: 414,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingTop: 8,
    paddingBottom: 8,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  bottomNavItem: {
    alignItems: 'center',
    paddingVertical: 4,
    flex: 1,
  },
  bottomNavItemActive: {},
  bottomNavIcon: {
    fontSize: 22,
    marginBottom: 2,
  },
  bottomNavLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '500',
  },
  bottomNavLabelActive: {
    color: THEME_COLOR,
  },
});
