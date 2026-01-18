/**
 * 申请仲裁对话框
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Modal,
  TouchableOpacity,
  TextInput,
  Alert,
  ActivityIndicator,
  ScrollView,
} from 'react-native';
import * as ImagePicker from 'expo-image-picker';
import type { Order } from '@/stores/trading.store';

interface DisputeDialogProps {
  visible: boolean;
  order: Order | null;
  onSubmit: (reason: string, evidenceUri?: string) => Promise<void>;
  onClose: () => void;
}

// 仲裁原因选项
const DISPUTE_REASONS = [
  { id: 'not_released', label: '已付款但未收到 DUST' },
  { id: 'wrong_amount', label: '收到的 DUST 数量不对' },
  { id: 'maker_unresponsive', label: '做市商长时间不响应' },
  { id: 'other', label: '其他原因' },
];

export const DisputeDialog: React.FC<DisputeDialogProps> = ({
  visible,
  order,
  onSubmit,
  onClose,
}) => {
  const [selectedReason, setSelectedReason] = useState<string>('');
  const [customReason, setCustomReason] = useState('');
  const [evidenceUri, setEvidenceUri] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  if (!order) return null;

  const handleSelectEvidence = async () => {
    try {
      // 请求权限
      const { status } = await ImagePicker.requestMediaLibraryPermissionsAsync();
      if (status !== 'granted') {
        Alert.alert('提示', '需要相册权限才能上传证据');
        return;
      }

      // 选择图片
      const result = await ImagePicker.launchImageLibraryAsync({
        mediaTypes: ImagePicker.MediaTypeOptions.Images,
        allowsEditing: false,
        quality: 0.8,
      });

      if (!result.canceled && result.assets[0]) {
        setEvidenceUri(result.assets[0].uri);
      }
    } catch (error) {
      console.error('Select evidence error:', error);
      Alert.alert('错误', '选择图片失败');
    }
  };

  const handleSubmit = async () => {
    if (!selectedReason) {
      Alert.alert('提示', '请选择仲裁原因');
      return;
    }

    if (selectedReason === 'other' && !customReason.trim()) {
      Alert.alert('提示', '请填写具体原因');
      return;
    }

    const reason = selectedReason === 'other'
      ? customReason
      : DISPUTE_REASONS.find(r => r.id === selectedReason)?.label || '';

    try {
      setSubmitting(true);
      await onSubmit(reason, evidenceUri || undefined);

      // 重置状态
      setSelectedReason('');
      setCustomReason('');
      setEvidenceUri(null);

      Alert.alert('成功', '仲裁申请已提交，平台将在 24 小时内处理');
      onClose();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '提交失败，请重试';
      Alert.alert('错误', errorMessage);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <View style={styles.container}>
          <ScrollView showsVerticalScrollIndicator={false}>
            {/* 标题 */}
            <View style={styles.header}>
              <Text style={styles.title}>申请仲裁</Text>
              <TouchableOpacity onPress={onClose}>
                <Text style={styles.closeIcon}>✕</Text>
              </TouchableOpacity>
            </View>

            {/* 订单信息 */}
            <View style={styles.orderInfo}>
              <View style={styles.orderRow}>
                <Text style={styles.orderLabel}>订单号</Text>
                <Text style={styles.orderValue}>#{order.id}</Text>
              </View>
              <View style={styles.orderRow}>
                <Text style={styles.orderLabel}>金额</Text>
                <Text style={styles.orderValue}>
                  {(Number(order.amount) / 1e6).toFixed(2)} USDT
                </Text>
              </View>
            </View>

            {/* 仲裁原因 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>仲裁原因</Text>
              {DISPUTE_REASONS.map((reason) => (
                <TouchableOpacity
                  key={reason.id}
                  style={[
                    styles.reasonItem,
                    selectedReason === reason.id && styles.reasonItemSelected,
                  ]}
                  onPress={() => setSelectedReason(reason.id)}
                >
                  <View style={[
                    styles.radioOuter,
                    selectedReason === reason.id && styles.radioOuterSelected,
                  ]}>
                    {selectedReason === reason.id && (
                      <View style={styles.radioInner} />
                    )}
                  </View>
                  <Text style={[
                    styles.reasonText,
                    selectedReason === reason.id && styles.reasonTextSelected,
                  ]}>
                    {reason.label}
                  </Text>
                </TouchableOpacity>
              ))}

              {/* 自定义原因输入 */}
              {selectedReason === 'other' && (
                <TextInput
                  style={styles.customInput}
                  placeholder="请描述具体原因..."
                  placeholderTextColor="#999999"
                  value={customReason}
                  onChangeText={setCustomReason}
                  multiline
                  numberOfLines={3}
                  textAlignVertical="top"
                />
              )}
            </View>

            {/* 上传证据 */}
            <View style={styles.section}>
              <Text style={styles.sectionTitle}>上传证据（可选）</Text>
              <Text style={styles.sectionDesc}>
                上传付款截图或聊天记录可加快处理速度
              </Text>

              <TouchableOpacity
                style={styles.uploadButton}
                onPress={handleSelectEvidence}
              >
                {evidenceUri ? (
                  <View style={styles.evidencePreview}>
                    <Text style={styles.evidenceText}>✓ 已选择图片</Text>
                    <TouchableOpacity onPress={() => setEvidenceUri(null)}>
                      <Text style={styles.removeText}>移除</Text>
                    </TouchableOpacity>
                  </View>
                ) : (
                  <Text style={styles.uploadText}>+ 选择图片</Text>
                )}
              </TouchableOpacity>
            </View>

            {/* 提示信息 */}
            <View style={styles.tipCard}>
              <Text style={styles.tipTitle}>⚠️ 仲裁须知</Text>
              <Text style={styles.tipText}>• 仲裁期间订单将被冻结</Text>
              <Text style={styles.tipText}>• 平台将在 24 小时内介入处理</Text>
              <Text style={styles.tipText}>• 恶意仲裁将影响信用评分</Text>
              <Text style={styles.tipText}>• 证据窗口期为订单完成后 24 小时</Text>
            </View>

            {/* 提交按钮 */}
            <TouchableOpacity
              style={[styles.submitButton, submitting && styles.submitButtonDisabled]}
              onPress={handleSubmit}
              disabled={submitting}
            >
              {submitting ? (
                <ActivityIndicator color="#FFFFFF" />
              ) : (
                <Text style={styles.submitButtonText}>提交仲裁申请</Text>
              )}
            </TouchableOpacity>

            <TouchableOpacity
              style={styles.cancelButton}
              onPress={onClose}
              disabled={submitting}
            >
              <Text style={styles.cancelButtonText}>取消</Text>
            </TouchableOpacity>
          </ScrollView>
        </View>
      </View>
    </Modal>
  );
};

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'flex-end',
  },
  container: {
    backgroundColor: '#FFFFFF',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    padding: 20,
    maxHeight: '90%',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 20,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
  },
  closeIcon: {
    fontSize: 20,
    color: '#666666',
    padding: 4,
  },
  orderInfo: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 20,
  },
  orderRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  orderLabel: {
    fontSize: 14,
    color: '#666666',
  },
  orderValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#000000',
  },
  section: {
    marginBottom: 20,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  sectionDesc: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 12,
  },
  reasonItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    paddingHorizontal: 16,
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    marginBottom: 8,
  },
  reasonItemSelected: {
    backgroundColor: '#FFF9F0',
    borderWidth: 1,
    borderColor: '#B2955D',
  },
  radioOuter: {
    width: 20,
    height: 20,
    borderRadius: 10,
    borderWidth: 2,
    borderColor: '#CCCCCC',
    marginRight: 12,
    justifyContent: 'center',
    alignItems: 'center',
  },
  radioOuterSelected: {
    borderColor: '#B2955D',
  },
  radioInner: {
    width: 10,
    height: 10,
    borderRadius: 5,
    backgroundColor: '#B2955D',
  },
  reasonText: {
    fontSize: 14,
    color: '#333333',
  },
  reasonTextSelected: {
    color: '#000000',
    fontWeight: '500',
  },
  customInput: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: '#000000',
    minHeight: 80,
    marginTop: 8,
  },
  uploadButton: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#E5E5EA',
    borderStyle: 'dashed',
  },
  uploadText: {
    fontSize: 14,
    color: '#666666',
  },
  evidencePreview: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  evidenceText: {
    fontSize: 14,
    color: '#4CD964',
    marginRight: 12,
  },
  removeText: {
    fontSize: 14,
    color: '#FF3B30',
  },
  tipCard: {
    backgroundColor: '#FFF3F3',
    borderRadius: 12,
    padding: 16,
    marginBottom: 20,
  },
  tipTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FF3B30',
    marginBottom: 8,
  },
  tipText: {
    fontSize: 13,
    color: '#666666',
    marginBottom: 4,
  },
  submitButton: {
    backgroundColor: '#FF3B30',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 12,
  },
  submitButtonDisabled: {
    backgroundColor: '#FFAAAA',
  },
  submitButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  cancelButton: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 20,
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
});
