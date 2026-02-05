import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Alert,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Scale, AlertTriangle } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { createDispute } from '@/services/dispute';
import { Button, Input, Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

export default function CreateDisputeScreen() {
  const colors = useColors();
  const router = useRouter();
  const { orderId } = useLocalSearchParams<{ orderId?: string }>();
  const { mnemonic } = useWalletStore();

  const [domain, setDomain] = useState('otc');
  const [bizId, setBizId] = useState(orderId || '');
  const [description, setDescription] = useState('');
  const [deposit, setDeposit] = useState('100');
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!mnemonic) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }
    if (!bizId.trim()) {
      Alert.alert('提示', '请输入订单/业务ID');
      return;
    }
    if (!description.trim()) {
      Alert.alert('提示', '请描述争议原因');
      return;
    }
    if (parseFloat(deposit) < 100) {
      Alert.alert('提示', '押金至少 100 COS');
      return;
    }

    Alert.alert(
      '确认提交',
      `您将支付 ${deposit} COS 作为押金参与仲裁，如胜诉将返还押金并获得对方押金赔偿`,
      [
        { text: '取消', style: 'cancel' },
        {
          text: '确认',
          onPress: async () => {
            setSubmitting(true);
            try {
              const depositAmount = (BigInt(Math.floor(parseFloat(deposit))) * BigInt(1e12)).toString();
              const disputeId = await createDispute(
                domain,
                bizId.trim(),
                description.trim(),
                depositAmount,
                mnemonic
              );
              Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
              Alert.alert('提交成功', '争议已创建，等待对方响应', [
                { text: '查看详情', onPress: () => router.replace(`/disputes/${disputeId}`) },
              ]);
            } catch (error: any) {
              Alert.alert('提交失败', error.message);
            } finally {
              setSubmitting(false);
            }
          },
        },
      ]
    );
  };

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <View style={styles.iconContainer}>
        <Scale size={48} color={Colors.primary} />
      </View>
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        发起争议
      </Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        请确保您有充分的证据支持您的诉求
      </Text>

      {/* Warning */}
      <Card style={[styles.warningCard, { backgroundColor: Colors.warning + '10' }]}>
        <AlertTriangle size={20} color={Colors.warning} />
        <View style={styles.warningContent}>
          <Text style={[styles.warningTitle, { color: Colors.warning }]}>
            重要提示
          </Text>
          <Text style={[styles.warningText, { color: colors.textSecondary }]}>
            • 发起争议需要支付押金{'\n'}
            • 胜诉方将获得对方押金赔偿{'\n'}
            • 虚假申诉将扣除押金并影响信用分{'\n'}
            • 请先尝试与对方协商解决
          </Text>
        </View>
      </Card>

      {/* Form */}
      <Card style={styles.formCard}>
        <Input
          label="争议类型"
          value={domain === 'otc' ? 'OTC 交易' : domain}
          editable={false}
        />
        <Input
          label="订单/业务ID"
          placeholder="输入相关订单号"
          value={bizId}
          onChangeText={setBizId}
          autoCapitalize="none"
        />
        <Input
          label="争议描述"
          placeholder="详细描述争议原因和诉求"
          value={description}
          onChangeText={setDescription}
          multiline
          numberOfLines={4}
          maxLength={500}
        />
        <Text style={[styles.charCount, { color: colors.textTertiary }]}>
          {description.length}/500
        </Text>
        <Input
          label="押金金额 (COS)"
          placeholder="最低 100 COS"
          value={deposit}
          onChangeText={setDeposit}
          keyboardType="decimal-pad"
        />
        <Text style={[styles.hint, { color: colors.textTertiary }]}>
          押金金额越高，仲裁员会优先处理
        </Text>
      </Card>

      {/* Submit */}
      <Button
        title="提交争议"
        onPress={handleSubmit}
        loading={submitting}
        disabled={!bizId.trim() || !description.trim() || parseFloat(deposit) < 100}
        style={styles.submitButton}
      />

      {/* Process */}
      <Card style={styles.processCard}>
        <Text style={[styles.processTitle, { color: colors.textPrimary }]}>
          仲裁流程
        </Text>
        <View style={styles.processItem}>
          <View style={[styles.processStep, { backgroundColor: Colors.primary }]}>
            <Text style={styles.processStepText}>1</Text>
          </View>
          <View style={styles.processContent}>
            <Text style={[styles.processLabel, { color: colors.textPrimary }]}>
              发起争议
            </Text>
            <Text style={[styles.processDesc, { color: colors.textTertiary }]}>
              支付押金，等待对方响应
            </Text>
          </View>
        </View>
        <View style={styles.processItem}>
          <View style={[styles.processStep, { backgroundColor: Colors.primary }]}>
            <Text style={styles.processStepText}>2</Text>
          </View>
          <View style={styles.processContent}>
            <Text style={[styles.processLabel, { color: colors.textPrimary }]}>
              举证阶段
            </Text>
            <Text style={[styles.processDesc, { color: colors.textTertiary }]}>
              双方提交证据材料 (72小时)
            </Text>
          </View>
        </View>
        <View style={styles.processItem}>
          <View style={[styles.processStep, { backgroundColor: Colors.primary }]}>
            <Text style={styles.processStepText}>3</Text>
          </View>
          <View style={styles.processContent}>
            <Text style={[styles.processLabel, { color: colors.textPrimary }]}>
              仲裁裁决
            </Text>
            <Text style={[styles.processDesc, { color: colors.textTertiary }]}>
              仲裁员审核证据并作出裁决
            </Text>
          </View>
        </View>
        <View style={styles.processItem}>
          <View style={[styles.processStep, { backgroundColor: Colors.primary }]}>
            <Text style={styles.processStepText}>4</Text>
          </View>
          <View style={styles.processContent}>
            <Text style={[styles.processLabel, { color: colors.textPrimary }]}>
              执行结果
            </Text>
            <Text style={[styles.processDesc, { color: colors.textTertiary }]}>
              自动执行裁决，分配押金
            </Text>
          </View>
        </View>
      </Card>
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
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: 'rgba(99, 102, 241, 0.1)',
    alignItems: 'center',
    justifyContent: 'center',
    alignSelf: 'center',
    marginTop: 10,
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    textAlign: 'center',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    textAlign: 'center',
    marginBottom: 20,
  },
  warningCard: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    marginBottom: 16,
    borderWidth: 0,
  },
  warningContent: {
    flex: 1,
    marginLeft: 12,
  },
  warningTitle: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 6,
  },
  warningText: {
    fontSize: 13,
    lineHeight: 20,
  },
  formCard: {
    marginBottom: 20,
  },
  charCount: {
    fontSize: 12,
    textAlign: 'right',
    marginTop: -12,
    marginBottom: 12,
  },
  hint: {
    fontSize: 12,
    marginTop: -8,
  },
  submitButton: {},
  processCard: {
    marginTop: 20,
  },
  processTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 16,
  },
  processItem: {
    flexDirection: 'row',
    marginBottom: 16,
  },
  processStep: {
    width: 28,
    height: 28,
    borderRadius: 14,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  processStepText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '600',
  },
  processContent: {
    flex: 1,
  },
  processLabel: {
    fontSize: 14,
    fontWeight: '500',
  },
  processDesc: {
    fontSize: 12,
    marginTop: 2,
  },
});
