import { useWalletStore } from '@/src/stores/wallet';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    ActivityIndicator,
    Alert,
    Platform,
    Pressable,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

export default function CreateAccountScreen() {
  const router = useRouter();
  const { createAccount, switchAccount } = useWalletStore();
  
  const [name, setName] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  const handleCreate = async () => {
    if (!name.trim()) {
      if (Platform.OS === 'web') {
        alert('请输入账户名称');
      } else {
        Alert.alert('提示', '请输入账户名称');
      }
      return;
    }

    try {
      setIsCreating(true);
      const newAccount = await createAccount(name.trim());
      
      if (newAccount) {
        await switchAccount(newAccount.id);
        if (Platform.OS === 'web') {
          alert('账户创建成功！');
        } else {
          Alert.alert('成功', '账户创建成功！');
        }
        router.back();
      } else {
        throw new Error('创建失败');
      }
    } catch (error) {
      if (Platform.OS === 'web') {
        alert('创建账户失败');
      } else {
        Alert.alert('错误', '创建账户失败');
      }
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable onPress={() => router.back()} style={styles.backButton}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>创建账户</Text>
        <View style={styles.headerRight} />
      </View>

      <View style={styles.content}>
        <View style={styles.iconContainer}>
          <Text style={styles.icon}>👤</Text>
        </View>

        <Text style={styles.title}>创建新账户</Text>
        <Text style={styles.subtitle}>
          新账户将从您的主助记词派生，无需额外备份
        </Text>

        <View style={styles.inputContainer}>
          <Text style={styles.label}>账户名称</Text>
          <TextInput
            style={styles.input}
            placeholder="例如：工作账户、储蓄账户"
            placeholderTextColor="#9ca3af"
            value={name}
            onChangeText={setName}
            maxLength={20}
          />
          <Text style={styles.hint}>{name.length}/20</Text>
        </View>

        <Pressable
          style={({ pressed }) => [
            styles.createButton,
            pressed && styles.buttonPressed,
            isCreating && styles.buttonDisabled,
          ]}
          onPress={handleCreate}
          disabled={isCreating}
        >
          {isCreating ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.createButtonText}>创建账户</Text>
          )}
        </Pressable>

        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>💡 HD派生账户</Text>
          <Text style={styles.infoText}>
            HD（分层确定性）钱包使用同一助记词派生多个账户。{'\n\n'}
            优点：{'\n'}
            • 只需备份一次助记词{'\n'}
            • 每个账户都有独立地址{'\n'}
            • 可随时恢复所有派生账户
          </Text>
        </View>
      </View>
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
    alignItems: 'center',
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: '#f5f3ff',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 24,
  },
  icon: {
    fontSize: 40,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 15,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 32,
  },
  inputContainer: {
    width: '100%',
    marginBottom: 24,
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
  hint: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'right',
    marginTop: 4,
  },
  createButton: {
    width: '100%',
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
  createButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  infoCard: {
    width: '100%',
    backgroundColor: '#f5f3ff',
    borderRadius: 12,
    padding: 16,
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
