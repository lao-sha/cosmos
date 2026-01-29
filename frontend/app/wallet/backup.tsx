import { MnemonicDisplay } from '@/src/components/MnemonicDisplay';
import { useAuthStore } from '@/src/stores/auth';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View
} from 'react-native';

export default function BackupScreen() {
  const router = useRouter();
  const { mnemonic, address } = useAuthStore();

  const [verified, setVerified] = useState(false);
  const [inputAddress, setInputAddress] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleVerify = () => {
    // 简单验证：要求输入地址的后6位
    const lastSix = address?.slice(-6) || '';
    if (inputAddress.toLowerCase() === lastSix.toLowerCase()) {
      setVerified(true);
      setError(null);
    } else {
      setError('验证失败，请输入正确的地址后6位');
    }
  };

  if (!mnemonic) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>备份助记词</Text>
          <View style={styles.headerRight} />
        </View>

        <View style={styles.errorContainer}>
          <Text style={styles.errorIcon}>⚠️</Text>
          <Text style={styles.errorTitle}>无法获取助记词</Text>
          <Text style={styles.errorDesc}>
            助记词未在当前会话中加载。请重新登录后再试。
          </Text>
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
        <Text style={styles.headerTitle}>备份助记词</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentContainer}
        showsVerticalScrollIndicator={false}
      >
        {!verified ? (
          <View style={styles.verifySection}>
            <View style={styles.lockIcon}>
              <Text style={styles.lockEmoji}>🔐</Text>
            </View>
            <Text style={styles.verifyTitle}>身份验证</Text>
            <Text style={styles.verifyDesc}>
              为了保护你的资产安全，请输入你的钱包地址后6位以验证身份。
            </Text>

            <View style={styles.addressPreview}>
              <Text style={styles.addressLabel}>你的钱包地址</Text>
              <Text style={styles.addressValue}>
                {address?.slice(0, 8)}...{address?.slice(-6)}
              </Text>
            </View>

            <View style={styles.inputGroup}>
              <Text style={styles.inputLabel}>请输入地址后6位</Text>
              <TextInput
                style={styles.input}
                placeholder="输入后6位字符"
                placeholderTextColor="#9ca3af"
                value={inputAddress}
                onChangeText={(text) => {
                  setInputAddress(text);
                  setError(null);
                }}
                autoCapitalize="none"
                maxLength={6}
              />
            </View>

            {error && (
              <View style={styles.errorBox}>
                <Text style={styles.errorText}>❌ {error}</Text>
              </View>
            )}

            <Pressable style={styles.primaryButton} onPress={handleVerify}>
              <Text style={styles.primaryButtonText}>验证身份</Text>
            </Pressable>
          </View>
        ) : (
          <View style={styles.backupSection}>
            <View style={styles.warningBanner}>
              <Text style={styles.warningIcon}>⚠️</Text>
              <Text style={styles.warningText}>
                请确保周围没有人偷看你的屏幕。不要截图或拍照。
              </Text>
            </View>

            <MnemonicDisplay mnemonic={mnemonic} blurred={true} />

            <View style={styles.tips}>
              <Text style={styles.tipsTitle}>安全备份建议</Text>
              <View style={styles.tipItem}>
                <Text style={styles.tipIcon}>📝</Text>
                <Text style={styles.tipText}>手写在纸上，保存在安全的地方</Text>
              </View>
              <View style={styles.tipItem}>
                <Text style={styles.tipIcon}>🔒</Text>
                <Text style={styles.tipText}>可以分开保存在多个位置</Text>
              </View>
              <View style={styles.tipItem}>
                <Text style={styles.tipIcon}>🚫</Text>
                <Text style={styles.tipText}>不要存储在云端或发送给任何人</Text>
              </View>
              <View style={styles.tipItem}>
                <Text style={styles.tipIcon}>💡</Text>
                <Text style={styles.tipText}>考虑使用金属助记词备份板</Text>
              </View>
            </View>

            <Pressable
              style={styles.doneButton}
              onPress={() => router.back()}
            >
              <Text style={styles.doneButtonText}>完成</Text>
            </Pressable>
          </View>
        )}
      </ScrollView>
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
  contentContainer: {
    padding: 16,
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  errorIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  errorTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  errorDesc: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
  },
  verifySection: {
    paddingTop: 40,
  },
  lockIcon: {
    alignSelf: 'center',
    marginBottom: 24,
  },
  lockEmoji: {
    fontSize: 64,
  },
  verifyTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 12,
  },
  verifyDesc: {
    fontSize: 15,
    color: '#6b7280',
    textAlign: 'center',
    lineHeight: 22,
    marginBottom: 32,
  },
  addressPreview: {
    backgroundColor: '#f3f4f6',
    borderRadius: 12,
    padding: 16,
    marginBottom: 24,
  },
  addressLabel: {
    fontSize: 12,
    color: '#6b7280',
    marginBottom: 4,
  },
  addressValue: {
    fontSize: 16,
    fontFamily: 'monospace',
    color: '#1f2937',
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
    backgroundColor: '#fff',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 18,
    color: '#1f2937',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    textAlign: 'center',
    letterSpacing: 4,
    fontFamily: 'monospace',
  },
  errorBox: {
    backgroundColor: '#fef2f2',
    borderRadius: 8,
    padding: 12,
    marginBottom: 16,
  },
  errorText: {
    fontSize: 14,
    color: '#dc2626',
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  backupSection: {},
  warningBanner: {
    flexDirection: 'row',
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 20,
    alignItems: 'center',
  },
  warningIcon: {
    fontSize: 20,
    marginRight: 12,
  },
  warningText: {
    flex: 1,
    fontSize: 13,
    color: '#92400e',
    lineHeight: 20,
  },
  tips: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginTop: 20,
  },
  tipsTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
  },
  tipItem: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  tipIcon: {
    fontSize: 18,
    marginRight: 12,
  },
  tipText: {
    flex: 1,
    fontSize: 14,
    color: '#4b5563',
  },
  doneButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 24,
  },
  doneButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
