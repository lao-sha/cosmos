import { chainService, FullBaziChartForApi } from '@/src/services/chain';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useCallback, useEffect, useState } from 'react';
import {
  ActivityIndicator,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from 'react-native';

export default function BaziDetailScreen() {
  const router = useRouter();
  const { id } = useLocalSearchParams<{ id: string }>();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();

  const [loading, setLoading] = useState(true);
  const [chartData, setChartData] = useState<FullBaziChartForApi | null>(null);
  const [error, setError] = useState<string | null>(null);

  const chartId = parseInt(id || '0');

  // 获取命盘详情
  const fetchChartDetail = useCallback(async () => {
    if (!isConnected || !chartId) return;

    setLoading(true);
    setError(null);
    try {
      const data = await chainService.getBaziChart(chartId);
      console.log('命盘详情数据:', JSON.stringify(data, null, 2));
      if (data) {
        setChartData(data);
      } else {
        setError('命盘不存在或已被删除');
      }
    } catch (e: any) {
      console.error('获取命盘详情失败:', e);
      setError(e.message || '获取命盘详情失败');
    } finally {
      setLoading(false);
    }
  }, [chartId, isConnected]);

  useEffect(() => {
    fetchChartDetail();
  }, [fetchChartDetail]);

  // 加载中
  if (loading) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>命盘详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6D28D9" />
          <Text style={styles.loadingText}>加载中...</Text>
        </View>
      </View>
    );
  }

  // 错误状态
  if (error || !chartData) {
    return (
      <View style={styles.container}>
        <View style={styles.header}>
          <Pressable style={styles.backButton} onPress={() => router.back()}>
            <Text style={styles.backText}>‹ 返回</Text>
          </Pressable>
          <Text style={styles.headerTitle}>命盘详情</Text>
          <View style={styles.headerRight} />
        </View>
        <View style={styles.errorContainer}>
          <Text style={styles.errorIcon}>😔</Text>
          <Text style={styles.errorTitle}>加载失败</Text>
          <Text style={styles.errorDesc}>{error || '命盘数据不存在'}</Text>
          <Pressable style={styles.retryButton} onPress={fetchChartDetail}>
            <Text style={styles.retryButtonText}>重试</Text>
          </Pressable>
        </View>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>八字命盘 #{chartId}</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 基本信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>📋 基本信息</Text>
          <View style={styles.infoGrid}>
            <View style={styles.infoItem}>
              <Text style={styles.infoLabel}>性别</Text>
              <Text style={styles.infoValue}>{chartData.gender === 'Male' ? '男' : '女'}</Text>
            </View>
            <View style={styles.infoItem}>
              <Text style={styles.infoLabel}>出生年份</Text>
              <Text style={styles.infoValue}>{chartData.birthYear}年</Text>
            </View>
            <View style={styles.infoItem}>
              <Text style={styles.infoLabel}>日历类型</Text>
              <Text style={styles.infoValue}>{chartData.inputCalendarType || '公历'}</Text>
            </View>
          </View>
        </View>

        {/* 四柱信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>📅 四柱八字</Text>
          <View style={styles.pillarsContainer}>
            {[
              { label: '年柱', zhu: chartData.sizhu?.yearZhu, color: '#dc2626' },
              { label: '月柱', zhu: chartData.sizhu?.monthZhu, color: '#f59e0b' },
              { label: '日柱', zhu: chartData.sizhu?.dayZhu, color: '#22c55e' },
              { label: '时柱', zhu: chartData.sizhu?.hourZhu, color: '#3b82f6' },
            ].map((pillar, idx) => {
              // ganzhi 可能是字符串 "辛巳" 或对象 { gan: '辛', zhi: '巳' }
              const ganzhi = pillar.zhu?.ganzhi;
              const ganzhiStr = typeof ganzhi === 'string' ? ganzhi : `${ganzhi?.gan || ''}${ganzhi?.zhi || ''}`;
              const gan = ganzhiStr.charAt(0) || '';
              const zhi = ganzhiStr.charAt(1) || '';
              return (
                <View key={idx} style={[styles.pillarCard, { borderTopColor: pillar.color }]}>
                  <Text style={[styles.pillarLabel, { color: pillar.color }]}>{pillar.label}</Text>
                  <Text style={styles.pillarGanzhi}>
                    <Text style={styles.pillarGan}>{gan}</Text>
                    <Text style={styles.pillarZhi}>{zhi}</Text>
                  </Text>
                  <Text style={styles.pillarShishen}>{pillar.zhu?.tianganShishen}</Text>
                  <Text style={styles.pillarNayin}>{pillar.zhu?.nayin}</Text>
                </View>
              );
            })}
          </View>
          <View style={styles.rizhuRow}>
            <Text style={styles.rizhuLabel}>日主：</Text>
            <Text style={styles.rizhuValue}>{chartData.sizhu?.rizhu}</Text>
          </View>
        </View>

        {/* 命盘分析 */}
        {chartData.analysis && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>🔮 命盘分析</Text>
            <View style={styles.analysisGrid}>
              <View style={styles.analysisItem}>
                <Text style={styles.analysisLabel}>格局</Text>
                <Text style={styles.analysisValue}>{chartData.analysis.geJu}</Text>
              </View>
              <View style={styles.analysisItem}>
                <Text style={styles.analysisLabel}>强弱</Text>
                <Text style={styles.analysisValue}>{chartData.analysis.qiangRuo}</Text>
              </View>
              <View style={[styles.analysisItem, { backgroundColor: '#dcfce7' }]}>
                <Text style={styles.analysisLabel}>用神</Text>
                <Text style={[styles.analysisValue, { color: '#16a34a' }]}>{chartData.analysis.yongShen}</Text>
              </View>
              <View style={[styles.analysisItem, { backgroundColor: '#dbeafe' }]}>
                <Text style={styles.analysisLabel}>喜神</Text>
                <Text style={[styles.analysisValue, { color: '#2563eb' }]}>{chartData.analysis.xiShen}</Text>
              </View>
              <View style={[styles.analysisItem, { backgroundColor: '#fee2e2' }]}>
                <Text style={styles.analysisLabel}>忌神</Text>
                <Text style={[styles.analysisValue, { color: '#dc2626' }]}>{chartData.analysis.jiShen}</Text>
              </View>
              <View style={styles.analysisItem}>
                <Text style={styles.analysisLabel}>评分</Text>
                <Text style={styles.analysisValue}>{chartData.analysis.score}/100</Text>
              </View>
            </View>
          </View>
        )}

        {/* 起运信息 */}
        {chartData.qiyun && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>🌟 起运信息</Text>
            <View style={styles.qiyunGrid}>
              <View style={styles.qiyunItem}>
                <Text style={styles.qiyunLabel}>起运年龄</Text>
                <Text style={styles.qiyunValue}>
                  {chartData.qiyun.ageYears}岁{chartData.qiyun.ageMonths}月{chartData.qiyun.ageDays}天
                </Text>
              </View>
              <View style={styles.qiyunItem}>
                <Text style={styles.qiyunLabel}>大运方向</Text>
                <Text style={[styles.qiyunValue, { color: chartData.qiyun.isShun ? '#22c55e' : '#ef4444' }]}>
                  {chartData.qiyun.isShun ? '顺排 ↑' : '逆排 ↓'}
                </Text>
              </View>
              <View style={styles.qiyunItem}>
                <Text style={styles.qiyunLabel}>交运时间</Text>
                <Text style={styles.qiyunValue}>
                  {chartData.qiyun.jiaoyunYear}年{chartData.qiyun.jiaoyunMonth}月{chartData.qiyun.jiaoyunDay}日
                </Text>
              </View>
            </View>
          </View>
        )}

        {/* 大运列表 */}
        {chartData.dayunList && chartData.dayunList.length > 0 && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>🔄 大运流年</Text>
            <ScrollView horizontal showsHorizontalScrollIndicator={false}>
              <View style={styles.dayunContainer}>
                {chartData.dayunList.map((dayun, idx) => {
                  // ganzhi 可能是字符串 "丁酉" 或对象
                  const ganzhi = dayun.ganzhi;
                  const ganzhiStr = typeof ganzhi === 'string' ? ganzhi : `${ganzhi?.gan || ''}${ganzhi?.zhi || ''}`;
                  const gan = ganzhiStr.charAt(0) || '';
                  const zhi = ganzhiStr.charAt(1) || '';
                  return (
                    <View key={idx} style={styles.dayunCard}>
                      <Text style={styles.dayunIndex}>第{idx + 1}运</Text>
                      <Text style={styles.dayunGanzhi}>
                        <Text style={{ color: '#dc2626' }}>{gan}</Text>
                        <Text style={{ color: '#2563eb' }}>{zhi}</Text>
                      </Text>
                      <Text style={styles.dayunAge}>{dayun.startAge}-{dayun.endAge}岁</Text>
                      <Text style={styles.dayunYear}>{dayun.startYear}-{dayun.endYear}年</Text>
                    </View>
                  );
                })}
              </View>
            </ScrollView>
          </View>
        )}

        {/* 神煞列表 */}
        {chartData.shenshaList && chartData.shenshaList.length > 0 && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>⭐ 神煞</Text>
            <View style={styles.shenshaContainer}>
              {chartData.shenshaList.map((shensha: any, idx) => (
                <View key={idx} style={[
                  styles.shenshaTag, 
                  { backgroundColor: shensha.nature === 'Ji' ? '#dcfce7' : shensha.nature === 'Xiong' ? '#fee2e2' : '#fef3c7' }
                ]}>
                  <Text style={[
                    styles.shenshaText,
                    { color: shensha.nature === 'Ji' ? '#16a34a' : shensha.nature === 'Xiong' ? '#dc2626' : '#92400e' }
                  ]}>
                    {shensha.shensha || shensha} ({shensha.position})
                  </Text>
                </View>
              ))}
            </View>
          </View>
        )}

        {/* 空亡信息 */}
        {chartData.kongwang && (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>🌑 空亡</Text>
            <View style={styles.kongwangGrid}>
              {[
                { label: '年柱', isKong: chartData.kongwang.yearKong },
                { label: '月柱', isKong: chartData.kongwang.monthKong },
                { label: '日柱', isKong: chartData.kongwang.dayKong },
                { label: '时柱', isKong: chartData.kongwang.hourKong },
              ].map((item, idx) => (
                <View key={idx} style={[
                  styles.kongwangItem,
                  { backgroundColor: item.isKong ? '#fee2e2' : '#f0fdf4' }
                ]}>
                  <Text style={styles.kongwangLabel}>{item.label}</Text>
                  <Text style={[
                    styles.kongwangValue,
                    { color: item.isKong ? '#dc2626' : '#16a34a' }
                  ]}>
                    {item.isKong ? '空亡' : '不空'}
                  </Text>
                </View>
              ))}
            </View>
          </View>
        )}

        {/* 底部留白 */}
        <View style={{ height: 40 }} />
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
  headerRight: {
    width: 50,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#6b7280',
  },
  errorContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  errorIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  errorTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  errorDesc: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 20,
  },
  retryButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  retryButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  infoGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  infoItem: {
    backgroundColor: '#f9fafb',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    minWidth: 100,
  },
  infoLabel: {
    fontSize: 11,
    color: '#6b7280',
    marginBottom: 2,
  },
  infoValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
  },
  pillarsContainer: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    gap: 8,
  },
  pillarCard: {
    flex: 1,
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    borderTopWidth: 3,
  },
  pillarLabel: {
    fontSize: 12,
    fontWeight: '600',
    marginBottom: 8,
  },
  pillarGanzhi: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 4,
  },
  pillarGan: {
    color: '#dc2626',
  },
  pillarZhi: {
    color: '#2563eb',
  },
  pillarShishen: {
    fontSize: 11,
    color: '#6b7280',
    marginBottom: 2,
  },
  pillarNayin: {
    fontSize: 10,
    color: '#9ca3af',
  },
  rizhuRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  rizhuLabel: {
    fontSize: 14,
    color: '#6b7280',
  },
  rizhuValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#dc2626',
  },
  analysisGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  analysisItem: {
    backgroundColor: '#f9fafb',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    minWidth: 100,
    flex: 1,
  },
  analysisLabel: {
    fontSize: 11,
    color: '#6b7280',
    marginBottom: 2,
  },
  analysisValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
  },
  qiyunGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  qiyunItem: {
    backgroundColor: '#f9fafb',
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    flex: 1,
    minWidth: 100,
  },
  qiyunLabel: {
    fontSize: 11,
    color: '#6b7280',
    marginBottom: 2,
  },
  qiyunValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
  },
  dayunContainer: {
    flexDirection: 'row',
    gap: 8,
    paddingRight: 16,
  },
  dayunCard: {
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    padding: 12,
    alignItems: 'center',
    minWidth: 80,
  },
  dayunIndex: {
    fontSize: 10,
    color: '#9ca3af',
    marginBottom: 4,
  },
  dayunGanzhi: {
    fontSize: 20,
    fontWeight: 'bold',
    marginBottom: 4,
  },
  dayunAge: {
    fontSize: 11,
    color: '#6b7280',
  },
  dayunYear: {
    fontSize: 10,
    color: '#9ca3af',
  },
  shenshaContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  shenshaTag: {
    backgroundColor: '#fef3c7',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 12,
  },
  shenshaText: {
    fontSize: 12,
    color: '#92400e',
  },
  kongwangText: {
    fontSize: 12,
    color: '#6b7280',
    fontFamily: 'monospace',
  },
  kongwangGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  kongwangItem: {
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderRadius: 8,
    minWidth: 70,
    alignItems: 'center',
  },
  kongwangLabel: {
    fontSize: 11,
    color: '#6b7280',
    marginBottom: 2,
  },
  kongwangValue: {
    fontSize: 13,
    fontWeight: '600',
  },
});
