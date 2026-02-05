import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Download, Key, FileText } from 'lucide-react-native';
import * as Haptics from 'expo-haptics';
import { mnemonicValidate } from '@polkadot/util-crypto';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { Button, Input, Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

type ImportType = 'mnemonic' | 'privateKey';

export default function ImportWalletScreen() {
  const colors = useColors();
  const router = useRouter();
  const { importWallet } = useWalletStore();

  const [importType, setImportType] = useState<ImportType>('mnemonic');
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [loading, setLoading] = useState(false);

  const handleImport = async () => {
    if (!name.trim()) {
      Alert.alert('提示', '请输入钱包名称');
      return;
    }
    if (password.length < 6) {
      Alert.alert('提示', '密码至少6位');
      return;
    }
    if (password !== confirmPassword) {
      Alert.alert('提示', '两次密码不一致');
      return;
    }

    if (importType === 'mnemonic') {
      const words = mnemonic.trim().split(/\s+/);
      if (words.length !== 12 && words.length !== 24) {
        Alert.alert('提示', '助记词应为12或24个单词');
        return;
      }
      if (!mnemonicValidate(mnemonic.trim())) {
        Alert.alert('提示', '无效的助记词');
        return;
      }
    } else {
      if (!privateKey.trim() || privateKey.length < 64) {
        Alert.alert('提示', '请输入有效的私钥');
        return;
      }
    }

    setLoading(true);
    try {
      const secret = importType === 'mnemonic' ? mnemonic.trim() : privateKey.trim();
      await importWallet(secret, name, password);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      router.replace('/(tabs)');
    } catch (error: any) {
      Alert.alert('导入失败', error.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      <View style={styles.iconContainer}>
        <Download size={48} color={Colors.primary} />
      </View>
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        导入钱包
      </Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        使用助记词或私钥恢复钱包
      </Text>

      {/* Import Type Selector */}
      <View style={styles.typeSelector}>
        <TouchableOpacity
          style={[
            styles.typeOption,
            {
              backgroundColor:
                importType === 'mnemonic' ? Colors.primary : colors.surface,
              borderColor:
                importType === 'mnemonic' ? Colors.primary : colors.border,
            },
          ]}
          onPress={() => setImportType('mnemonic')}
        >
          <FileText
            size={20}
            color={importType === 'mnemonic' ? '#FFFFFF' : colors.textSecondary}
          />
          <Text
            style={[
              styles.typeText,
              {
                color:
                  importType === 'mnemonic' ? '#FFFFFF' : colors.textPrimary,
              },
            ]}
          >
            助记词
          </Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[
            styles.typeOption,
            {
              backgroundColor:
                importType === 'privateKey' ? Colors.primary : colors.surface,
              borderColor:
                importType === 'privateKey' ? Colors.primary : colors.border,
            },
          ]}
          onPress={() => setImportType('privateKey')}
        >
          <Key
            size={20}
            color={importType === 'privateKey' ? '#FFFFFF' : colors.textSecondary}
          />
          <Text
            style={[
              styles.typeText,
              {
                color:
                  importType === 'privateKey' ? '#FFFFFF' : colors.textPrimary,
              },
            ]}
          >
            私钥
          </Text>
        </TouchableOpacity>
      </View>

      {/* Form */}
      <View style={styles.form}>
        <Input
          label="钱包名称"
          placeholder="输入钱包名称"
          value={name}
          onChangeText={setName}
          maxLength={20}
        />

        {importType === 'mnemonic' ? (
          <View style={styles.inputGroup}>
            <Text style={[styles.label, { color: colors.textSecondary }]}>
              助记词
            </Text>
            <Card style={styles.mnemonicInput}>
              <Input
                placeholder="输入12或24个助记词，用空格分隔"
                value={mnemonic}
                onChangeText={setMnemonic}
                multiline
                numberOfLines={4}
                style={styles.textArea}
                containerStyle={styles.noMargin}
                autoCapitalize="none"
                autoCorrect={false}
              />
            </Card>
            <Text style={[styles.hint, { color: colors.textTertiary }]}>
              请按顺序输入助记词，单词之间用空格分隔
            </Text>
          </View>
        ) : (
          <View style={styles.inputGroup}>
            <Text style={[styles.label, { color: colors.textSecondary }]}>
              私钥
            </Text>
            <Input
              placeholder="输入私钥（0x开头或不带）"
              value={privateKey}
              onChangeText={setPrivateKey}
              containerStyle={styles.noMargin}
              autoCapitalize="none"
              autoCorrect={false}
              secureTextEntry
            />
          </View>
        )}

        <Input
          label="密码"
          placeholder="设置钱包密码（至少6位）"
          value={password}
          onChangeText={setPassword}
          secureTextEntry
        />

        <Input
          label="确认密码"
          placeholder="再次输入密码"
          value={confirmPassword}
          onChangeText={setConfirmPassword}
          secureTextEntry
        />
      </View>

      {/* Warning */}
      <Card style={[styles.warningCard, { backgroundColor: Colors.warning + '10' }]}>
        <Text style={[styles.warningTitle, { color: Colors.warning }]}>
          ⚠️ 安全提示
        </Text>
        <Text style={[styles.warningText, { color: colors.textSecondary }]}>
          • 请确保在安全环境下导入{'\n'}
          • 不要在公共网络或设备上操作{'\n'}
          • 导入后助记词/私钥不会上传服务器
        </Text>
      </Card>

      <Button
        title="导入钱包"
        onPress={handleImport}
        style={styles.button}
        loading={loading}
      />
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 20,
    paddingBottom: 40,
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: 'rgba(99, 102, 241, 0.1)',
    alignItems: 'center',
    justifyContent: 'center',
    alignSelf: 'center',
    marginBottom: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    textAlign: 'center',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    textAlign: 'center',
    marginBottom: 24,
  },
  typeSelector: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 24,
  },
  typeOption: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 14,
    borderRadius: 12,
    borderWidth: 1,
    gap: 8,
  },
  typeText: {
    fontSize: 16,
    fontWeight: '500',
  },
  form: {
    marginBottom: 16,
  },
  inputGroup: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 8,
  },
  mnemonicInput: {
    padding: 4,
  },
  textArea: {
    minHeight: 100,
    textAlignVertical: 'top',
  },
  noMargin: {
    marginBottom: 0,
  },
  hint: {
    fontSize: 12,
    marginTop: 8,
  },
  warningCard: {
    marginBottom: 24,
    borderWidth: 0,
  },
  warningTitle: {
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 8,
  },
  warningText: {
    fontSize: 13,
    lineHeight: 20,
  },
  button: {
    width: '100%',
  },
});
