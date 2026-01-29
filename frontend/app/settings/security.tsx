import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Switch,
    Text,
    TextInput,
    View,
} from 'react-native';

export default function SecuritySettingsScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();

  const [biometricEnabled, setBiometricEnabled] = useState(false);
  const [transactionPassword, setTransactionPassword] = useState(false);
  const [autoLock, setAutoLock] = useState(true);
  const [autoLockTime, setAutoLockTime] = useState<'1' | '5' | '15' | '30'>('5');

  // 修改交易密码
  const [showPasswordModal, setShowPasswordModal] = useState(false);
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  const showAlert = (title: string, msg: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n\n${msg}`);
    } else {
      Alert.alert(title, msg);
    }
  };

  const handleToggleBiometric = (value: boolean) => {
    if (value) {
      // TODO: 实际检查设备是否支持生物识别
      showAlert('提示', '生物识别功能需要设备支持');
    }
    setBiometricEnabled(value);
  };

  const handleToggleTransactionPassword = (value: boolean) => {
    if (value && !transactionPassword) {
      setShowPasswordModal(true);
    } else {
      setTransactionPassword(value);
    }
  };

  const handleSetPassword = () => {
    if (!newPassword || newPassword.length < 6) {
      showAlert('错误', '密码至少需要6位');
      return;
    }
    if (newPassword !== confirmPassword) {
      showAlert('错误', '两次输入的密码不一致');
      return;
    }
    
    // TODO: 实际保存密码到安全存储
    setTransactionPassword(true);
    setShowPasswordModal(false);
    setCurrentPassword('');
    setNewPassword('');
    setConfirmPassword('');
    showAlert('成功', '交易密码已设置');
  };

  const autoLockOptions = [
    { value: '1', label: '1分钟' },
    { value: '5', label: '5分钟' },
    { value: '15', label: '15分钟' },
    { value: '30', label: '30分钟' },
  ];

  if (!isLoggedIn) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>安全设置</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.emptyContainer}>
          <Text style={styles.emptyIcon}>🔒</Text>
          <Text style={styles.emptyText}>请先登录钱包</Text>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>安全设置</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>身份验证</Text>
          <View style={styles.sectionContent}>
            <View style={styles.settingItem}>
              <View style={styles.settingInfo}>
                <Text style={styles.settingTitle}>生物识别</Text>
                <Text style={styles.settingDesc}>使用指纹或面容解锁应用</Text>
              </View>
              <Switch
                value={biometricEnabled}
                onValueChange={handleToggleBiometric}
                trackColor={{ false: '#e5e7eb', true: '#c4b5fd' }}
                thumbColor={biometricEnabled ? '#6D28D9' : '#f4f3f4'}
              />
            </View>

            <View style={styles.settingItem}>
              <View style={styles.settingInfo}>
                <Text style={styles.settingTitle}>交易密码</Text>
                <Text style={styles.settingDesc}>
                  {transactionPassword ? '已设置' : '未设置'} · 每次交易时验证
                </Text>
              </View>
              <Switch
                value={transactionPassword}
                onValueChange={handleToggleTransactionPassword}
                trackColor={{ false: '#e5e7eb', true: '#c4b5fd' }}
                thumbColor={transactionPassword ? '#6D28D9' : '#f4f3f4'}
              />
            </View>

            {transactionPassword && (
              <Pressable
                style={styles.changePasswordButton}
                onPress={() => setShowPasswordModal(true)}
              >
                <Text style={styles.changePasswordText}>修改交易密码</Text>
              </Pressable>
            )}
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>自动锁定</Text>
          <View style={styles.sectionContent}>
            <View style={styles.settingItem}>
              <View style={styles.settingInfo}>
                <Text style={styles.settingTitle}>自动锁定</Text>
                <Text style={styles.settingDesc}>闲置后自动锁定应用</Text>
              </View>
              <Switch
                value={autoLock}
                onValueChange={setAutoLock}
                trackColor={{ false: '#e5e7eb', true: '#c4b5fd' }}
                thumbColor={autoLock ? '#6D28D9' : '#f4f3f4'}
              />
            </View>

            {autoLock && (
              <View style={styles.optionsRow}>
                {autoLockOptions.map((option) => (
                  <Pressable
                    key={option.value}
                    style={[
                      styles.optionButton,
                      autoLockTime === option.value && styles.optionButtonActive,
                    ]}
                    onPress={() => setAutoLockTime(option.value as any)}
                  >
                    <Text
                      style={[
                        styles.optionText,
                        autoLockTime === option.value && styles.optionTextActive,
                      ]}
                    >
                      {option.label}
                    </Text>
                  </Pressable>
                ))}
              </View>
            )}
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>钱包安全</Text>
          <View style={styles.sectionContent}>
            <Pressable
              style={styles.menuItem}
              onPress={() => router.push('/wallet/backup')}
            >
              <View style={styles.menuInfo}>
                <Text style={styles.menuIcon}>🔐</Text>
                <Text style={styles.menuTitle}>备份助记词</Text>
              </View>
              <Text style={styles.menuArrow}>›</Text>
            </Pressable>

            <Pressable style={styles.menuItem}>
              <View style={styles.menuInfo}>
                <Text style={styles.menuIcon}>🔑</Text>
                <Text style={styles.menuTitle}>导出私钥</Text>
              </View>
              <Text style={styles.menuArrow}>›</Text>
            </Pressable>

            <Pressable style={styles.menuItem}>
              <View style={styles.menuInfo}>
                <Text style={styles.menuIcon}>📱</Text>
                <Text style={styles.menuTitle}>授权管理</Text>
              </View>
              <Text style={styles.menuArrow}>›</Text>
            </Pressable>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>安全提醒</Text>
          <View style={styles.tipCard}>
            <Text style={styles.tipTitle}>🛡️ 保护你的资产</Text>
            <Text style={styles.tipText}>
              • 永远不要分享你的助记词或私钥{'\n'}
              • 不要在不信任的网站输入钱包信息{'\n'}
              • 定期检查授权的应用和合约{'\n'}
              • 使用强密码并开启双重验证
            </Text>
          </View>
        </View>

        <View style={styles.bottomPadding} />
      </ScrollView>

      {showPasswordModal && (
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>
              {transactionPassword ? '修改交易密码' : '设置交易密码'}
            </Text>

            {transactionPassword && (
              <View style={styles.inputGroup}>
                <Text style={styles.inputLabel}>当前密码</Text>
                <TextInput
                  style={styles.input}
                  value={currentPassword}
                  onChangeText={setCurrentPassword}
                  placeholder="输入当前密码"
                  placeholderTextColor="#9ca3af"
                  secureTextEntry
                />
              </View>
            )}

            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>新密码</Text>
              <TextInput
                style={styles.input}
                value={newPassword}
                onChangeText={setNewPassword}
                placeholder="输入新密码（至少6位）"
                placeholderTextColor="#9ca3af"
                secureTextEntry
              />
            </View>

            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>确认密码</Text>
              <TextInput
                style={styles.input}
                value={confirmPassword}
                onChangeText={setConfirmPassword}
                placeholder="再次输入新密码"
                placeholderTextColor="#9ca3af"
                secureTextEntry
              />
            </View>

            <View style={styles.modalActions}>
              <Pressable
                style={[styles.modalButton, styles.cancelButton]}
                onPress={() => {
                  setShowPasswordModal(false);
                  setCurrentPassword('');
                  setNewPassword('');
                  setConfirmPassword('');
                }}
              >
                <Text style={styles.cancelButtonText}>取消</Text>
              </Pressable>
              <Pressable
                style={[styles.modalButton, styles.confirmButton]}
                onPress={handleSetPassword}
              >
                <Text style={styles.confirmButtonText}>确认</Text>
              </Pressable>
            </View>
          </View>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  emptyText: {
    fontSize: 16,
    color: '#6b7280',
  },
  section: {
    marginTop: 24,
  },
  sectionTitle: {
    fontSize: 13,
    fontWeight: '600',
    color: '#9ca3af',
    paddingHorizontal: 16,
    marginBottom: 8,
    textTransform: 'uppercase',
  },
  sectionContent: {
    backgroundColor: '#fff',
  },
  settingItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 14,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  settingInfo: {
    flex: 1,
  },
  settingTitle: {
    fontSize: 16,
    color: '#1f2937',
  },
  settingDesc: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 2,
  },
  changePasswordButton: {
    padding: 16,
  },
  changePasswordText: {
    fontSize: 15,
    color: '#6D28D9',
    fontWeight: '500',
  },
  optionsRow: {
    flexDirection: 'row',
    padding: 16,
    gap: 8,
  },
  optionButton: {
    flex: 1,
    paddingVertical: 10,
    borderRadius: 8,
    backgroundColor: '#f3f4f6',
    alignItems: 'center',
  },
  optionButtonActive: {
    backgroundColor: '#6D28D9',
  },
  optionText: {
    fontSize: 13,
    color: '#6b7280',
  },
  optionTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  menuItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 14,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  menuInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  menuIcon: {
    fontSize: 20,
  },
  menuTitle: {
    fontSize: 16,
    color: '#1f2937',
  },
  menuArrow: {
    fontSize: 20,
    color: '#d1d5db',
  },
  tipCard: {
    backgroundColor: '#eff6ff',
    margin: 16,
    marginTop: 0,
    padding: 16,
    borderRadius: 12,
  },
  tipTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1e40af',
    marginBottom: 8,
  },
  tipText: {
    fontSize: 13,
    color: '#1e3a8a',
    lineHeight: 22,
  },
  bottomPadding: {
    height: 32,
  },
  modalOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modalContent: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 24,
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    color: '#374151',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#f9fafb',
    borderRadius: 10,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 16,
    color: '#1f2937',
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  modalActions: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 8,
  },
  modalButton: {
    flex: 1,
    paddingVertical: 14,
    borderRadius: 10,
    alignItems: 'center',
  },
  cancelButton: {
    backgroundColor: '#f3f4f6',
  },
  cancelButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#6b7280',
  },
  confirmButton: {
    backgroundColor: '#6D28D9',
  },
  confirmButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#fff',
  },
});
