/**
 * 支付信息输入表单组件
 * 收集用户的支付信息和联系方式
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  Alert,
  ScrollView,
  KeyboardAvoidingView,
  Platform,
} from 'react-native';

export interface PaymentData {
  realName: string;
  idCard: string;
  phone: string;
  wechatId: string;
}

interface PaymentFormProps {
  onSubmit: (data: PaymentData) => void;
  onCancel?: () => void;
  initialData?: Partial<PaymentData>;
}

export const PaymentForm: React.FC<PaymentFormProps> = ({
  onSubmit,
  onCancel,
  initialData,
}) => {
  const [realName, setRealName] = useState(initialData?.realName || '');
  const [idCard, setIdCard] = useState(initialData?.idCard || '');
  const [phone, setPhone] = useState(initialData?.phone || '');
  const [wechatId, setWechatId] = useState(initialData?.wechatId || '');

  const validateForm = (): boolean => {
    if (!realName.trim()) {
      Alert.alert('提示', '请输入真实姓名');
      return false;
    }

    if (realName.trim().length < 2) {
      Alert.alert('提示', '姓名至少2个字符');
      return false;
    }

    if (!idCard.trim()) {
      Alert.alert('提示', '请输入身份证号');
      return false;
    }

    // 简单的身份证号验证
    const idCardRegex = /^[1-9]\d{5}(18|19|20)\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])\d{3}[\dXx]$/;
    if (!idCardRegex.test(idCard.trim())) {
      Alert.alert('提示', '请输入有效的身份证号');
      return false;
    }

    if (!phone.trim()) {
      Alert.alert('提示', '请输入手机号');
      return false;
    }

    // 手机号验证
    const phoneRegex = /^1[3-9]\d{9}$/;
    if (!phoneRegex.test(phone.trim())) {
      Alert.alert('提示', '请输入有效的手机号');
      return false;
    }

    if (!wechatId.trim()) {
      Alert.alert('提示', '请输入微信号');
      return false;
    }

    if (wechatId.trim().length < 6) {
      Alert.alert('提示', '微信号至少6个字符');
      return false;
    }

    return true;
  };

  const handleSubmit = () => {
    if (!validateForm()) {
      return;
    }

    const data: PaymentData = {
      realName: realName.trim(),
      idCard: idCard.trim(),
      phone: phone.trim(),
      wechatId: wechatId.trim(),
    };

    onSubmit(data);
  };

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      style={styles.container}
    >
      <ScrollView style={styles.scrollView}>
        <View style={styles.content}>
          {/* 标题 */}
          <View style={styles.header}>
            <Text style={styles.title}>支付信息</Text>
            <Text style={styles.subtitle}>
              请填写真实信息，用于生成支付承诺
            </Text>
          </View>

          {/* 真实姓名 */}
          <View style={styles.formGroup}>
            <Text style={styles.label}>真实姓名 *</Text>
            <TextInput
              style={styles.input}
              value={realName}
              onChangeText={setRealName}
              placeholder="请输入真实姓名"
              placeholderTextColor="#999999"
              autoCapitalize="words"
            />
          </View>

          {/* 身份证号 */}
          <View style={styles.formGroup}>
            <Text style={styles.label}>身份证号 *</Text>
            <TextInput
              style={styles.input}
              value={idCard}
              onChangeText={setIdCard}
              placeholder="请输入身份证号"
              placeholderTextColor="#999999"
              keyboardType="default"
              maxLength={18}
            />
          </View>

          {/* 手机号 */}
          <View style={styles.formGroup}>
            <Text style={styles.label}>手机号 *</Text>
            <TextInput
              style={styles.input}
              value={phone}
              onChangeText={setPhone}
              placeholder="请输入手机号"
              placeholderTextColor="#999999"
              keyboardType="phone-pad"
              maxLength={11}
            />
          </View>

          {/* 微信号 */}
          <View style={styles.formGroup}>
            <Text style={styles.label}>微信号 *</Text>
            <TextInput
              style={styles.input}
              value={wechatId}
              onChangeText={setWechatId}
              placeholder="请输入微信号"
              placeholderTextColor="#999999"
              autoCapitalize="none"
            />
          </View>

          {/* 隐私说明 */}
          <View style={styles.privacyNote}>
            <Text style={styles.privacyTitle}>🔒 隐私保护</Text>
            <Text style={styles.privacyText}>
              • 您的信息将使用 SHA256 加密
            </Text>
            <Text style={styles.privacyText}>
              • 链上只存储加密后的哈希值
            </Text>
            <Text style={styles.privacyText}>
              • 做市商无法获取您的真实信息
            </Text>
            <Text style={styles.privacyText}>
              • 仅在争议时用于身份验证
            </Text>
          </View>

          {/* 按钮 */}
          <View style={styles.buttonGroup}>
            <TouchableOpacity
              style={styles.submitButton}
              onPress={handleSubmit}
            >
              <Text style={styles.submitButtonText}>确认</Text>
            </TouchableOpacity>

            {onCancel && (
              <TouchableOpacity
                style={styles.cancelButton}
                onPress={onCancel}
              >
                <Text style={styles.cancelButtonText}>取消</Text>
              </TouchableOpacity>
            )}
          </View>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  header: {
    marginBottom: 24,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#666666',
  },
  formGroup: {
    marginBottom: 20,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 16,
    color: '#000000',
    borderWidth: 1,
    borderColor: '#E5E5EA',
  },
  privacyNote: {
    backgroundColor: '#F0F9FF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
    borderWidth: 1,
    borderColor: '#DBEAFE',
  },
  privacyTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  privacyText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
  buttonGroup: {
    gap: 12,
  },
  submitButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  cancelButton: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#E5E5EA',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
});
