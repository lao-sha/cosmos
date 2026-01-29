import { MnemonicDisplay } from '@/src/components/MnemonicDisplay';
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
    TextInput,
    View,
} from 'react-native';

type Step = 'generate' | 'backup' | 'verify' | 'complete';

export default function CreateWalletScreen() {
  const router = useRouter();
  const { login } = useAuthStore();
  const { refreshAccounts } = useWalletStore();

  const [step, setStep] = useState<Step>('generate');
  const [mnemonic, setMnemonic] = useState<string>('');
  const [verifyIndexes, setVerifyIndexes] = useState<number[]>([]);
  const [verifyInputs, setVerifyInputs] = useState<string[]>(['', '', '']);
  const [verifyError, setVerifyError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleGenerate = async () => {
    setIsLoading(true);
    try {
      await WalletService.init();
      const newMnemonic = WalletService.generateMnemonic();
      setMnemonic(newMnemonic);

      // 随机选择3个索引用于验证
      const indexes: number[] = [];
      while (indexes.length < 3) {
        const idx = Math.floor(Math.random() * 12);
        if (!indexes.includes(idx)) {
          indexes.push(idx);
        }
      }
      setVerifyIndexes(indexes.sort((a, b) => a - b));
      setStep('backup');
    } catch (error) {
      const msg = '生成助记词失败: ' + (error as Error).message;
      if (Platform.OS === 'web') {
        alert(msg);
      } else {
        Alert.alert('错误', msg);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleBackupComplete = () => {
    setStep('verify');
  };

  const handleVerify = async () => {
    const words = mnemonic.split(' ');
    const isCorrect = verifyIndexes.every(
      (idx, i) => verifyInputs[i].toLowerCase().trim() === words[idx]
    );

    if (!isCorrect) {
      setVerifyError('验证失败，请检查输入的单词');
      return;
    }

    setIsLoading(true);
    try {
      await WalletService.saveMnemonic(mnemonic);
      // 初始化多账户系统的主账户
      const primaryAccount = await WalletService.initializePrimaryAccount(mnemonic);
      login(primaryAccount.address, mnemonic);
      await refreshAccounts();
      setStep('complete');
    } catch (error) {
      const msg = '保存钱包失败: ' + (error as Error).message;
      if (Platform.OS === 'web') {
        alert(msg);
      } else {
        Alert.alert('错误', msg);
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleFinish = () => {
    router.replace('/wallet');
  };

  const renderStep = () => {
    switch (step) {
      case 'generate':
        return (
          <View style={styles.stepContent}>
            <View style={styles.iconContainer}>
              <Text style={styles.icon}>🔑</Text>
            </View>
            <Text style={styles.stepTitle}>创建新钱包</Text>
            <Text style={styles.stepDesc}>
              系统将生成一组12个单词的助记词，这是恢复钱包的唯一凭证。
              请确保在安全的环境中操作。
            </Text>

            <View style={styles.securityTips}>
              <Text style={styles.tipTitle}>安全提示</Text>
              <Text style={styles.tipItem}>• 不要截图或拍照保存助记词</Text>
              <Text style={styles.tipItem}>• 不要在网络上传输或存储</Text>
              <Text style={styles.tipItem}>• 建议手抄并保存在安全位置</Text>
              <Text style={styles.tipItem}>• 任何人获取助记词都能控制你的资产</Text>
            </View>

            <Pressable
              style={[styles.primaryButton, isLoading && styles.buttonDisabled]}
              onPress={handleGenerate}
              disabled={isLoading}
            >
              <Text style={styles.primaryButtonText}>
                {isLoading ? '生成中...' : '生成助记词'}
              </Text>
            </Pressable>
          </View>
        );

      case 'backup':
        return (
          <View style={styles.stepContent}>
            <View style={styles.progressBar}>
              <View style={[styles.progressStep, styles.progressActive]} />
              <View style={styles.progressStep} />
              <View style={styles.progressStep} />
            </View>

            <Text style={styles.stepTitle}>备份助记词</Text>
            <Text style={styles.stepDesc}>
              请按顺序抄写这12个单词，完成后点击继续。
            </Text>

            <MnemonicDisplay mnemonic={mnemonic} showCopy={false} />

            <Pressable style={styles.primaryButton} onPress={handleBackupComplete}>
              <Text style={styles.primaryButtonText}>我已完成备份</Text>
            </Pressable>
          </View>
        );

      case 'verify':
        const words = mnemonic.split(' ');
        return (
          <View style={styles.stepContent}>
            <View style={styles.progressBar}>
              <View style={[styles.progressStep, styles.progressActive]} />
              <View style={[styles.progressStep, styles.progressActive]} />
              <View style={styles.progressStep} />
            </View>

            <Text style={styles.stepTitle}>验证助记词</Text>
            <Text style={styles.stepDesc}>
              请输入以下位置的单词以验证你已正确备份
            </Text>

            <View style={styles.verifyInputs}>
              {verifyIndexes.map((wordIndex, i) => (
                <View key={wordIndex} style={styles.verifyItem}>
                  <Text style={styles.verifyLabel}>第 {wordIndex + 1} 个单词</Text>
                  <TextInput
                    style={styles.verifyInput}
                    placeholder="输入单词"
                    placeholderTextColor="#9ca3af"
                    value={verifyInputs[i]}
                    onChangeText={(text) => {
                      const newInputs = [...verifyInputs];
                      newInputs[i] = text;
                      setVerifyInputs(newInputs);
                      setVerifyError(null);
                    }}
                    autoCapitalize="none"
                    autoCorrect={false}
                  />
                </View>
              ))}
            </View>

            {verifyError && (
              <View style={styles.errorBox}>
                <Text style={styles.errorText}>❌ {verifyError}</Text>
              </View>
            )}

            <Pressable
              style={[styles.primaryButton, isLoading && styles.buttonDisabled]}
              onPress={handleVerify}
              disabled={isLoading}
            >
              <Text style={styles.primaryButtonText}>
                {isLoading ? '验证中...' : '验证并创建钱包'}
              </Text>
            </Pressable>

            <Pressable
              style={styles.textButton}
              onPress={() => setStep('backup')}
            >
              <Text style={styles.textButtonText}>返回查看助记词</Text>
            </Pressable>
          </View>
        );

      case 'complete':
        return (
          <View style={styles.stepContent}>
            <View style={styles.progressBar}>
              <View style={[styles.progressStep, styles.progressActive]} />
              <View style={[styles.progressStep, styles.progressActive]} />
              <View style={[styles.progressStep, styles.progressActive]} />
            </View>

            <View style={styles.successIcon}>
              <Text style={styles.successEmoji}>✅</Text>
            </View>
            <Text style={styles.stepTitle}>钱包创建成功！</Text>
            <Text style={styles.stepDesc}>
              你的钱包已安全创建。请妥善保管助记词，
              它是找回钱包的唯一方式。
            </Text>

            <Pressable style={styles.primaryButton} onPress={handleFinish}>
              <Text style={styles.primaryButtonText}>开始使用</Text>
            </Pressable>
          </View>
        );
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable
          style={styles.backButton}
          onPress={() => {
            if (step === 'generate' || step === 'complete') {
              router.back();
            } else if (step === 'backup') {
              setStep('generate');
              setMnemonic('');
            } else if (step === 'verify') {
              setStep('backup');
            }
          }}
        >
          <Text style={styles.backText}>
            {step === 'complete' ? '✕' : '‹ 返回'}
          </Text>
        </Pressable>
        <Text style={styles.headerTitle}>创建钱包</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView
        style={styles.content}
        contentContainerStyle={styles.contentContainer}
        showsVerticalScrollIndicator={false}
      >
        {renderStep()}
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
  stepContent: {
    flex: 1,
  },
  progressBar: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 24,
  },
  progressStep: {
    flex: 1,
    height: 4,
    backgroundColor: '#e5e7eb',
    borderRadius: 2,
  },
  progressActive: {
    backgroundColor: '#6D28D9',
  },
  iconContainer: {
    alignSelf: 'center',
    marginBottom: 24,
    marginTop: 40,
  },
  icon: {
    fontSize: 64,
  },
  stepTitle: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 12,
  },
  stepDesc: {
    fontSize: 15,
    color: '#6b7280',
    textAlign: 'center',
    lineHeight: 22,
    marginBottom: 24,
  },
  securityTips: {
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 32,
  },
  tipTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  tipItem: {
    fontSize: 13,
    color: '#78350f',
    lineHeight: 22,
  },
  primaryButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 16,
  },
  buttonDisabled: {
    backgroundColor: '#d1d5db',
  },
  primaryButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  verifyInputs: {
    gap: 16,
    marginBottom: 16,
  },
  verifyItem: {},
  verifyLabel: {
    fontSize: 14,
    color: '#374151',
    marginBottom: 8,
  },
  verifyInput: {
    backgroundColor: '#fff',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 16,
    color: '#1f2937',
    borderWidth: 1,
    borderColor: '#e5e7eb',
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
  textButton: {
    marginTop: 16,
    alignItems: 'center',
  },
  textButtonText: {
    fontSize: 14,
    color: '#6D28D9',
  },
  successIcon: {
    alignSelf: 'center',
    marginTop: 40,
    marginBottom: 24,
  },
  successEmoji: {
    fontSize: 72,
  },
});
