/**
 * 官方桥接页面
 * 通过治理账户进行 DUST → USDT 兑换
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { SwapAmountInput, TronAddressInput } from '@/features/bridge/components';
import { isWebEnvironment, isSignerUnlocked, getCurrentSignerAddress, unlockWalletForSigning } from '@/lib/signer';
import { bridgeService } from '@/services/bridge.service';

const MIN_AMOUNT = 10;
const DUST_DECIMALS = 12; // DUST token decimals

export default function OfficialBridgePage() {
  const router = useRouter();
  const [dustAmount, setDustAmount] = useState('');
  const [tronAddress, setTronAddress] = useState('');
  const [dustPrice, setDustPrice] = useState(0.10);
  const [balance, setBalance] = useState('1000');
  const [loading, setLoading] = useState(false);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      // 获取 DUST 价格
      const price = await bridgeService.getDustPrice();
      setDustPrice(price);

      // 获取用户余额
      const address = getCurrentSignerAddress();
      if (address) {
        const balanceBigInt = await bridgeService.getDustBalance(address);
        const balanceFormatted = (Number(balanceBigInt) / Math.pow(10, DUST_DECIMALS)).toFixed(2);
        setBalance(balanceFormatted);
      }
    } catch (error) {
      console.error('加载数据失败:', error);
      // 使用默认值
      setDustPrice(0.10);
      setBalance('1000');
    }
  };

  const validateForm = (): boolean => {
    const amount = parseFloat(dustAmount);
    if (isNaN(amount) || amount < MIN_AMOUNT) {
      Alert.alert('提示', `最小兑换金额为 ${MIN_AMOUNT} DUST`);
      return false;
    }

    if (amount > parseFloat(balance)) {
      Alert.alert('提示', 'DUST 余额不足');
      return false;
    }

    // 验证 TRON 地址
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    if (!tronRegex.test(tronAddress)) {
      Alert.alert('提示', '请输入有效的 TRON 地址');
      return false;
    }

    return true;
  };

  const handleSwap = async () => {
    if (!validateForm()) return;

    // 检查是否需要解锁钱包
    if (!isWebEnvironment() && !isSignerUnlocked()) {
      setShowUnlockDialog(true);
      return;
    }

    await executeSwap();
  };

  const handleWalletUnlocked = async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      await executeSwap();
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  const executeSwap = async () => {
    try {
      setShowTxStatus(true);
      setTxStatus('正在创建兑换请求...');

      // 将 DUST 数量转换为最小单位（BigInt）
      const amount = parseFloat(dustAmount);
      const dustAmountBigInt = BigInt(Math.floor(amount * Math.pow(10, DUST_DECIMALS)));

      // 调用链上 bridge.officialSwap() 方法
      const swapId = await bridgeService.officialSwap(
        dustAmountBigInt,
        tronAddress,
        (status) => {
          setTxStatus(status);
        }
      );

      setTxStatus('兑换成功！');

      setTimeout(() => {
        setShowTxStatus(false);

        Alert.alert(
          '成功',
          `兑换请求已创建，请等待治理账户处理\n兑换ID: ${swapId}`,
          [
            {
              text: '查看记录',
              onPress: () => router.push('/bridge/history' as any),
            },
            {
              text: '确定',
              style: 'cancel',
            },
          ]
        );
      }, 1500);
    } catch (error: any) {
      console.error('兑换失败:', error);
      setTxStatus('兑换失败');
      setTimeout(() => {
        setShowTxStatus(false);
        const errorMessage = error.message || '创建兑换失败';
        Alert.alert('错误', errorMessage);
      }, 1500);
    }
  };

  const usdtEstimate = (parseFloat(dustAmount) || 0) * dustPrice;

  return (
    <View style={styles.wrapper}>
      <PageHeader title="官方桥接" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 说明卡片 */}
        <View style={styles.section}>
          <View style={styles.infoCard}>
            <Text style={styles.infoTitle}>🏛️ 官方桥接</Text>
            <Text style={styles.infoText}>
              由治理账户处理，无溢价，安全可靠。
              通常在 24 小时内完成转账。
            </Text>
          </View>
        </View>

        {/* 金额输入 */}
        <View style={styles.section}>
          <SwapAmountInput
            value={dustAmount}
            onChangeText={setDustAmount}
            dustPrice={dustPrice}
            balance={balance}
            minAmount={MIN_AMOUNT}
          />
        </View>

        {/* TRON 地址输入 */}
        <View style={styles.section}>
          <TronAddressInput
            value={tronAddress}
            onChangeText={setTronAddress}
          />
        </View>

        {/* 兑换详情 */}
        <View style={styles.section}>
          <View style={styles.detailCard}>
            <Text style={styles.detailTitle}>兑换详情</Text>
            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>支付</Text>
              <Text style={styles.detailValue}>
                {dustAmount || '0'} DUST
              </Text>
            </View>
            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>汇率</Text>
              <Text style={styles.detailValue}>
                1 DUST = {dustPrice.toFixed(4)} USDT
              </Text>
            </View>
            <View style={styles.detailRow}>
              <Text style={styles.detailLabel}>手续费</Text>
              <Text style={styles.detailValue}>0 USDT</Text>
            </View>
            <View style={styles.divider} />
            <View style={styles.detailRow}>
              <Text style={styles.detailLabelBold}>预计获得</Text>
              <Text style={styles.detailValueGreen}>
                ≈ {usdtEstimate.toFixed(2)} USDT
              </Text>
            </View>
          </View>
        </View>

        {/* 提交按钮 */}
        <View style={styles.section}>
          <TouchableOpacity
            style={[
              styles.submitButton,
              (!dustAmount || !tronAddress) && styles.submitButtonDisabled,
            ]}
            onPress={handleSwap}
            disabled={!dustAmount || !tronAddress || loading}
          >
            {loading ? (
              <ActivityIndicator color="#FFFFFF" />
            ) : (
              <Text style={styles.submitButtonText}>确认兑换</Text>
            )}
          </TouchableOpacity>
        </View>

        {/* 注意事项 */}
        <View style={styles.section}>
          <Text style={styles.noticeTitle}>⚠️ 注意事项</Text>
          <Text style={styles.noticeText}>• 兑换请求提交后，DUST 将被锁定</Text>
          <Text style={styles.noticeText}>• 治理账户将在 24 小时内处理</Text>
          <Text style={styles.noticeText}>• 超时未处理将自动退款</Text>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />

      {/* 解锁钱包对话框 */}
      <UnlockWalletDialog
        visible={showUnlockDialog}
        onUnlock={handleWalletUnlocked}
        onCancel={() => setShowUnlockDialog(false)}
      />

      {/* 交易状态对话框 */}
      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        title="创建兑换中"
      />
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 20,
  },
  section: {
    padding: 16,
  },
  infoCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#B2955D',
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  detailCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
  },
  detailTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  detailRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  detailLabel: {
    fontSize: 14,
    color: '#666666',
  },
  detailLabelBold: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
  },
  detailValue: {
    fontSize: 14,
    color: '#000000',
  },
  detailValueGreen: {
    fontSize: 16,
    fontWeight: '600',
    color: '#4CD964',
  },
  divider: {
    height: 1,
    backgroundColor: '#F0F0F0',
    marginVertical: 8,
  },
  submitButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  submitButtonDisabled: {
    backgroundColor: '#CCCCCC',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  noticeTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  noticeText: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
});
