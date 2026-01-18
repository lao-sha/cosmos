/**
 * TRON 地址输入组件
 * 支持地址验证和粘贴
 */

import React, { useState } from 'react';
import { View, Text, TextInput, StyleSheet, TouchableOpacity, Alert } from 'react-native';
import * as Clipboard from 'expo-clipboard';

interface TronAddressInputProps {
  value: string;
  onChangeText: (text: string) => void;
  label?: string;
  placeholder?: string;
}

export const TronAddressInput: React.FC<TronAddressInputProps> = ({
  value,
  onChangeText,
  label = 'USDT 收款地址 (TRC20)',
  placeholder = '请输入 TRON 地址 (T开头)',
}) => {
  const [isValid, setIsValid] = useState<boolean | null>(null);

  // 验证 TRON 地址格式
  const validateTronAddress = (address: string): boolean => {
    // TRON 地址以 T 开头，长度为 34 字符
    const tronRegex = /^T[A-Za-z1-9]{33}$/;
    return tronRegex.test(address);
  };

  const handleChangeText = (text: string) => {
    onChangeText(text);
    if (text.length > 0) {
      setIsValid(validateTronAddress(text));
    } else {
      setIsValid(null);
    }
  };

  const handlePaste = async () => {
    try {
      const text = await Clipboard.getStringAsync();
      if (text) {
        handleChangeText(text.trim());
      }
    } catch (error) {
      Alert.alert('错误', '无法读取剪贴板');
    }
  };

  return (
    <View style={styles.container}>
      <Text style={styles.label}>{label}</Text>

      <View style={[
        styles.inputContainer,
        isValid === true && styles.inputValid,
        isValid === false && styles.inputInvalid,
      ]}>
        <TextInput
          style={styles.input}
          value={value}
          onChangeText={handleChangeText}
          placeholder={placeholder}
          placeholderTextColor="#999999"
          autoCapitalize="none"
          autoCorrect={false}
        />
        <TouchableOpacity style={styles.pasteButton} onPress={handlePaste}>
          <Text style={styles.pasteText}>粘贴</Text>
        </TouchableOpacity>
      </View>

      {isValid === false && (
        <Text style={styles.errorText}>
          ⚠️ 无效的 TRON 地址格式
        </Text>
      )}

      {isValid === true && (
        <Text style={styles.validText}>
          ✓ 地址格式正确
        </Text>
      )}

      <Text style={styles.hint}>
        请确保地址正确，转账后无法撤回
      </Text>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000000',
    marginBottom: 8,
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    paddingHorizontal: 12,
    borderWidth: 1,
    borderColor: '#E5E5EA',
  },
  inputValid: {
    borderColor: '#4CD964',
    backgroundColor: '#F0FFF4',
  },
  inputInvalid: {
    borderColor: '#FF3B30',
    backgroundColor: '#FFF0F0',
  },
  input: {
    flex: 1,
    fontSize: 14,
    color: '#000000',
    paddingVertical: 12,
  },
  pasteButton: {
    backgroundColor: '#E5E5EA',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 4,
  },
  pasteText: {
    fontSize: 12,
    fontWeight: '500',
    color: '#666666',
  },
  errorText: {
    fontSize: 12,
    color: '#FF3B30',
    marginTop: 4,
  },
  validText: {
    fontSize: 12,
    color: '#4CD964',
    marginTop: 4,
  },
  hint: {
    fontSize: 12,
    color: '#999999',
    marginTop: 4,
  },
});
