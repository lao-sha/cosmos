import * as Clipboard from 'expo-clipboard';
import { useState } from 'react';
import { Pressable, StyleSheet, Text, View } from 'react-native';

interface MnemonicDisplayProps {
  mnemonic: string;
  showCopy?: boolean;
  blurred?: boolean;
}

export function MnemonicDisplay({
  mnemonic,
  showCopy = true,
  blurred = false,
}: MnemonicDisplayProps) {
  const [copied, setCopied] = useState(false);
  const [revealed, setRevealed] = useState(!blurred);
  const words = mnemonic.split(' ');

  const handleCopy = async () => {
    await Clipboard.setStringAsync(mnemonic);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>助记词</Text>
        <Text style={styles.subtitle}>请按顺序抄写并妥善保管</Text>
      </View>

      <Pressable
        style={styles.grid}
        onPress={() => blurred && setRevealed(!revealed)}
      >
        {words.map((word, index) => (
          <View key={index} style={styles.wordItem}>
            <Text style={styles.wordIndex}>{index + 1}</Text>
            <Text style={[styles.word, !revealed && styles.wordBlurred]}>
              {revealed ? word : '••••••'}
            </Text>
          </View>
        ))}
        {!revealed && (
          <View style={styles.blurOverlay}>
            <Text style={styles.blurText}>点击显示</Text>
          </View>
        )}
      </Pressable>

      {showCopy && (
        <Pressable style={styles.copyButton} onPress={handleCopy}>
          <Text style={styles.copyText}>
            {copied ? '✓ 已复制' : '📋 复制助记词'}
          </Text>
        </Pressable>
      )}

      <View style={styles.warning}>
        <Text style={styles.warningIcon}>⚠️</Text>
        <Text style={styles.warningText}>
          助记词是恢复钱包的唯一凭证，请勿截图、拍照或在线存储。任何获取你助记词的人都可以控制你的资产。
        </Text>
      </View>
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
    marginBottom: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  subtitle: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 4,
  },
  grid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    position: 'relative',
  },
  wordItem: {
    width: '31%',
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    paddingVertical: 10,
    paddingHorizontal: 8,
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  wordIndex: {
    fontSize: 11,
    color: '#9ca3af',
    width: 18,
  },
  word: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
    fontFamily: 'monospace',
  },
  wordBlurred: {
    color: '#d1d5db',
  },
  blurOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(255,255,255,0.8)',
    justifyContent: 'center',
    alignItems: 'center',
    borderRadius: 8,
  },
  blurText: {
    fontSize: 16,
    color: '#6D28D9',
    fontWeight: '600',
  },
  copyButton: {
    marginTop: 16,
    paddingVertical: 12,
    borderRadius: 8,
    backgroundColor: '#f3f4f6',
    alignItems: 'center',
  },
  copyText: {
    fontSize: 14,
    color: '#4b5563',
    fontWeight: '500',
  },
  warning: {
    flexDirection: 'row',
    marginTop: 16,
    padding: 12,
    backgroundColor: '#fef3c7',
    borderRadius: 8,
  },
  warningIcon: {
    fontSize: 16,
    marginRight: 8,
  },
  warningText: {
    flex: 1,
    fontSize: 12,
    color: '#92400e',
    lineHeight: 18,
  },
});
