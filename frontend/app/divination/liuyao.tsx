import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  Alert,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View
} from 'react-native';

const YAOLINES = ['初爻', '二爻', '三爻', '四爻', '五爻', '上爻'];

export default function LiuyaoScreen() {
  const router = useRouter();
  const [question, setQuestion] = useState('');
  const [shakeCount, setShakeCount] = useState(0);
  const [yaoResults, setYaoResults] = useState<number[]>([]);
  const [isShaking, setIsShaking] = useState(false);

  const handleShake = () => {
    if (shakeCount >= 6) return;
    if (!question.trim() && shakeCount === 0) {
      Alert.alert('提示', '请先输入您想问的问题');
      return;
    }

    setIsShaking(true);
    
    setTimeout(() => {
      const result = Math.floor(Math.random() * 4);
      setYaoResults([...yaoResults, result]);
      setShakeCount(shakeCount + 1);
      setIsShaking(false);
    }, 800);
  };

  const getYaoSymbol = (value: number) => {
    switch (value) {
      case 0: return '⚊';
      case 1: return '⚋';
      case 2: return '⚊○';
      case 3: return '⚋×';
      default: return '?';
    }
  };

  const getYaoName = (value: number) => {
    switch (value) {
      case 0: return '少阳';
      case 1: return '少阴';
      case 2: return '老阳';
      case 3: return '老阴';
      default: return '未知';
    }
  };

  const handleReset = () => {
    setQuestion('');
    setShakeCount(0);
    setYaoResults([]);
  };

  const handleFindMaster = () => {
    router.push('/divination/masters/liuyao' as any);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>🎲 六爻占卜</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.introCard}>
          <Text style={styles.introTitle}>六爻占卜</Text>
          <Text style={styles.introDesc}>
            摇卦断事，趋吉避凶。六爻占卜源于《周易》，通过摇卦得出六个爻位，组成卦象以断吉凶。
          </Text>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>📝 您的问题</Text>
          <TextInput
            style={styles.questionInput}
            placeholder="请输入您想问的问题..."
            placeholderTextColor="#9ca3af"
            value={question}
            onChangeText={setQuestion}
            multiline
            numberOfLines={3}
            editable={shakeCount === 0}
          />
        </View>

        <View style={styles.shakeCard}>
          <Text style={styles.cardTitle}>🎲 摇卦 ({shakeCount}/6)</Text>
          
          <View style={styles.yaoDisplay}>
            {[...Array(6)].map((_, index) => {
              const yaoIndex = 5 - index;
              const hasResult = yaoIndex < yaoResults.length;
              return (
                <View key={index} style={styles.yaoRow}>
                  <Text style={styles.yaoLabel}>{YAOLINES[yaoIndex]}</Text>
                  <View style={[styles.yaoLine, hasResult && styles.yaoLineActive]}>
                    {hasResult ? (
                      <>
                        <Text style={styles.yaoSymbol}>{getYaoSymbol(yaoResults[yaoIndex])}</Text>
                        <Text style={styles.yaoName}>{getYaoName(yaoResults[yaoIndex])}</Text>
                      </>
                    ) : (
                      <Text style={styles.yaoPlaceholder}>待摇</Text>
                    )}
                  </View>
                </View>
              );
            })}
          </View>

          {shakeCount < 6 ? (
            <Pressable
              style={[styles.shakeButton, isShaking && styles.shakeButtonDisabled]}
              onPress={handleShake}
              disabled={isShaking}
            >
              <Text style={styles.shakeButtonText}>
                {isShaking ? '🎲 摇卦中...' : `🎲 摇第${shakeCount + 1}爻`}
              </Text>
            </Pressable>
          ) : (
            <View style={styles.completeBox}>
              <Text style={styles.completeText}>✅ 六爻已成</Text>
            </View>
          )}
        </View>

        {shakeCount === 6 && (
          <View style={styles.resultCard}>
            <Text style={styles.resultTitle}>卦象已成</Text>
            <Text style={styles.resultHint}>
              💡 六爻卦象需要专业大师结合世应、六亲、六神等详细解读
            </Text>

            <View style={styles.resultActions}>
              <Pressable style={styles.resetButton} onPress={handleReset}>
                <Text style={styles.resetButtonText}>重新摇卦</Text>
              </Pressable>
              <Pressable style={styles.findMasterButton} onPress={handleFindMaster}>
                <Text style={styles.findMasterButtonText}>找大师解卦</Text>
              </Pressable>
            </View>
          </View>
        )}

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 摇卦须知</Text>
          <Text style={styles.tipsText}>
            • 心诚则灵，摇卦前请静心默念问题{'\n'}
            • 一事一问，问题要明确具体{'\n'}
            • 每次点击代表摇三枚铜钱{'\n'}
            • 从初爻到上爻依次摇出
          </Text>
        </View>

        <View style={styles.bottomPadding} />
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
    backgroundColor: '#6D28D9',
    paddingTop: 50,
    paddingBottom: 16,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    color: '#fff',
    fontSize: 18,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  headerRight: {
    width: 40,
  },
  content: {
    flex: 1,
  },
  introCard: {
    backgroundColor: '#fff',
    margin: 16,
    marginBottom: 8,
    padding: 20,
    borderRadius: 16,
  },
  introTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1f2937',
    marginBottom: 8,
  },
  introDesc: {
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 22,
  },
  inputCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  questionInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 16,
    fontSize: 15,
    color: '#1f2937',
    minHeight: 80,
    textAlignVertical: 'top',
  },
  shakeCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  yaoDisplay: {
    marginBottom: 16,
  },
  yaoRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 4,
  },
  yaoLabel: {
    width: 50,
    fontSize: 14,
    color: '#6b7280',
  },
  yaoLine: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#f9fafb',
    padding: 12,
    borderRadius: 8,
  },
  yaoLineActive: {
    backgroundColor: '#fef3c7',
  },
  yaoSymbol: {
    fontSize: 20,
    color: '#1f2937',
    marginRight: 12,
  },
  yaoName: {
    fontSize: 14,
    color: '#92400e',
    fontWeight: '500',
  },
  yaoPlaceholder: {
    fontSize: 14,
    color: '#9ca3af',
  },
  shakeButton: {
    backgroundColor: '#6D28D9',
    padding: 18,
    borderRadius: 16,
    alignItems: 'center',
  },
  shakeButtonDisabled: {
    backgroundColor: '#a78bfa',
  },
  shakeButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: '600',
  },
  completeBox: {
    backgroundColor: '#dcfce7',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  completeText: {
    fontSize: 16,
    color: '#166534',
    fontWeight: '600',
  },
  resultCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  resultTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#1f2937',
    textAlign: 'center',
    marginBottom: 12,
  },
  resultHint: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 16,
  },
  resultActions: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  resetButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#6D28D9',
    alignItems: 'center',
    marginRight: 8,
  },
  resetButtonText: {
    color: '#6D28D9',
    fontSize: 15,
    fontWeight: '600',
  },
  findMasterButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    marginLeft: 8,
  },
  findMasterButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  tipsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginVertical: 8,
    padding: 20,
    borderRadius: 16,
  },
  tipsTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  tipsText: {
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 24,
  },
  bottomPadding: {
    height: 40,
  },
});
