import { MnemonicInput } from '@/src/components/MnemonicInput';
import { WalletService } from '@/src/lib/wallet';
import { useAuthStore } from '@/src/stores/auth';
import { useWalletStore } from '@/src/stores/wallet';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    View,
} from 'react-native';

export default function ImportWalletScreen() {
  const router = useRouter();
  const { login } = useAuthStore();
  const { refreshAccounts } = useWalletStore();

  const [isImporting, setIsImporting] = useState(false);
  const [success, setSuccess] = useState(false);

  const handleValidMnemonic = async (mnemonic: string) => {
    setIsImporting(true);
    try {
      await WalletService.saveMnemonic(mnemonic);
      // 初始化多账户系统的主账户
      const primaryAccount = await WalletService.initializePrimaryAccount(mnemonic);
      login(primaryAccount.address, mnemonic);
      await refreshAccounts();
      setSuccess(true);
    } catch (error) {
      const msg = '导入失败: ' + (error as Error).message;
      if (Platform.OS === 'web') {
        alert(msg);
      } else {
        Alert.alert('错误', msg);
      }
    } finally {
      setIsImporting(false);
    }
  };

  const handleFinish = () => {
    router.replace('/wallet');
  };

  if (success) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <View style={styles.backButton} />
          <Text style={styles.headerTitle}>导入钱包</Text>
          <View style={styles.headerRight} />
        </View>

        <View style={styles.successContainer}>
          <Text style={styles.successEmoji}>✅</Text>
          <Text style={styles.successTitle}>导入成功！</Text>
          <Text style={styles.successDesc}>
            你的钱包已成功导入，现在可以开始使用了。
          </Text>
          <Pressable style={styles.primaryButton} onPress={handleFinish}>
            <Text style={styles.primaryButtonText}>开始使用</Text>
          </Pressable>
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
        <Text style={styles.headerTitle}>导入钱包</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentContainer}
        showsVerticalScrollIndicator={false}
      >
        <View style={styles.intro}>
          <Text style={styles.introTitle}>恢复你的钱包</Text>
          <Text style={styles.introDesc}>
            输入你之前备份的12个助记词单词来恢复钱包。
            请确保在安全的环境中操作。
          </Text>
        </View>

        <MnemonicInput
          onValidMnemonic={handleValidMnemonic}
          onInvalid={() => {}}
        />

        {isImporting && (
          <View style={styles.loadingOverlay}>
            <Text style={styles.loadingText}>正在导入钱包...</Text>
          </View>
        )}

        <View style={styles.securityNote}>
          <Text style={styles.noteIcon}>🔒</Text>
          <Text style={styles.noteText}>
            你的助记词将安全存储在设备本地，不会上传到任何服务器。
          </Text>
        </View>
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
    width: 50,
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
  intro: {
    marginBottom: 24,
  },
  introTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1f2937',
    marginBottom: 8,
  },
  introDesc: {
    fontSize: 15,
    color: '#6b7280',
    lineHeight: 22,
  },
  loadingOverlay: {
    marginTop: 16,
    padding: 16,
    backgroundColor: '#f5f3ff',
    borderRadius: 12,
    alignItems: 'center',
  },
  loadingText: {
    fontSize: 15,
    color: '#6D28D9',
    fontWeight: '500',
  },
  securityNote: {
    flexDirection: 'row',
    marginTop: 24,
    padding: 16,
    backgroundColor: '#eff6ff',
    borderRadius: 12,
  },
  noteIcon: {
    fontSize: 20,
    marginRight: 12,
  },
  noteText: {
    flex: 1,
    fontSize: 13,
    color: '#1e40af',
    lineHeight: 20,
  },
  successContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  successEmoji: {
    fontSize: 72,
    marginBottom: 24,
  },
  successTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1f2937',
    marginBottom: 12,
  },
  successDesc: {
    fontSize: 15,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 32,
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    paddingHorizontal: 48,
    borderRadius: 12,
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
