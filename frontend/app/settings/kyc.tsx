import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  Image,
} from 'react-native';
import { useRouter } from 'expo-router';
import {
  Shield,
  CheckCircle,
  Clock,
  XCircle,
  Camera,
  Upload,
  ChevronRight,
} from 'lucide-react-native';
import * as ImagePicker from 'expo-image-picker';
import * as Haptics from 'expo-haptics';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getKycInfo,
  submitKyc,
  uploadImage,
  KYC_LEVEL_CONFIG,
  type KycInfo,
  type KycStatus,
} from '@/services/kyc';
import { Button, Input, Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

type Step = 'info' | 'upload' | 'selfie' | 'review';

const STATUS_CONFIG: Record<KycStatus, { icon: any; color: string; label: string }> = {
  none: { icon: Shield, color: Colors.kyc.none, label: '未认证' },
  pending: { icon: Clock, color: Colors.kyc.pending, label: '审核中' },
  verified: { icon: CheckCircle, color: Colors.kyc.verified, label: '已认证' },
  rejected: { icon: XCircle, color: Colors.kyc.rejected, label: '已拒绝' },
  expired: { icon: Clock, color: Colors.kyc.pending, label: '已过期' },
};

export default function KycScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address, mnemonic } = useWalletStore();

  const [kycInfo, setKycInfo] = useState<KycInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const [step, setStep] = useState<Step>('info');
  const [realName, setRealName] = useState('');
  const [idNumber, setIdNumber] = useState('');
  const [idType, setIdType] = useState<'id_card' | 'passport'>('id_card');
  const [frontImage, setFrontImage] = useState<string | null>(null);
  const [backImage, setBackImage] = useState<string | null>(null);
  const [selfieImage, setSelfieImage] = useState<string | null>(null);

  useEffect(() => {
    loadKycInfo();
  }, [address]);

  const loadKycInfo = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const info = await getKycInfo(address);
      setKycInfo(info);
    } catch (error) {
      console.error('Failed to load KYC info:', error);
    } finally {
      setLoading(false);
    }
  };

  const pickImage = async (type: 'front' | 'back' | 'selfie') => {
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: type === 'selfie' ? [1, 1] : [3, 2],
      quality: 0.8,
    });

    if (!result.canceled) {
      const uri = result.assets[0].uri;
      if (type === 'front') setFrontImage(uri);
      else if (type === 'back') setBackImage(uri);
      else setSelfieImage(uri);
    }
  };

  const takePhoto = async (type: 'front' | 'back' | 'selfie') => {
    const { status } = await ImagePicker.requestCameraPermissionsAsync();
    if (status !== 'granted') {
      Alert.alert('提示', '需要相机权限');
      return;
    }

    const result = await ImagePicker.launchCameraAsync({
      allowsEditing: true,
      aspect: type === 'selfie' ? [1, 1] : [3, 2],
      quality: 0.8,
    });

    if (!result.canceled) {
      const uri = result.assets[0].uri;
      if (type === 'front') setFrontImage(uri);
      else if (type === 'back') setBackImage(uri);
      else setSelfieImage(uri);
    }
  };

  const handleSubmit = async () => {
    if (!mnemonic) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }
    if (!realName || !idNumber || !frontImage || !backImage) {
      Alert.alert('提示', '请填写完整信息');
      return;
    }

    setSubmitting(true);
    try {
      const frontCid = await uploadImage(frontImage, 'front');
      const backCid = await uploadImage(backImage, 'back');
      const selfieCid = selfieImage ? await uploadImage(selfieImage, 'selfie') : undefined;

      await submitKyc(
        {
          realName,
          idNumber,
          idType,
          frontImageCid: frontCid,
          backImageCid: backCid,
          selfieImageCid: selfieCid,
        },
        mnemonic
      );

      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      Alert.alert('提交成功', '您的认证申请已提交，请等待审核', [
        { text: '确定', onPress: () => loadKycInfo() },
      ]);
      setStep('info');
    } catch (error: any) {
      Alert.alert('提交失败', error.message);
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  const statusConfig = STATUS_CONFIG[kycInfo?.status || 'none'];
  const StatusIcon = statusConfig.icon;
  const canApply = kycInfo?.status === 'none' || kycInfo?.status === 'rejected';

  if (kycInfo?.status === 'verified') {
    return (
      <ScrollView
        style={[styles.container, { backgroundColor: colors.background }]}
        contentContainerStyle={styles.content}
      >
        <View style={[styles.statusHeader, { backgroundColor: Colors.success + '15' }]}>
          <CheckCircle size={48} color={Colors.success} />
          <Text style={[styles.statusTitle, { color: Colors.success }]}>
            认证通过
          </Text>
          <Text style={[styles.statusSubtitle, { color: colors.textSecondary }]}>
            {kycInfo.realName}
          </Text>
        </View>

        <Card style={styles.infoCard}>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              认证等级
            </Text>
            <Text style={[styles.infoValue, { color: Colors.primary }]}>
              {KYC_LEVEL_CONFIG[kycInfo.level].label}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              证件号码
            </Text>
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              {kycInfo.idNumber}
            </Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              认证时间
            </Text>
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              {kycInfo.verifiedAt ? new Date(kycInfo.verifiedAt).toLocaleDateString() : '-'}
            </Text>
          </View>
        </Card>

        <Card style={styles.limitsCard}>
          <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
            认证权益
          </Text>
          <View style={styles.limitItem}>
            <Text style={[styles.limitLabel, { color: colors.textSecondary }]}>
              单笔限额
            </Text>
            <Text style={[styles.limitValue, { color: colors.textPrimary }]}>
              {KYC_LEVEL_CONFIG[kycInfo.level].limits.single} USDT
            </Text>
          </View>
          <View style={styles.limitItem}>
            <Text style={[styles.limitLabel, { color: colors.textSecondary }]}>
              日限额
            </Text>
            <Text style={[styles.limitValue, { color: colors.textPrimary }]}>
              {KYC_LEVEL_CONFIG[kycInfo.level].limits.daily} USDT
            </Text>
          </View>
        </Card>
      </ScrollView>
    );
  }

  if (kycInfo?.status === 'pending') {
    return (
      <ScrollView
        style={[styles.container, { backgroundColor: colors.background }]}
        contentContainerStyle={styles.content}
      >
        <View style={[styles.statusHeader, { backgroundColor: Colors.warning + '15' }]}>
          <Clock size={48} color={Colors.warning} />
          <Text style={[styles.statusTitle, { color: Colors.warning }]}>
            审核中
          </Text>
          <Text style={[styles.statusSubtitle, { color: colors.textSecondary }]}>
            预计 1-3 个工作日完成审核
          </Text>
        </View>

        <Card style={styles.infoCard}>
          <View style={styles.infoRow}>
            <Text style={[styles.infoLabel, { color: colors.textSecondary }]}>
              提交时间
            </Text>
            <Text style={[styles.infoValue, { color: colors.textPrimary }]}>
              {kycInfo.submittedAt ? new Date(kycInfo.submittedAt).toLocaleDateString() : '-'}
            </Text>
          </View>
        </Card>
      </ScrollView>
    );
  }

  // Apply Form
  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      {/* Progress */}
      <View style={styles.progress}>
        {['info', 'upload', 'selfie', 'review'].map((s, i) => {
          const steps = ['info', 'upload', 'selfie', 'review'];
          const currentIndex = steps.indexOf(step);
          const isActive = i <= currentIndex;
          return (
            <View key={s} style={styles.progressItem}>
              <View
                style={[
                  styles.progressDot,
                  { backgroundColor: isActive ? Colors.primary : colors.border },
                ]}
              >
                <Text style={[styles.progressNum, { color: isActive ? '#FFF' : colors.textTertiary }]}>
                  {i + 1}
                </Text>
              </View>
              {i < 3 && (
                <View
                  style={[
                    styles.progressLine,
                    { backgroundColor: i < currentIndex ? Colors.primary : colors.border },
                  ]}
                />
              )}
            </View>
          );
        })}
      </View>

      {step === 'info' && (
        <View>
          <Text style={[styles.stepTitle, { color: colors.textPrimary }]}>
            填写基本信息
          </Text>

          <Card style={styles.formCard}>
            <View style={styles.idTypeSelector}>
              <TouchableOpacity
                style={[
                  styles.idTypeOption,
                  {
                    backgroundColor: idType === 'id_card' ? Colors.primary : colors.surface,
                    borderColor: idType === 'id_card' ? Colors.primary : colors.border,
                  },
                ]}
                onPress={() => setIdType('id_card')}
              >
                <Text
                  style={[
                    styles.idTypeText,
                    { color: idType === 'id_card' ? '#FFF' : colors.textPrimary },
                  ]}
                >
                  身份证
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[
                  styles.idTypeOption,
                  {
                    backgroundColor: idType === 'passport' ? Colors.primary : colors.surface,
                    borderColor: idType === 'passport' ? Colors.primary : colors.border,
                  },
                ]}
                onPress={() => setIdType('passport')}
              >
                <Text
                  style={[
                    styles.idTypeText,
                    { color: idType === 'passport' ? '#FFF' : colors.textPrimary },
                  ]}
                >
                  护照
                </Text>
              </TouchableOpacity>
            </View>

            <Input
              label="真实姓名"
              placeholder="请输入真实姓名"
              value={realName}
              onChangeText={setRealName}
            />
            <Input
              label={idType === 'id_card' ? '身份证号' : '护照号'}
              placeholder={`请输入${idType === 'id_card' ? '身份证号' : '护照号'}`}
              value={idNumber}
              onChangeText={setIdNumber}
              autoCapitalize="characters"
            />
          </Card>

          <Button
            title="下一步"
            onPress={() => setStep('upload')}
            disabled={!realName || !idNumber}
          />
        </View>
      )}

      {step === 'upload' && (
        <View>
          <Text style={[styles.stepTitle, { color: colors.textPrimary }]}>
            上传证件照片
          </Text>

          <Card style={styles.uploadCard}>
            <Text style={[styles.uploadLabel, { color: colors.textPrimary }]}>
              证件正面
            </Text>
            <TouchableOpacity
              style={[styles.uploadBox, { borderColor: colors.border }]}
              onPress={() => pickImage('front')}
            >
              {frontImage ? (
                <Image source={{ uri: frontImage }} style={styles.uploadImage} />
              ) : (
                <>
                  <Upload size={32} color={colors.textTertiary} />
                  <Text style={[styles.uploadText, { color: colors.textTertiary }]}>
                    点击上传
                  </Text>
                </>
              )}
            </TouchableOpacity>
          </Card>

          <Card style={styles.uploadCard}>
            <Text style={[styles.uploadLabel, { color: colors.textPrimary }]}>
              证件反面
            </Text>
            <TouchableOpacity
              style={[styles.uploadBox, { borderColor: colors.border }]}
              onPress={() => pickImage('back')}
            >
              {backImage ? (
                <Image source={{ uri: backImage }} style={styles.uploadImage} />
              ) : (
                <>
                  <Upload size={32} color={colors.textTertiary} />
                  <Text style={[styles.uploadText, { color: colors.textTertiary }]}>
                    点击上传
                  </Text>
                </>
              )}
            </TouchableOpacity>
          </Card>

          <View style={styles.buttonRow}>
            <Button
              title="上一步"
              variant="outline"
              onPress={() => setStep('info')}
              style={styles.halfButton}
            />
            <Button
              title="下一步"
              onPress={() => setStep('selfie')}
              disabled={!frontImage || !backImage}
              style={styles.halfButton}
            />
          </View>
        </View>
      )}

      {step === 'selfie' && (
        <View>
          <Text style={[styles.stepTitle, { color: colors.textPrimary }]}>
            人脸识别（可选）
          </Text>

          <Card style={styles.uploadCard}>
            <Text style={[styles.uploadLabel, { color: colors.textPrimary }]}>
              手持证件自拍
            </Text>
            <TouchableOpacity
              style={[styles.uploadBox, styles.selfieBox, { borderColor: colors.border }]}
              onPress={() => takePhoto('selfie')}
            >
              {selfieImage ? (
                <Image source={{ uri: selfieImage }} style={styles.selfieImage} />
              ) : (
                <>
                  <Camera size={40} color={colors.textTertiary} />
                  <Text style={[styles.uploadText, { color: colors.textTertiary }]}>
                    点击拍照
                  </Text>
                </>
              )}
            </TouchableOpacity>
            <Text style={[styles.uploadHint, { color: colors.textTertiary }]}>
              提供自拍可提升认证等级
            </Text>
          </Card>

          <View style={styles.buttonRow}>
            <Button
              title="上一步"
              variant="outline"
              onPress={() => setStep('upload')}
              style={styles.halfButton}
            />
            <Button
              title="下一步"
              onPress={() => setStep('review')}
              style={styles.halfButton}
            />
          </View>
        </View>
      )}

      {step === 'review' && (
        <View>
          <Text style={[styles.stepTitle, { color: colors.textPrimary }]}>
            确认提交
          </Text>

          <Card style={styles.reviewCard}>
            <View style={styles.reviewRow}>
              <Text style={[styles.reviewLabel, { color: colors.textSecondary }]}>
                姓名
              </Text>
              <Text style={[styles.reviewValue, { color: colors.textPrimary }]}>
                {realName}
              </Text>
            </View>
            <View style={styles.reviewRow}>
              <Text style={[styles.reviewLabel, { color: colors.textSecondary }]}>
                证件类型
              </Text>
              <Text style={[styles.reviewValue, { color: colors.textPrimary }]}>
                {idType === 'id_card' ? '身份证' : '护照'}
              </Text>
            </View>
            <View style={styles.reviewRow}>
              <Text style={[styles.reviewLabel, { color: colors.textSecondary }]}>
                证件号码
              </Text>
              <Text style={[styles.reviewValue, { color: colors.textPrimary }]}>
                {idNumber}
              </Text>
            </View>
          </Card>

          <View style={[styles.notice, { backgroundColor: Colors.warning + '15' }]}>
            <Text style={[styles.noticeText, { color: Colors.warning }]}>
              提交后信息将用于身份验证，请确保信息真实有效
            </Text>
          </View>

          <View style={styles.buttonRow}>
            <Button
              title="上一步"
              variant="outline"
              onPress={() => setStep('selfie')}
              style={styles.halfButton}
            />
            <Button
              title="提交认证"
              onPress={handleSubmit}
              loading={submitting}
              style={styles.halfButton}
            />
          </View>
        </View>
      )}
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
  statusHeader: {
    alignItems: 'center',
    padding: 32,
    borderRadius: 16,
    marginBottom: 16,
  },
  statusTitle: {
    fontSize: 24,
    fontWeight: '700',
    marginTop: 12,
  },
  statusSubtitle: {
    fontSize: 14,
    marginTop: 4,
  },
  infoCard: {
    marginBottom: 16,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  infoLabel: {
    fontSize: 14,
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '500',
  },
  limitsCard: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  limitItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  limitLabel: {
    fontSize: 14,
  },
  limitValue: {
    fontSize: 14,
    fontWeight: '500',
  },
  progress: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 24,
  },
  progressItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  progressDot: {
    width: 28,
    height: 28,
    borderRadius: 14,
    alignItems: 'center',
    justifyContent: 'center',
  },
  progressNum: {
    fontSize: 14,
    fontWeight: '600',
  },
  progressLine: {
    width: 40,
    height: 2,
    marginHorizontal: 4,
  },
  stepTitle: {
    fontSize: 20,
    fontWeight: '700',
    marginBottom: 16,
  },
  formCard: {
    marginBottom: 24,
  },
  idTypeSelector: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 16,
  },
  idTypeOption: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 12,
    borderWidth: 1,
    alignItems: 'center',
  },
  idTypeText: {
    fontSize: 15,
    fontWeight: '500',
  },
  uploadCard: {
    marginBottom: 16,
  },
  uploadLabel: {
    fontSize: 15,
    fontWeight: '500',
    marginBottom: 12,
  },
  uploadBox: {
    height: 160,
    borderRadius: 12,
    borderWidth: 2,
    borderStyle: 'dashed',
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  selfieBox: {
    height: 200,
  },
  uploadImage: {
    width: '100%',
    height: '100%',
    resizeMode: 'cover',
  },
  selfieImage: {
    width: '100%',
    height: '100%',
    resizeMode: 'cover',
  },
  uploadText: {
    fontSize: 14,
    marginTop: 8,
  },
  uploadHint: {
    fontSize: 12,
    textAlign: 'center',
    marginTop: 8,
  },
  buttonRow: {
    flexDirection: 'row',
    gap: 12,
  },
  halfButton: {
    flex: 1,
  },
  reviewCard: {
    marginBottom: 16,
  },
  reviewRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 12,
  },
  reviewLabel: {
    fontSize: 14,
  },
  reviewValue: {
    fontSize: 14,
    fontWeight: '500',
  },
  notice: {
    padding: 12,
    borderRadius: 8,
    marginBottom: 24,
  },
  noticeText: {
    fontSize: 13,
    textAlign: 'center',
  },
});
