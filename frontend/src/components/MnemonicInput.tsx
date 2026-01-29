import { WalletService } from '@/src/lib/wallet';
import { useState } from 'react';
import { Pressable, StyleSheet, Text, TextInput, View } from 'react-native';

interface MnemonicInputProps {
  onValidMnemonic: (mnemonic: string) => void;
  onInvalid?: () => void;
}

export function MnemonicInput({ onValidMnemonic, onInvalid }: MnemonicInputProps) {
  const [words, setWords] = useState<string[]>(Array(12).fill(''));
  const [error, setError] = useState<string | null>(null);
  const [pasteMode, setPasteMode] = useState(false);
  const [pasteInput, setPasteInput] = useState('');

  const handleWordChange = (index: number, value: string) => {
    const newWords = [...words];
    newWords[index] = value.toLowerCase().trim();
    setWords(newWords);
    setError(null);
  };

  const handlePaste = () => {
    const pastedWords = pasteInput
      .toLowerCase()
      .trim()
      .split(/\s+/)
      .filter((w) => w.length > 0);

    if (pastedWords.length === 12) {
      setWords(pastedWords);
      setPasteMode(false);
      setPasteInput('');
      setError(null);
    } else {
      setError(`需要12个单词，当前输入了${pastedWords.length}个`);
    }
  };

  const handleValidate = async () => {
    const mnemonic = words.join(' ');
    
    if (words.some((w) => !w)) {
      setError('请填写所有单词');
      onInvalid?.();
      return;
    }

    await WalletService.init();
    const isValid = WalletService.validateMnemonic(mnemonic);

    if (isValid) {
      setError(null);
      onValidMnemonic(mnemonic);
    } else {
      setError('助记词无效，请检查拼写');
      onInvalid?.();
    }
  };

  const filledCount = words.filter((w) => w.length > 0).length;

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>输入助记词</Text>
        <Pressable onPress={() => setPasteMode(!pasteMode)}>
          <Text style={styles.pasteToggle}>
            {pasteMode ? '逐词输入' : '粘贴全部'}
          </Text>
        </Pressable>
      </View>

      {pasteMode ? (
        <View style={styles.pasteContainer}>
          <TextInput
            style={styles.pasteInput}
            placeholder="粘贴12个助记词，用空格分隔"
            placeholderTextColor="#9ca3af"
            multiline
            value={pasteInput}
            onChangeText={setPasteInput}
            autoCapitalize="none"
            autoCorrect={false}
          />
          <Pressable style={styles.pasteButton} onPress={handlePaste}>
            <Text style={styles.pasteButtonText}>确认粘贴</Text>
          </Pressable>
        </View>
      ) : (
        <View style={styles.grid}>
          {words.map((word, index) => (
            <View key={index} style={styles.inputItem}>
              <Text style={styles.inputIndex}>{index + 1}</Text>
              <TextInput
                style={styles.input}
                value={word}
                onChangeText={(v) => handleWordChange(index, v)}
                placeholder="单词"
                placeholderTextColor="#d1d5db"
                autoCapitalize="none"
                autoCorrect={false}
                returnKeyType={index < 11 ? 'next' : 'done'}
              />
            </View>
          ))}
        </View>
      )}

      <Text style={styles.progress}>
        已输入 {filledCount}/12 个单词
      </Text>

      {error && (
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>❌ {error}</Text>
        </View>
      )}

      <Pressable
        style={[styles.validateButton, filledCount < 12 && styles.validateButtonDisabled]}
        onPress={handleValidate}
        disabled={filledCount < 12}
      >
        <Text style={styles.validateButtonText}>验证并导入</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  pasteToggle: {
    fontSize: 14,
    color: '#6D28D9',
    fontWeight: '500',
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  inputItem: {
    width: '31%',
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    paddingLeft: 8,
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  inputIndex: {
    fontSize: 11,
    color: '#9ca3af',
    width: 18,
  },
  input: {
    flex: 1,
    paddingVertical: 10,
    paddingRight: 8,
    fontSize: 14,
    color: '#1f2937',
    fontFamily: 'monospace',
  },
  pasteContainer: {
    gap: 12,
  },
  pasteInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 16,
    fontSize: 15,
    color: '#1f2937',
    minHeight: 120,
    textAlignVertical: 'top',
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  pasteButton: {
    backgroundColor: '#f3f4f6',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  pasteButtonText: {
    fontSize: 14,
    color: '#4b5563',
    fontWeight: '500',
  },
  progress: {
    marginTop: 12,
    fontSize: 13,
    color: '#6b7280',
    textAlign: 'center',
  },
  errorContainer: {
    marginTop: 12,
    padding: 12,
    backgroundColor: '#fef2f2',
    borderRadius: 8,
  },
  errorText: {
    fontSize: 13,
    color: '#dc2626',
  },
  validateButton: {
    marginTop: 20,
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
  },
  validateButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  validateButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
