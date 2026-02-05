import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { CheckSquare, Square } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { applyMaker } from '@/services/maker';
import { Button, Input, Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function MakerApplyScreen() {
  const colors = useColors();
  const router = useRouter();
  const { mnemonic } = useWalletStore();

  const [name, setName] = useState('');
  const [depositAmount, setDepositAmount] = useState('10000');
  const [paymentMethods, setPaymentMethods] = useState<string[]>(['bank']);
  const [bankName, setBankName] = useState('');
  const [bankAccount, setBankAccount] = useState('');
  const [bankHolder, setBankHolder] = useState('');
  const [alipayAccount, setAlipayAccount] = useState('');
  const [alipayName, setAlipayName] = useState('');
  const [wechatAccount, setWechatAccount] = useState('');
  const [wechatName, setWechatName] = useState('');
  const [agreed, setAgreed] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const togglePayment = (method: string) => {
    if (paymentMethods.includes(method)) {
      if (paymentMethods.length > 1) {
        setPaymentMethods(paymentMethods.filter((m) => m !== method));
      }
    } else {
      setPaymentMethods([...paymentMethods, method]);
    }
  };

  const handleSubmit = async () => {
    if (!mnemonic) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }
    if (!name.trim()) {
      Alert.alert('提示', '请输入商家名称');
      return;
    }
    if (parseFloat(depositAmount) < 10000) {
      Alert.alert('提示', '保证金至少 10,000 COS');
      return;
    }
    if (!agreed) {
      Alert.alert('提示', '请阅读并同意做市商协议');
      return;
    }

    const paymentInfo: any = {};
    if (paymentMethods.includes('bank')) {
      if (!bankName || !bankAccount || !bankHolder) {
        Alert.alert('提示', '请填写完整的银行卡信息');
        return;
      }
      paymentInfo.bank = { bankName, accountNumber: bankAccount, accountName: bankHolder };
    }
    if (paymentMethods.includes('alipay')) {
      if (!alipayAccount || !alipayName) {
        Alert.alert('提示', '请填写完整的支付宝信息');
        return;
      }
      paymentInfo.alipay = { account: alipayAccount, name: alipayName };
    }
    if (paymentMethods.includes('wechat')) {
      if (!wechatAccount || !wechatName) {
        Alert.alert('提示', '请填写完整的微信信息');
        return;
      }
      paymentInfo.wechat = { account: wechatAccount, name: wechatName };
    }

    setSubmitting(true);
    try {
      const amount = (BigInt(Math.floor(parseFloat(depositAmount))) * BigInt(1e12)).toString();
      await applyMaker(name, amount, paymentMethods, paymentInfo, mnemonic);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      Alert.alert('申请成功', '您的做市商申请已提交，请等待审核', [
        { text: '确定', onPress: () => router.replace('/maker') },
      ]);
    } catch (error: any) {
      Alert.alert('申请失败', error.message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        申请成为做市商
      </Text>

      {/* Basic Info */}
      <Card style={styles.card}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          基本信息
        </Text>
        <Input
          label="商家名称"
          placeholder="输入您的商家名称"
          value={name}
          onChangeText={setName}
          maxLength={20}
        />
        <Input
          label="保证金 (COS)"
          placeholder="最低 10,000 COS"
          value={depositAmount}
          onChangeText={setDepositAmount}
          keyboardType="decimal-pad"
        />
        <Text style={[styles.hint, { color: colors.textTertiary }]}>
          保证金将在审核通过后扣除，用于保障交易安全
        </Text>
      </Card>

      {/* Payment Methods */}
      <Card style={styles.card}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          收款方式
        </Text>
        <View style={styles.paymentOptions}>
          {[
            { id: 'bank', label: '银行卡', color: '#3B82F6' },
            { id: 'alipay', label: '支付宝', color: '#1677FF' },
            { id: 'wechat', label: '微信', color: '#07C160' },
          ].map((method) => {
            const isSelected = paymentMethods.includes(method.id);
            return (
              <TouchableOpacity
                key={method.id}
                style={[
                  styles.paymentOption,
                  {
                    borderColor: isSelected ? method.color : colors.border,
                    backgroundColor: isSelected ? method.color + '10' : 'transparent',
                  },
                ]}
                onPress={() => togglePayment(method.id)}
              >
                {isSelected ? (
                  <CheckSquare size={20} color={method.color} />
                ) : (
                  <Square size={20} color={colors.textTertiary} />
                )}
                <Text
                  style={[
                    styles.paymentLabel,
                    { color: isSelected ? method.color : colors.textPrimary },
                  ]}
                >
                  {method.label}
                </Text>
              </TouchableOpacity>
            );
          })}
        </View>
      </Card>

      {/* Bank Info */}
      {paymentMethods.includes('bank') && (
        <Card style={styles.card}>
          <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
            银行卡信息
          </Text>
          <Input
            label="银行名称"
            placeholder="如：中国工商银行"
            value={bankName}
            onChangeText={setBankName}
          />
          <Input
            label="银行卡号"
            placeholder="输入银行卡号"
            value={bankAccount}
            onChangeText={setBankAccount}
            keyboardType="number-pad"
          />
          <Input
            label="开户人姓名"
            placeholder="输入开户人姓名"
            value={bankHolder}
            onChangeText={setBankHolder}
          />
        </Card>
      )}

      {/* Alipay Info */}
      {paymentMethods.includes('alipay') && (
        <Card style={styles.card}>
          <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
            支付宝信息
          </Text>
          <Input
            label="支付宝账号"
            placeholder="手机号或邮箱"
            value={alipayAccount}
            onChangeText={setAlipayAccount}
          />
          <Input
            label="实名姓名"
            placeholder="输入支付宝实名姓名"
            value={alipayName}
            onChangeText={setAlipayName}
          />
        </Card>
      )}

      {/* WeChat Info */}
      {paymentMethods.includes('wechat') && (
        <Card style={styles.card}>
          <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
            微信信息
          </Text>
          <Input
            label="微信号"
            placeholder="输入微信号"
            value={wechatAccount}
            onChangeText={setWechatAccount}
          />
          <Input
            label="实名姓名"
            placeholder="输入微信实名姓名"
            value={wechatName}
            onChangeText={setWechatName}
          />
        </Card>
      )}

      {/* Agreement */}
      <TouchableOpacity
        style={styles.agreementRow}
        onPress={() => setAgreed(!agreed)}
      >
        {agreed ? (
          <CheckSquare size={20} color={Colors.primary} />
        ) : (
          <Square size={20} color={colors.textTertiary} />
        )}
        <Text style={[styles.agreementText, { color: colors.textSecondary }]}>
          我已阅读并同意{' '}
          <Text style={{ color: Colors.primary }}>《做市商服务协议》</Text>
        </Text>
      </TouchableOpacity>

      {/* Submit */}
      <Button
        title="提交申请"
        onPress={handleSubmit}
        loading={submitting}
        disabled={!agreed}
        style={styles.submitButton}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    marginBottom: 20,
  },
  card: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  hint: {
    fontSize: 12,
    marginTop: -8,
  },
  paymentOptions: {
    flexDirection: 'row',
    gap: 10,
  },
  paymentOption: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
    borderRadius: 12,
    borderWidth: 1.5,
    gap: 6,
  },
  paymentLabel: {
    fontSize: 14,
    fontWeight: '500',
  },
  agreementRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 24,
  },
  agreementText: {
    fontSize: 14,
    flex: 1,
  },
  submitButton: {},
});
