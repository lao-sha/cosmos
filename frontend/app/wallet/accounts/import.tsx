import { WalletService } from '@/src/lib/wallet';
import { useWalletStore } from '@/src/stores/wallet';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    ActivityIndicator,
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

export default function ImportAccountScreen() {
  const router = useRouter();
  const { importAccount, switchAccount } = useWalletStore();
  
  const [name, setName] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [isImporting, setIsImporting] = useState(false);

  const handleImport = async () => {
    if (!name.trim()) {
      if (Platform.OS === 'web') {
        alert('请输入账户名称');
      } else {
        Alert.alert('提示', '请输入账户名称');
      }
      return;
    }

    const words = mnemonic.trim().split(/\s+/);
    if (words.length !== 12 && words.length !== 24) {
      if (Platform.OS === 'web') {
        alert('请输入有效的12或24个助记词');
      } else {
        Alert.alert('提示', '请输入有效的12或24个助记词');
      }
      return;
    }

    if (!WalletService.validateMnemonic(mnemonic.trim())) {
      if (Platform.OS === 'web') {
        alert('助记词无效，请检查拼写');
      } else {
        Alert.alert('错误', '助记词无效，请检查拼写');
      }
      return;
    }

    try {
      setIsImporting(true);
      const newAccount = await importAccount(name.trim(), mnemonic.trim());
      
      if (newAccount) {
        await switchAccount(newAccount.id);
        if (Platform.OS === 'web') {
          alert('账户导入成功！');
        } else {
          Alert.alert('成功', '账户导入成功！');
        }
        router.back();
      } else {
        throw new Error('导入失败，可能该地址已存在');
      }
    } catch (error) {
      if (Platform.OS === 'web') {
        alert('导入账户失败');
      } else {
        Alert.alert('错误', '导入账户失败');
      }
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable onPress={() => router.back()} style={styles.backButton}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>导入账户</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} keyboardShouldPersistTaps="handled">
        <View style={styles.iconContainer}>
          <Text style={styles.icon}>📥</Text>
        </View>

        <Text style={styles.title}>导入账户</Text>
        <Text style={styles.subtitle}>
          使用其他钱包的助记词导入账户
        </Text>

        <View style={styles.inputContainer}>
          <Text style={styles.label}>账户名称</Text>
          <TextInput
            style={styles.input}
            placeholder="给导入的账户起个名字"
            placeholderTextColor="#9ca3af"
            value={name}
            onChangeText={setName}
            maxLength={20}
          />
        </View>

        <View style={styles.inputContainer}>
          <Text style={styles.label}>助记词</Text>
          <TextInput
            style={[styles.input, styles.mnemonicInput]}
            placeholder="输入12或24个助记词，用空格分隔"
            placeholderTextColor="#9ca3af"
            value={mnemonic}
            onChangeText={setMnemonic}
            multiline
            numberOfLines={4}
            autoCapitalize="none"
            autoCorrect={false}
          />
          <Text style={styles.hint}>
            {mnemonic.trim().split(/\s+/).filter(Boolean).length} / 12 或 24 个词
          </Text>
        </View>

        <View style={styles.warningCard}>
          <Text style={styles.warningIcon}>⚠️</Text>
          <Text style={styles.warningText}>
            导入的账户需要单独备份其助记词。如果您丢失此助记词，将无法恢复该账户的资产。
          </Text>
        </View>

        <Pressable
          style={({ pressed }) => [
            styles.importButton,
            pressed && styles.buttonPressed,
            isImporting && styles.buttonDisabled,
          ]}
          onPress={handleImport}
          disabled={isImporting}
        >
          {isImporting ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.importButtonText}>导入账户</Text>
          )}
        </Pressable>

        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>💡 关于导入账户</Text>
          <Text style={styles.infoText}>
            • 导入的账户独立于主钱包{'\n'}
            • 每个导入账户都需要单独备份{'\n'}
            • 导入后可以随时切换使用{'\n'}
            • 删除导入账户不会影响其他账户
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
    padding: 24,
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#fef3c7',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 24,
    alignSelf: 'center',
  },
  icon: {
    fontSize: 40,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
    textAlign: 'center',
  },
  subtitle: {
    fontSize: 15,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 32,
  },
  inputContainer: {
    marginBottom: 20,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    color: '#374151',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#fff',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderRadius: 12,
    padding: 16,
    fontSize: 16,
    color: '#1f2937',
  },
  mnemonicInput: {
    height: 120,
    textAlignVertical: 'top',
  },
  hint: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'right',
    marginTop: 4,
  },
  warningCard: {
    flexDirection: 'row',
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 12,
    marginBottom: 24,
    alignItems: 'flex-start',
  },
  warningIcon: {
    fontSize: 16,
    marginRight: 8,
    marginTop: 2,
  },
  warningText: {
    flex: 1,
    fontSize: 13,
    color: '#92400e',
    lineHeight: 18,
  },
  importButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginBottom: 24,
  },
  buttonPressed: {
    opacity: 0.8,
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  importButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  infoCard: {
    backgroundColor: '#f5f3ff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 40,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#6D28D9',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#6b7280',
    lineHeight: 20,
  },
});
