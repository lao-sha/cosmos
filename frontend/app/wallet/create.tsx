import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Copy, Eye, EyeOff, Shield, CheckCircle } from 'lucide-react-native';
import * as Clipboard from 'expo-clipboard';
import * as Haptics from 'expo-haptics';
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import { Button, Input, Card } from '@/components/ui';
import { Colors, Shadows } from '@/constants/colors';

type Step = 'setup' | 'mnemonic' | 'verify';

export default function CreateWalletScreen() {
  const colors = useColors();
  const router = useRouter();
  const { createWallet } = useWalletStore();

  const [step, setStep] = useState<Step>('setup');
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [showMnemonic, setShowMnemonic] = useState(false);
  const [copied, setCopied] = useState(false);
  const [selectedWords, setSelectedWords] = useState<number[]>([]);
  const [verifyIndices, setVerifyIndices] = useState<number[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (step === 'mnemonic' && !mnemonic) {
      const newMnemonic = mnemonicGenerate();
      setMnemonic(newMnemonic);
    }
  }, [step]);

  useEffect(() => {
    if (step === 'verify' && mnemonic) {
      const indices = [];
      while (indices.length < 3) {
        const idx = Math.floor(Math.random() * 12);
        if (!indices.includes(idx)) indices.push(idx);
      }
      setVerifyIndices(indices.sort((a, b) => a - b));
      setSelectedWords([]);
    }
  }, [step, mnemonic]);

  const words = mnemonic.split(' ');

  const handleCopy = async () => {
    await Clipboard.setStringAsync(mnemonic);
    setCopied(true);
    Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleSetupNext = () => {
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
    setStep('mnemonic');
  };

  const handleMnemonicNext = () => {
    if (!copied) {
      Alert.alert('提示', '请先复制并安全保存助记词');
      return;
    }
    setStep('verify');
  };

  const handleVerifyWord = (wordIndex: number) => {
    if (selectedWords.includes(wordIndex)) {
      setSelectedWords(selectedWords.filter((i) => i !== wordIndex));
    } else if (selectedWords.length < 3) {
      setSelectedWords([...selectedWords, wordIndex]);
    }
  };

  const handleCreate = async () => {
    const correctOrder = verifyIndices.every(
      (idx, i) => selectedWords[i] === idx
    );

    if (!correctOrder || selectedWords.length !== 3) {
      Alert.alert('验证失败', '请按顺序选择正确的助记词');
      return;
    }

    setLoading(true);
    try {
      await createWallet(name, password);
      Haptics.notificationAsync(Haptics.NotificationFeedbackType.Success);
      router.replace('/(tabs)');
    } catch (error: any) {
      Alert.alert('创建失败', error.message);
    } finally {
      setLoading(false);
    }
  };

  const renderSetup = () => (
    <View style={styles.stepContent}>
      <View style={styles.iconContainer}>
        <Shield size={48} color={Colors.primary} />
      </View>
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        创建新钱包
      </Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        设置钱包名称和密码
      </Text>

      <View style={styles.form}>
        <Input
          label="钱包名称"
          placeholder="输入钱包名称"
          value={name}
          onChangeText={setName}
          maxLength={20}
        />
        <Input
          label="密码"
          placeholder="至少6位密码"
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

      <Button title="下一步" onPress={handleSetupNext} style={styles.button} />
    </View>
  );

  const renderMnemonic = () => (
    <View style={styles.stepContent}>
      <View style={styles.iconContainer}>
        <Eye size={48} color={Colors.warning} />
      </View>
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        备份助记词
      </Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        请按顺序抄写并安全保存，丢失将无法恢复
      </Text>

      <Card style={styles.mnemonicCard}>
        <View style={styles.mnemonicHeader}>
          <TouchableOpacity onPress={() => setShowMnemonic(!showMnemonic)}>
            {showMnemonic ? (
              <EyeOff size={20} color={colors.textSecondary} />
            ) : (
              <Eye size={20} color={colors.textSecondary} />
            )}
          </TouchableOpacity>
          <TouchableOpacity onPress={handleCopy} style={styles.copyButton}>
            {copied ? (
              <CheckCircle size={20} color={Colors.success} />
            ) : (
              <Copy size={20} color={colors.textSecondary} />
            )}
            <Text
              style={[
                styles.copyText,
                { color: copied ? Colors.success : colors.textSecondary },
              ]}
            >
              {copied ? '已复制' : '复制'}
            </Text>
          </TouchableOpacity>
        </View>

        <View style={styles.wordsGrid}>
          {words.map((word, index) => (
            <View
              key={index}
              style={[styles.wordItem, { backgroundColor: colors.surface }]}
            >
              <Text style={[styles.wordIndex, { color: colors.textTertiary }]}>
                {index + 1}
              </Text>
              <Text style={[styles.wordText, { color: colors.textPrimary }]}>
                {showMnemonic ? word : '••••'}
              </Text>
            </View>
          ))}
        </View>
      </Card>

      <View style={styles.warningBox}>
        <Text style={[styles.warningText, { color: Colors.warning }]}>
          ⚠️ 请勿截图或分享助记词，任何人获取助记词都能控制您的资产
        </Text>
      </View>

      <Button
        title="我已安全保存"
        onPress={handleMnemonicNext}
        style={styles.button}
        disabled={!copied}
      />
    </View>
  );

  const renderVerify = () => (
    <View style={styles.stepContent}>
      <View style={styles.iconContainer}>
        <CheckCircle size={48} color={Colors.success} />
      </View>
      <Text style={[styles.title, { color: colors.textPrimary }]}>
        验证助记词
      </Text>
      <Text style={[styles.subtitle, { color: colors.textSecondary }]}>
        请按顺序选择第 {verifyIndices.map((i) => i + 1).join('、')} 个词
      </Text>

      <View style={styles.selectedArea}>
        {verifyIndices.map((targetIdx, i) => (
          <View
            key={targetIdx}
            style={[
              styles.selectedSlot,
              {
                backgroundColor:
                  selectedWords[i] !== undefined
                    ? Colors.primary + '20'
                    : colors.surface,
                borderColor:
                  selectedWords[i] !== undefined
                    ? Colors.primary
                    : colors.border,
              },
            ]}
          >
            <Text style={[styles.slotLabel, { color: colors.textTertiary }]}>
              #{targetIdx + 1}
            </Text>
            <Text style={[styles.slotWord, { color: colors.textPrimary }]}>
              {selectedWords[i] !== undefined ? words[selectedWords[i]] : ''}
            </Text>
          </View>
        ))}
      </View>

      <View style={styles.wordsOptions}>
        {words.map((word, index) => {
          const isSelected = selectedWords.includes(index);
          return (
            <TouchableOpacity
              key={index}
              style={[
                styles.optionWord,
                {
                  backgroundColor: isSelected
                    ? Colors.primary
                    : colors.surface,
                  borderColor: isSelected ? Colors.primary : colors.border,
                },
              ]}
              onPress={() => handleVerifyWord(index)}
              disabled={isSelected && !selectedWords.includes(index)}
            >
              <Text
                style={[
                  styles.optionText,
                  { color: isSelected ? '#FFFFFF' : colors.textPrimary },
                ]}
              >
                {word}
              </Text>
            </TouchableOpacity>
          );
        })}
      </View>

      <Button
        title="确认创建"
        onPress={handleCreate}
        style={styles.button}
        loading={loading}
        disabled={selectedWords.length !== 3}
      />
    </View>
  );

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
      keyboardShouldPersistTaps="handled"
    >
      {/* Progress */}
      <View style={styles.progress}>
        {['setup', 'mnemonic', 'verify'].map((s, i) => (
          <View key={s} style={styles.progressItem}>
            <View
              style={[
                styles.progressDot,
                {
                  backgroundColor:
                    step === s || i < ['setup', 'mnemonic', 'verify'].indexOf(step)
                      ? Colors.primary
                      : colors.border,
                },
              ]}
            />
            {i < 2 && (
              <View
                style={[
                  styles.progressLine,
                  {
                    backgroundColor:
                      i < ['setup', 'mnemonic', 'verify'].indexOf(step)
                        ? Colors.primary
                        : colors.border,
                  },
                ]}
              />
            )}
          </View>
        ))}
      </View>

      {step === 'setup' && renderSetup()}
      {step === 'mnemonic' && renderMnemonic()}
      {step === 'verify' && renderVerify()}
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
  progress: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 32,
  },
  progressItem: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  progressDot: {
    width: 12,
    height: 12,
    borderRadius: 6,
  },
  progressLine: {
    width: 60,
    height: 2,
    marginHorizontal: 4,
  },
  stepContent: {
    alignItems: 'center',
  },
  iconContainer: {
    width: 80,
    height: 80,
    borderRadius: 40,
    backgroundColor: 'rgba(99, 102, 241, 0.1)',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    textAlign: 'center',
    marginBottom: 32,
  },
  form: {
    width: '100%',
  },
  button: {
    width: '100%',
    marginTop: 16,
  },
  mnemonicCard: {
    width: '100%',
    marginBottom: 16,
  },
  mnemonicHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  copyButton: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  copyText: {
    marginLeft: 4,
    fontSize: 14,
  },
  wordsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  wordItem: {
    width: '31%',
    flexDirection: 'row',
    alignItems: 'center',
    padding: 10,
    borderRadius: 8,
  },
  wordIndex: {
    fontSize: 12,
    marginRight: 6,
    width: 18,
  },
  wordText: {
    fontSize: 14,
    fontWeight: '500',
  },
  warningBox: {
    width: '100%',
    padding: 12,
    borderRadius: 8,
    backgroundColor: 'rgba(245, 158, 11, 0.1)',
    marginBottom: 16,
  },
  warningText: {
    fontSize: 13,
    lineHeight: 20,
  },
  selectedArea: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 24,
  },
  selectedSlot: {
    width: 100,
    height: 70,
    borderRadius: 12,
    borderWidth: 2,
    alignItems: 'center',
    justifyContent: 'center',
  },
  slotLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  slotWord: {
    fontSize: 16,
    fontWeight: '600',
  },
  wordsOptions: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
    justifyContent: 'center',
    marginBottom: 24,
  },
  optionWord: {
    paddingHorizontal: 16,
    paddingVertical: 10,
    borderRadius: 20,
    borderWidth: 1,
  },
  optionText: {
    fontSize: 14,
    fontWeight: '500',
  },
});
