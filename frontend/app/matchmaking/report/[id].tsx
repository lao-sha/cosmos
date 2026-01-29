import { useLocalSearchParams, useRouter } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

interface ScoreDetail {
  dayPillar: number;
  wuxing: number;
  personality: number;
  shensha: number;
  dayun: number;
}

interface CompatibilityReport {
  id: string;
  overallScore: number;
  scoreDetail: ScoreDetail;
  recommendation: 'excellent' | 'good' | 'average' | 'poor';
  partyA: { name: string; birthDate: string };
  partyB: { name: string; birthDate: string };
  generatedAt: string;
}

const MOCK_REPORT: CompatibilityReport = {
  id: '3',
  overallScore: 78,
  scoreDetail: {
    dayPillar: 85,
    wuxing: 72,
    personality: 80,
    shensha: 75,
    dayun: 78,
  },
  recommendation: 'good',
  partyA: { name: '我', birthDate: '1990-05-15 08:30' },
  partyB: { name: '王五', birthDate: '1992-08-20 14:00' },
  generatedAt: '2025-01-20',
};

const RECOMMENDATION_INFO = {
  excellent: { label: '上上婚', color: '#22c55e', desc: '天作之合，大吉大利' },
  good: { label: '上等婚', color: '#3b82f6', desc: '良缘佳偶，幸福美满' },
  average: { label: '中等婚', color: '#f59e0b', desc: '平淡之合，需要磨合' },
  poor: { label: '下等婚', color: '#ef4444', desc: '不太相配，需慎重考虑' },
};

const SCORE_LABELS = {
  dayPillar: '日柱合婚',
  wuxing: '五行互补',
  personality: '性格匹配',
  shensha: '神煞吉凶',
  dayun: '大运流年',
};

export default function ReportScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();

  const report = MOCK_REPORT;
  const recInfo = RECOMMENDATION_INFO[report.recommendation];

  const getScoreColor = (score: number) => {
    if (score >= 80) return '#22c55e';
    if (score >= 60) return '#3b82f6';
    if (score >= 40) return '#f59e0b';
    return '#ef4444';
  };

  const renderScoreBar = (label: string, score: number) => (
    <View style={styles.scoreRow} key={label}>
      <Text style={styles.scoreLabel}>{label}</Text>
      <View style={styles.scoreBarContainer}>
        <View
          style={[
            styles.scoreBar,
            { width: `${score}%`, backgroundColor: getScoreColor(score) },
          ]}
        />
      </View>
      <Text style={[styles.scoreValue, { color: getScoreColor(score) }]}>
        {score}
      </Text>
    </View>
  );

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>合婚报告</Text>
        <Pressable style={styles.shareButton}>
          <Text style={styles.shareText}>分享</Text>
        </Pressable>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.mainScoreCard}>
          <View style={styles.partiesInfo}>
            <View style={styles.partyInfo}>
              <Text style={styles.partyName}>{report.partyA.name}</Text>
              <Text style={styles.partyBirth}>{report.partyA.birthDate}</Text>
            </View>
            <View style={styles.heartIcon}>
              <Text style={styles.heartEmoji}>💕</Text>
            </View>
            <View style={styles.partyInfo}>
              <Text style={styles.partyName}>{report.partyB.name}</Text>
              <Text style={styles.partyBirth}>{report.partyB.birthDate}</Text>
            </View>
          </View>

          <View style={styles.overallScoreContainer}>
            <View style={[styles.scoreCircle, { borderColor: recInfo.color }]}>
              <Text style={[styles.overallScore, { color: recInfo.color }]}>
                {report.overallScore}
              </Text>
              <Text style={styles.scoreUnit}>分</Text>
            </View>
          </View>

          <View style={[styles.recommendationBadge, { backgroundColor: `${recInfo.color}15` }]}>
            <Text style={[styles.recommendationText, { color: recInfo.color }]}>
              {recInfo.label}
            </Text>
          </View>
          <Text style={styles.recommendationDesc}>{recInfo.desc}</Text>
        </View>

        <View style={styles.detailCard}>
          <Text style={styles.cardTitle}>详细评分</Text>
          {Object.entries(report.scoreDetail).map(([key, value]) =>
            renderScoreBar(SCORE_LABELS[key as keyof typeof SCORE_LABELS], value)
          )}
        </View>

        <View style={styles.detailCard}>
          <Text style={styles.cardTitle}>日柱合婚分析</Text>
          <Text style={styles.analysisText}>
            男方日柱为【甲午】，女方日柱为【庚子】。甲木与庚金相克，但午火与子水相冲又相合，
            形成"天克地冲"的格局。虽有冲克，但也意味着双方性格互补，能够相互吸引。
            建议双方在日常相处中多包容、多沟通，化解矛盾。
          </Text>
        </View>

        <View style={styles.detailCard}>
          <Text style={styles.cardTitle}>五行互补分析</Text>
          <Text style={styles.analysisText}>
            男方八字以木火为用神，女方八字以金水为用神。双方用神互不冲突，
            且男方缺金、女方缺木，正好形成互补。从五行角度看，双方结合有利于各自运势的提升。
          </Text>
        </View>

        <View style={styles.detailCard}>
          <Text style={styles.cardTitle}>性格匹配分析</Text>
          <Text style={styles.analysisText}>
            男方性格偏向外向、果断，具有较强的领导力和执行力。
            女方性格偏向内敛、细腻，善于观察和照顾他人。
            两种性格可以形成良好的互补关系，男方主外、女方主内，分工明确。
          </Text>
        </View>

        <View style={styles.disclaimer}>
          <Text style={styles.disclaimerTitle}>⚠️ 免责声明</Text>
          <Text style={styles.disclaimerText}>
            本报告基于传统命理学理论生成，仅供参考娱乐，不应作为人生重大决策的唯一依据。
            婚姻幸福与否取决于双方的共同努力，请理性看待命理分析结果。
          </Text>
        </View>

        <Text style={styles.generatedAt}>
          报告生成于 {report.generatedAt}
        </Text>
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
  shareButton: {
    padding: 4,
  },
  shareText: {
    fontSize: 15,
    color: '#6D28D9',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  mainScoreCard: {
    backgroundColor: '#fff',
    borderRadius: 20,
    padding: 24,
    alignItems: 'center',
    marginBottom: 16,
  },
  partiesInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 20,
    marginBottom: 24,
  },
  partyInfo: {
    alignItems: 'center',
  },
  partyName: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  partyBirth: {
    fontSize: 12,
    color: '#6b7280',
    marginTop: 4,
  },
  heartIcon: {
    padding: 8,
  },
  heartEmoji: {
    fontSize: 28,
  },
  overallScoreContainer: {
    marginBottom: 16,
  },
  scoreCircle: {
    width: 120,
    height: 120,
    borderRadius: 60,
    borderWidth: 6,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#fff',
  },
  overallScore: {
    fontSize: 48,
    fontWeight: '700',
  },
  scoreUnit: {
    fontSize: 14,
    color: '#6b7280',
  },
  recommendationBadge: {
    paddingHorizontal: 20,
    paddingVertical: 8,
    borderRadius: 20,
    marginBottom: 8,
  },
  recommendationText: {
    fontSize: 18,
    fontWeight: '700',
  },
  recommendationDesc: {
    fontSize: 14,
    color: '#6b7280',
  },
  detailCard: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
  },
  scoreRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  scoreLabel: {
    width: 80,
    fontSize: 14,
    color: '#374151',
  },
  scoreBarContainer: {
    flex: 1,
    height: 8,
    backgroundColor: '#e5e7eb',
    borderRadius: 4,
    marginHorizontal: 12,
    overflow: 'hidden',
  },
  scoreBar: {
    height: '100%',
    borderRadius: 4,
  },
  scoreValue: {
    width: 32,
    fontSize: 14,
    fontWeight: '600',
    textAlign: 'right',
  },
  analysisText: {
    fontSize: 14,
    color: '#4b5563',
    lineHeight: 22,
  },
  disclaimer: {
    backgroundColor: '#fef3c7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
  },
  disclaimerTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#92400e',
    marginBottom: 8,
  },
  disclaimerText: {
    fontSize: 13,
    color: '#78350f',
    lineHeight: 20,
  },
  generatedAt: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'center',
    marginBottom: 32,
  },
});
