/**
 * 创建套餐页面
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TextInput,
  Pressable,
  Switch,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Picker } from '@react-native-picker/picker';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import {
  DivinationType,
  ServiceType,
  DIVINATION_TYPE_CONFIG,
  SERVICE_TYPE_CONFIG,
} from '@/features/diviner';

const THEME_COLOR = '#B2955D';

export default function CreatePackagePage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  // 表单状态
  const [divinationType, setDivinationType] = useState<DivinationType>(DivinationType.Meihua);
  const [serviceType, setServiceType] = useState<ServiceType>(ServiceType.TextReading);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [price, setPrice] = useState('');
  const [duration, setDuration] = useState('0');
  const [followUpCount, setFollowUpCount] = useState('3');
  const [urgentAvailable, setUrgentAvailable] = useState(false);
  const [urgentSurcharge, setUrgentSurcharge] = useState('50');

  // 验证
  const nameValid = name.length >= 1 && name.length <= 64;
  const descValid = description.length >= 1 && description.length <= 1024;
  const priceValid = parseFloat(price) >= 1;
  const formValid = nameValid && descValid && priceValid;

  const handleSubmit = async () => {
    if (!formValid) {
      Alert.alert('提示', '请完整填写所有必填项');
      return;
    }

    setLoading(true);
    try {
      // TODO: 调用链上创建套餐方法
      await new Promise(resolve => setTimeout(resolve, 1500));

      Alert.alert('创建成功', '套餐已创建', [
        { text: '确定', onPress: () => router.back() },
      ]);
    } catch (error: any) {
      Alert.alert('创建失败', error.message || '请稍后重试');
    } finally {
      setLoading(false);
    }
  };

  return (
    <View style={styles.wrapper}>
      <PageHeader title="创建套餐" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 基本信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>

          <View style={styles.formCard}>
            <View style={styles.formItem}>
              <Text style={styles.label}>占卜类型</Text>
              <View style={styles.pickerWrapper}>
                <Picker
                  selectedValue={divinationType}
                  onValueChange={setDivinationType}
                  style={styles.picker}
                >
                  {Object.entries(DIVINATION_TYPE_CONFIG).map(([key, config]) => (
                    <Picker.Item key={key} label={`${config.icon} ${config.label}`} value={Number(key)} />
                  ))}
                </Picker>
              </View>
            </View>

            <View style={styles.formItem}>
              <Text style={styles.label}>服务类型</Text>
              <View style={styles.pickerWrapper}>
                <Picker
                  selectedValue={serviceType}
                  onValueChange={setServiceType}
                  style={styles.picker}
                >
                  {Object.entries(SERVICE_TYPE_CONFIG).map(([key, config]) => (
                    <Picker.Item key={key} label={`${config.icon} ${config.label}`} value={Number(key)} />
                  ))}
                </Picker>
              </View>
            </View>

            <View style={styles.formItem}>
              <Text style={styles.label}>
                套餐名称 <Text style={styles.required}>*</Text>
              </Text>
              <TextInput
                style={styles.input}
                value={name}
                onChangeText={setName}
                placeholder="例如：梅花易数·详细解读"
                placeholderTextColor="#999"
                maxLength={64}
              />
            </View>

            <View style={styles.formItem}>
              <Text style={styles.label}>
                套餐描述 <Text style={styles.required}>*</Text>
              </Text>
              <TextInput
                style={styles.textArea}
                value={description}
                onChangeText={setDescription}
                placeholder="详细描述您的服务内容..."
                placeholderTextColor="#999"
                multiline
                numberOfLines={4}
                maxLength={1024}
                textAlignVertical="top"
              />
              <Text style={styles.charCount}>{description.length}/1024</Text>
            </View>
          </View>
        </View>

        {/* 价格设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>价格设置</Text>

          <View style={styles.formCard}>
            <View style={styles.formItem}>
              <Text style={styles.label}>
                服务价格 <Text style={styles.required}>*</Text>
              </Text>
              <View style={styles.priceRow}>
                <TextInput
                  style={[styles.input, styles.priceInput]}
                  value={price}
                  onChangeText={setPrice}
                  placeholder="最低 1"
                  placeholderTextColor="#999"
                  keyboardType="decimal-pad"
                />
                <Text style={styles.priceUnit}>DUST</Text>
              </View>
            </View>

            <View style={styles.formItem}>
              <Text style={styles.label}>追问次数</Text>
              <View style={styles.pickerWrapper}>
                <Picker
                  selectedValue={followUpCount}
                  onValueChange={setFollowUpCount}
                  style={styles.picker}
                >
                  {[0, 1, 2, 3, 4, 5].map(n => (
                    <Picker.Item key={n} label={`${n} 次`} value={String(n)} />
                  ))}
                </Picker>
              </View>
            </View>

            {(serviceType === ServiceType.VoiceReading || serviceType === ServiceType.VideoReading) && (
              <View style={styles.formItem}>
                <Text style={styles.label}>服务时长（分钟）</Text>
                <TextInput
                  style={styles.input}
                  value={duration}
                  onChangeText={setDuration}
                  placeholder="0 表示不限时"
                  placeholderTextColor="#999"
                  keyboardType="number-pad"
                />
              </View>
            )}
          </View>
        </View>

        {/* 加急设置 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>加急设置</Text>

          <View style={styles.formCard}>
            <View style={styles.switchRow}>
              <View style={styles.switchLabel}>
                <Text style={styles.label}>支持加急服务</Text>
                <Text style={styles.switchHint}>开启后用户可选择加急</Text>
              </View>
              <Switch
                value={urgentAvailable}
                onValueChange={setUrgentAvailable}
                trackColor={{ false: '#E8E8E8', true: `${THEME_COLOR}80` }}
                thumbColor={urgentAvailable ? THEME_COLOR : '#FFF'}
              />
            </View>

            {urgentAvailable && (
              <View style={styles.formItem}>
                <Text style={styles.label}>加急加价比例</Text>
                <View style={styles.priceRow}>
                  <TextInput
                    style={[styles.input, styles.priceInput]}
                    value={urgentSurcharge}
                    onChangeText={setUrgentSurcharge}
                    placeholder="50"
                    placeholderTextColor="#999"
                    keyboardType="number-pad"
                  />
                  <Text style={styles.priceUnit}>%</Text>
                </View>
              </View>
            )}
          </View>
        </View>

        {/* 提交按钮 */}
        <View style={styles.actionSection}>
          <Pressable
            style={[styles.submitBtn, !formValid && styles.submitBtnDisabled]}
            onPress={handleSubmit}
            disabled={!formValid || loading}
          >
            {loading ? (
              <ActivityIndicator color="#FFF" />
            ) : (
              <Text style={styles.submitBtnText}>创建套餐</Text>
            )}
          </Pressable>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />
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
    paddingBottom: 100,
  },
  section: {
    paddingHorizontal: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000',
    marginBottom: 12,
    marginTop: 16,
  },
  formCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
  },
  formItem: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    color: '#333',
    marginBottom: 8,
    fontWeight: '500',
  },
  required: {
    color: '#FF3B30',
  },
  input: {
    height: 44,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  textArea: {
    height: 100,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#FAFAFA',
  },
  charCount: {
    fontSize: 12,
    color: '#999',
    textAlign: 'right',
    marginTop: 4,
  },
  pickerWrapper: {
    borderWidth: 1,
    borderColor: '#E8E8E8',
    borderRadius: 8,
    backgroundColor: '#FAFAFA',
    overflow: 'hidden',
  },
  picker: {
    height: 44,
  },
  priceRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  priceInput: {
    flex: 1,
  },
  priceUnit: {
    fontSize: 14,
    color: '#666',
    marginLeft: 8,
    width: 50,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  switchLabel: {
    flex: 1,
  },
  switchHint: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  actionSection: {
    paddingHorizontal: 16,
    paddingTop: 16,
  },
  submitBtn: {
    height: 52,
    backgroundColor: THEME_COLOR,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  submitBtnDisabled: {
    opacity: 0.5,
  },
  submitBtnText: {
    fontSize: 18,
    color: '#FFF',
    fontWeight: '600',
  },
});
