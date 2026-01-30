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

const PALACES = ['命宫', '兄弟', '夫妻', '子女', '财帛', '疾厄', '迁移', '仆役', '官禄', '田宅', '福德', '父母'];
const MAIN_STARS = ['紫微', '天机', '太阳', '武曲', '天同', '廉贞', '天府', '太阴', '贪狼', '巨门', '天相', '天梁', '七杀', '破军'];

export default function ZiweiScreen() {
  const router = useRouter();
  const [birthDate, setBirthDate] = useState({
    year: '',
    month: '',
    day: '',
    hour: '',
  });
  const [gender, setGender] = useState<'male' | 'female' | null>(null);
  const [result, setResult] = useState<{
    mingGong: string;
    shenGong: string;
    mainStar: string;
    bodyStars: string[];
  } | null>(null);

  const handleCalculate = () => {
    const { year, month, day, hour } = birthDate;
    if (!year || !month || !day || !hour) {
      Alert.alert('提示', '请填写完整的出生时间');
      return;
    }
    if (!gender) {
      Alert.alert('提示', '请选择性别');
      return;
    }

    const mingGongIndex = (parseInt(month) + Math.floor((parseInt(hour) + 1) / 2)) % 12;
    const shenGongIndex = (parseInt(month) - Math.floor((parseInt(hour) + 1) / 2) + 24) % 12;
    const mainStarIndex = Math.floor(Math.random() * MAIN_STARS.length);
    
    const bodyStars = [];
    for (let i = 0; i < 3; i++) {
      bodyStars.push(MAIN_STARS[Math.floor(Math.random() * MAIN_STARS.length)]);
    }

    setResult({
      mingGong: PALACES[mingGongIndex],
      shenGong: PALACES[shenGongIndex],
      mainStar: MAIN_STARS[mainStarIndex],
      bodyStars: [...new Set(bodyStars)],
    });
  };

  const handleReset = () => {
    setBirthDate({ year: '', month: '', day: '', hour: '' });
    setGender(null);
    setResult(null);
  };

  const handleFindMaster = () => {
    router.push('/divination/masters/ziwei' as any);
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>⭐ 紫微斗数</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.introCard}>
          <Text style={styles.introTitle}>紫微斗数</Text>
          <Text style={styles.introDesc}>
            帝王之学，命宫推演。紫微斗数以紫微星为主，配合其他星曜，排布十二宫位，推算人生命运。
          </Text>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>📅 出生时间</Text>
          <Text style={styles.cardHint}>请输入农历出生时间</Text>
          
          <View style={styles.dateInputs}>
            <View style={styles.dateInputWrapper}>
              <TextInput
                style={styles.dateInput}
                placeholder="1990"
                placeholderTextColor="#9ca3af"
                value={birthDate.year}
                onChangeText={(v) => setBirthDate({ ...birthDate, year: v.replace(/[^0-9]/g, '') })}
                keyboardType="number-pad"
                maxLength={4}
              />
              <Text style={styles.dateLabel}>年</Text>
            </View>
            <View style={styles.dateInputWrapper}>
              <TextInput
                style={styles.dateInput}
                placeholder="01"
                placeholderTextColor="#9ca3af"
                value={birthDate.month}
                onChangeText={(v) => setBirthDate({ ...birthDate, month: v.replace(/[^0-9]/g, '') })}
                keyboardType="number-pad"
                maxLength={2}
              />
              <Text style={styles.dateLabel}>月</Text>
            </View>
            <View style={styles.dateInputWrapper}>
              <TextInput
                style={styles.dateInput}
                placeholder="01"
                placeholderTextColor="#9ca3af"
                value={birthDate.day}
                onChangeText={(v) => setBirthDate({ ...birthDate, day: v.replace(/[^0-9]/g, '') })}
                keyboardType="number-pad"
                maxLength={2}
              />
              <Text style={styles.dateLabel}>日</Text>
            </View>
            <View style={styles.dateInputWrapper}>
              <TextInput
                style={styles.dateInput}
                placeholder="12"
                placeholderTextColor="#9ca3af"
                value={birthDate.hour}
                onChangeText={(v) => setBirthDate({ ...birthDate, hour: v.replace(/[^0-9]/g, '') })}
                keyboardType="number-pad"
                maxLength={2}
              />
              <Text style={styles.dateLabel}>时</Text>
            </View>
          </View>
        </View>

        <View style={styles.inputCard}>
          <Text style={styles.cardTitle}>👤 性别</Text>
          <View style={styles.genderButtons}>
            <Pressable
              style={[styles.genderButton, gender === 'male' && styles.genderButtonSelected]}
              onPress={() => setGender('male')}
            >
              <Text style={[styles.genderButtonText, gender === 'male' && styles.genderButtonTextSelected]}>
                ♂ 男
              </Text>
            </Pressable>
            <Pressable
              style={[styles.genderButton, gender === 'female' && styles.genderButtonSelected]}
              onPress={() => setGender('female')}
            >
              <Text style={[styles.genderButtonText, gender === 'female' && styles.genderButtonTextSelected]}>
                ♀ 女
              </Text>
            </Pressable>
          </View>
        </View>

        {!result ? (
          <Pressable style={styles.calculateButton} onPress={handleCalculate}>
            <Text style={styles.calculateButtonText}>🔮 排盘分析</Text>
          </Pressable>
        ) : (
          <View style={styles.resultCard}>
            <Text style={styles.resultTitle}>紫微命盘</Text>
            
            <View style={styles.palaceDisplay}>
              <View style={styles.palaceItem}>
                <Text style={styles.palaceLabel}>命宫</Text>
                <Text style={styles.palaceValue}>{result.mingGong}</Text>
              </View>
              <View style={styles.palaceItem}>
                <Text style={styles.palaceLabel}>身宫</Text>
                <Text style={styles.palaceValue}>{result.shenGong}</Text>
              </View>
            </View>

            <View style={styles.starDisplay}>
              <Text style={styles.starTitle}>命宫主星</Text>
              <View style={styles.mainStarBox}>
                <Text style={styles.mainStarText}>⭐ {result.mainStar}</Text>
              </View>
            </View>

            <View style={styles.starDisplay}>
              <Text style={styles.starTitle}>命宫星曜</Text>
              <View style={styles.starsRow}>
                {result.bodyStars.map((star, index) => (
                  <View key={index} style={styles.starTag}>
                    <Text style={styles.starTagText}>{star}</Text>
                  </View>
                ))}
              </View>
            </View>

            <Text style={styles.resultHint}>
              💡 紫微命盘需要专业大师结合四化、大限详细解读
            </Text>

            <View style={styles.resultActions}>
              <Pressable style={styles.resetButton} onPress={handleReset}>
                <Text style={styles.resetButtonText}>重新排盘</Text>
              </Pressable>
              <Pressable style={styles.findMasterButton} onPress={handleFindMaster}>
                <Text style={styles.findMasterButtonText}>找大师解盘</Text>
              </Pressable>
            </View>
          </View>
        )}

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 排盘须知</Text>
          <Text style={styles.tipsText}>
            • 紫微斗数使用农历出生时间{'\n'}
            • 出生时辰对命盘影响很大{'\n'}
            • 需结合大限、流年综合分析{'\n'}
            • 命盘解读需专业大师指导
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
    marginBottom: 8,
  },
  cardHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 16,
  },
  dateInputs: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  dateInputWrapper: {
    alignItems: 'center',
    flex: 1,
    marginHorizontal: 4,
  },
  dateInput: {
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    padding: 12,
    fontSize: 18,
    fontWeight: '600',
    color: '#6D28D9',
    textAlign: 'center',
    width: '100%',
  },
  dateLabel: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 6,
  },
  genderButtons: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  genderButton: {
    flex: 1,
    padding: 16,
    borderRadius: 12,
    backgroundColor: '#f9fafb',
    alignItems: 'center',
    marginHorizontal: 4,
  },
  genderButtonSelected: {
    backgroundColor: '#6D28D9',
  },
  genderButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#6b7280',
  },
  genderButtonTextSelected: {
    color: '#fff',
  },
  calculateButton: {
    backgroundColor: '#6D28D9',
    marginHorizontal: 16,
    marginVertical: 16,
    padding: 18,
    borderRadius: 16,
    alignItems: 'center',
  },
  calculateButtonText: {
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
  palaceDisplay: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginBottom: 20,
  },
  palaceItem: {
    alignItems: 'center',
    marginHorizontal: 20,
    padding: 16,
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    minWidth: 100,
  },
  palaceLabel: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 4,
  },
  palaceValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#6D28D9',
  },
  starDisplay: {
    marginBottom: 16,
  },
  starTitle: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 8,
  },
  mainStarBox: {
    backgroundColor: '#fef3c7',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  mainStarText: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#92400e',
  },
  starsRow: {
    flexDirection: 'row',
    flexWrap: 'wrap',
  },
  starTag: {
    backgroundColor: '#f3f4f6',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    marginRight: 8,
    marginBottom: 8,
  },
  starTagText: {
    fontSize: 14,
    color: '#4b5563',
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
