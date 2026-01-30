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

const TRIGRAMS = ['☰', '☱', '☲', '☳', '☴', '☵', '☶', '☷'];
const TRIGRAM_NAMES = ['乾', '兑', '离', '震', '巽', '坎', '艮', '坤'];

export default function MeihuaScreen() {
  const router = useRouter();
  const [question, setQuestion] = useState('');
  const [numbers, setNumbers] = useState(['', '', '']);
  const [result, setResult] = useState<{
    upperTrigram: number;
    lowerTrigram: number;
    changingLine: number;
  } | null>(null);

  const handleNumberChange = (index: number, value: string) => {
    const newNumbers = [...numbers];
    newNumbers[index] = value.replace(/[^0-9]/g, '');
    setNumbers(newNumbers);
  };

  const handleDivine = () => {
    if (!question.trim()) {
      Alert.alert('提示', '请先输入您想问的问题');
      return;
    }

    const nums = numbers.map(n => parseInt(n) || Math.floor(Math.random() * 100) + 1);
    
    const upperTrigram = (nums[0] - 1) % 8;
    const lowerTrigram = (nums[1] - 1) % 8;
    const changingLine = ((nums[0] + nums[1] + nums[2]) - 1) % 6 + 1;

    setResult({
      upperTrigram,
      lowerTrigram,
      changingLine,
    });
  };

  const handleReset = () => {
    setQuestion('');
    setNumbers(['', '', '']);
    setResult(null);
  };

  const handleFindMaster = () => {
    router.push('/divination/masters/meihua' as any);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>🌸 梅花易数</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.introCard}>
          <Text style={styles.introTitle}>梅花易数</Text>
          <Text style={styles.introDesc}>
            以数起卦，灵活应变。梅花易数是宋代邵雍所创，以先天八卦数理为基础，通过数字起卦，简便灵活。
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
          />
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>🔢 起卦数字</Text>
          <Text style={styles.cardHint}>输入三个数字，或留空随机生成</Text>
          <View style={styles.numberInputs}>
            {numbers.map((num, index) => (
              <View key={index} style={styles.numberInputWrapper}>
                <TextInput
                  style={styles.numberInput}
                  placeholder="?"
                  placeholderTextColor="#9ca3af"
                  value={num}
                  onChangeText={(v) => handleNumberChange(index, v)}
                  keyboardType="number-pad"
                  maxLength={3}
                />
                <Text style={styles.numberLabel}>
                  {index === 0 ? '上卦数' : index === 1 ? '下卦数' : '动爻数'}
                </Text>
              </View>
            ))}
          </View>
        </View>

        {!result ? (
          <Pressable style={styles.divineButton} onPress={handleDivine}>
            <Text style={styles.divineButtonText}>🔮 开始起卦</Text>
          </Pressable>
        ) : (
          <View style={styles.resultCard}>
            <Text style={styles.resultTitle}>卦象结果</Text>
            
            <View style={styles.hexagramDisplay}>
              <View style={styles.trigramBox}>
                <Text style={styles.trigramSymbol}>{TRIGRAMS[result.upperTrigram]}</Text>
                <Text style={styles.trigramName}>{TRIGRAM_NAMES[result.upperTrigram]}卦</Text>
                <Text style={styles.trigramLabel}>上卦</Text>
              </View>
              <View style={styles.trigramBox}>
                <Text style={styles.trigramSymbol}>{TRIGRAMS[result.lowerTrigram]}</Text>
                <Text style={styles.trigramName}>{TRIGRAM_NAMES[result.lowerTrigram]}卦</Text>
                <Text style={styles.trigramLabel}>下卦</Text>
              </View>
            </View>

            <View style={styles.changingLineBox}>
              <Text style={styles.changingLineText}>动爻：第 {result.changingLine} 爻</Text>
            </View>

            <Text style={styles.resultHint}>
              💡 此卦象需要专业大师为您详细解读
            </Text>

            <View style={styles.resultActions}>
              <Pressable style={styles.resetButton} onPress={handleReset}>
                <Text style={styles.resetButtonText}>重新起卦</Text>
              </Pressable>
              <Pressable style={styles.findMasterButton} onPress={handleFindMaster}>
                <Text style={styles.findMasterButtonText}>找大师解卦</Text>
              </Pressable>
            </View>
          </View>
        )}

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 起卦须知</Text>
          <Text style={styles.tipsText}>
            • 心诚则灵，起卦前请静心片刻{'\n'}
            • 一事一问，问题要明确具体{'\n'}
            • 同一问题不宜反复起卦{'\n'}
            • 卦象解读需结合具体情况
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
  cardHint: {
    fontSize: 12,
    color: '#9ca3af',
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
  numberInputs: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  numberInputWrapper: {
    alignItems: 'center',
    flex: 1,
    marginHorizontal: 4,
  },
  numberInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 16,
    fontSize: 24,
    fontWeight: 'bold',
    color: '#6D28D9',
    textAlign: 'center',
    width: '100%',
  },
  numberLabel: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 8,
  },
  divineButton: {
    backgroundColor: '#6D28D9',
    marginHorizontal: 16,
    marginVertical: 16,
    padding: 18,
    borderRadius: 16,
    alignItems: 'center',
  },
  divineButtonText: {
    color: '#fff',
    fontSize: 18,
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
    marginBottom: 20,
  },
  hexagramDisplay: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginBottom: 20,
  },
  trigramBox: {
    alignItems: 'center',
    marginHorizontal: 20,
    padding: 16,
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    minWidth: 100,
  },
  trigramSymbol: {
    fontSize: 48,
    marginBottom: 8,
  },
  trigramName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  trigramLabel: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 4,
  },
  changingLineBox: {
    backgroundColor: '#fef3c7',
    padding: 12,
    borderRadius: 8,
    alignItems: 'center',
    marginBottom: 16,
  },
  changingLineText: {
    fontSize: 14,
    color: '#92400e',
    fontWeight: '500',
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
