/**
 * 星尘玄鉴 - 验证助记词页面
 * 步骤3: 让用户按顺序选择助记词以验证正确备份
 * 主题色：金棕色 #B2955D
 */

import { useState, useMemo } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Pressable,
  ScrollView,
  Alert,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_COLOR_LIGHT = '#F7D3A1';
const THEME_BG = '#F5F5F7';

interface VerifyMnemonicPageProps {
  mnemonic: string;
  onVerifySuccess: () => void;
  onBack: () => void;
}

export default function VerifyMnemonicPage({
  mnemonic,
  onVerifySuccess,
  onBack,
}: VerifyMnemonicPageProps) {
  const router = useRouter();
  const words = mnemonic.split(' ');

  // 随机选择 4 个位置让用户填写
  const verifyIndices = useMemo(() => {
    const indices = Array.from({ length: 12 }, (_, i) => i);
    const shuffled = indices.sort(() => Math.random() - 0.5);
    return shuffled.slice(0, 4).sort((a, b) => a - b);
  }, []);

  // 打乱的单词选项（只包含需要验证的 4 个单词）
  const shuffledOptions = useMemo(() => {
    return verifyIndices
      .map((i) => words[i])
      .sort(() => Math.random() - 0.5);
  }, [verifyIndices, words]);

  const [selectedWords, setSelectedWords] = useState<(string | null)[]>([null, null, null, null]);
  const [currentSlot, setCurrentSlot] = useState(0);

  const handleSelectWord = (word: string) => {
    if (selectedWords.includes(word)) {
      // 已选择的单词，取消选择
      const index = selectedWords.indexOf(word);
      const newSelected = [...selectedWords];
      newSelected[index] = null;
      setSelectedWords(newSelected);
      setCurrentSlot(index);
    } else {
      // 选择新单词
      const newSelected = [...selectedWords];
      newSelected[currentSlot] = word;
      setSelectedWords(newSelected);

      // 移动到下一个空槽
      const nextEmpty = newSelected.findIndex((w, i) => i > currentSlot && w === null);
      if (nextEmpty !== -1) {
        setCurrentSlot(nextEmpty);
      } else {
        const firstEmpty = newSelected.findIndex((w) => w === null);
        if (firstEmpty !== -1) {
          setCurrentSlot(firstEmpty);
        }
      }
    }
  };

  const handleClearSlot = (index: number) => {
    const newSelected = [...selectedWords];
    newSelected[index] = null;
    setSelectedWords(newSelected);
    setCurrentSlot(index);
  };

  const handleVerify = () => {
    const allFilled = selectedWords.every((w) => w !== null);
    if (!allFilled) {
      Alert.alert('提示', '请填写所有空位');
      return;
    }

    // 验证是否正确
    const isCorrect = verifyIndices.every(
      (wordIndex, slotIndex) => selectedWords[slotIndex] === words[wordIndex]
    );

    if (isCorrect) {
      onVerifySuccess();
    } else {
      Alert.alert('验证失败', '助记词顺序不正确，请重试', [
        {
          text: '重试',
          onPress: () => setSelectedWords([null, null, null, null]),
        },
      ]);
    }
  };

  const allFilled = selectedWords.every((w) => w !== null);

  return (
    <View style={styles.wrapper}>
      <ScrollView style={styles.container} contentContainerStyle={styles.content} showsVerticalScrollIndicator={false}>
      {/* 返回按钮 */}
      <Pressable style={styles.backButton} onPress={onBack}>
        <Ionicons name="chevron-back" size={24} color="#333" />
      </Pressable>

      {/* 标题 */}
      <Text style={styles.title}>验证助记词</Text>
      <Text style={styles.subtitle}>
        请按顺序点击选择正确的单词，以确认您已正确备份
      </Text>

      {/* 助记词展示区（带空位） */}
      <View style={styles.mnemonicCard}>
        <View style={styles.mnemonicGrid}>
          {words.map((word, index) => {
            const verifySlotIndex = verifyIndices.indexOf(index);
            const isVerifySlot = verifySlotIndex !== -1;
            const selectedWord = isVerifySlot ? selectedWords[verifySlotIndex] : null;

            return (
              <Pressable
                key={index}
                style={[
                  styles.wordItem,
                  isVerifySlot && styles.wordItemEmpty,
                  isVerifySlot && currentSlot === verifySlotIndex && styles.wordItemActive,
                  isVerifySlot && selectedWord && styles.wordItemFilled,
                ]}
                onPress={() => {
                  if (isVerifySlot) {
                    if (selectedWord) {
                      handleClearSlot(verifySlotIndex);
                    } else {
                      setCurrentSlot(verifySlotIndex);
                    }
                  }
                }}
              >
                <Text style={styles.wordIndex}>{index + 1}</Text>
                <Text
                  style={[
                    styles.wordText,
                    isVerifySlot && !selectedWord && styles.wordTextEmpty,
                  ]}
                >
                  {isVerifySlot ? selectedWord || '?' : word}
                </Text>
              </Pressable>
            );
          })}
        </View>
      </View>

      {/* 提示 */}
      <Text style={styles.hint}>
        点击下方单词填入第 {verifyIndices.map((i) => i + 1).join(', ')} 位
      </Text>

      {/* 可选单词 */}
      <View style={styles.optionsContainer}>
        {shuffledOptions.map((word, index) => {
          const isSelected = selectedWords.includes(word);
          return (
            <Pressable
              key={index}
              style={[styles.optionButton, isSelected && styles.optionButtonSelected]}
              onPress={() => handleSelectWord(word)}
            >
              <Text
                style={[styles.optionText, isSelected && styles.optionTextSelected]}
              >
                {word}
              </Text>
            </Pressable>
          );
        })}
      </View>

      {/* 验证按钮 */}
      <Pressable
        style={[styles.primaryButton, !allFilled && styles.disabledButton]}
        onPress={handleVerify}
      >
        <Text style={styles.primaryButtonText}>验证并完成</Text>
      </Pressable>
    </ScrollView>

      {/* 底部导航 - 全局统一 */}
      <View style={styles.bottomNav}>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/' as any)}>
          <Text style={styles.bottomNavIcon}>🏠</Text>
          <Text style={styles.bottomNavLabel}>首页</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/divination' as any)}>
          <Text style={styles.bottomNavIcon}>🧭</Text>
          <Text style={styles.bottomNavLabel}>占卜</Text>
        </Pressable>
        <Pressable style={styles.bottomNavItem} onPress={() => router.push('/chat' as any)}>
          <Text style={styles.bottomNavIcon}>💬</Text>
          <Text style={styles.bottomNavLabel}>消息</Text>
        </Pressable>
        <Pressable style={[styles.bottomNavItem, styles.bottomNavItemActive]} onPress={() => router.push('/profile' as any)}>
          <Text style={styles.bottomNavIcon}>👤</Text>
          <Text style={[styles.bottomNavLabel, styles.bottomNavLabelActive]}>我的</Text>
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: THEME_BG,
    maxWidth: 414,
    width: '100%',
    alignSelf: 'center',
  },
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  content: {
    padding: 16,
    paddingTop: 60,
    paddingBottom: 40,
  },
  backButton: {
    width: 40,
    height: 40,
    justifyContent: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    color: '#333',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#999',
    marginBottom: 20,
    lineHeight: 22,
  },
  mnemonicCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  mnemonicGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  wordItem: {
    width: '30%',
    backgroundColor: '#FAFAFA',
    borderRadius: 8,
    padding: 10,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    borderWidth: 1,
    borderColor: '#E8E8E8',
  },
  wordItemEmpty: {
    backgroundColor: '#FFF',
    borderWidth: 2,
    borderColor: '#D9D9D9',
    borderStyle: 'dashed',
  },
  wordItemActive: {
    borderColor: THEME_COLOR,
    borderStyle: 'solid',
  },
  wordItemFilled: {
    backgroundColor: '#E8F5E9',
    borderColor: '#27AE60',
    borderStyle: 'solid',
  },
  wordIndex: {
    fontSize: 12,
    color: '#999',
    width: 18,
  },
  wordText: {
    fontSize: 14,
    color: '#333',
    fontWeight: '500',
  },
  wordTextEmpty: {
    color: '#999',
  },
  hint: {
    fontSize: 13,
    color: '#999',
    textAlign: 'center',
    marginBottom: 16,
  },
  optionsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'center',
    gap: 10,
    marginBottom: 24,
  },
  optionButton: {
    backgroundColor: '#FFF',
    paddingHorizontal: 18,
    paddingVertical: 10,
    borderRadius: 20,
    borderWidth: 1,
    borderColor: '#E8E8E8',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.04,
    shadowRadius: 4,
    elevation: 1,
  },
  optionButtonSelected: {
    backgroundColor: '#E8F5E9',
    borderColor: '#27AE60',
  },
  optionText: {
    fontSize: 15,
    color: '#333',
    fontWeight: '500',
  },
  optionTextSelected: {
    color: '#27AE60',
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    paddingVertical: 16,
    borderRadius: 8,
    alignItems: 'center',
  },
  primaryButtonText: {
    fontSize: 17,
    fontWeight: '600',
    color: '#FFF',
  },
  disabledButton: {
    opacity: 0.6,
  },
  bottomNav: {
    position: 'absolute',
    bottom: 0,
    left: '50%',
    transform: [{ translateX: -207 }],
    width: 414,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    backgroundColor: '#FFF',
    paddingTop: 8,
    paddingBottom: 8,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  bottomNavItem: {
    alignItems: 'center',
    paddingVertical: 4,
    flex: 1,
  },
  bottomNavItemActive: {},
  bottomNavIcon: {
    fontSize: 22,
    marginBottom: 2,
  },
  bottomNavLabel: {
    fontSize: 12,
    color: '#999',
    fontWeight: '500',
  },
  bottomNavLabelActive: {
    color: THEME_COLOR,
  },
});
