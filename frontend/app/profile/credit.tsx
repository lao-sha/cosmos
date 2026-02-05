import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Dimensions,
} from 'react-native';
import { useRouter } from 'expo-router';
import {
  Shield,
  TrendingUp,
  TrendingDown,
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
} from 'lucide-react-native';
import Svg, { Circle, G } from 'react-native-svg';
import { useColors } from '@/hooks/useColors';
import { useWalletStore } from '@/stores/wallet';
import {
  getCreditInfo,
  getCreditHistory,
  getScoreColor,
  getLevelConfig,
  CREDIT_LEVELS,
  CREDIT_FACTORS,
  type CreditInfo,
  type CreditHistory,
} from '@/services/credit';
import { Card } from '@/components/ui';
import { Colors } from '@/constants/colors';

const { width } = Dimensions.get('window');
const CIRCLE_SIZE = width * 0.5;
const STROKE_WIDTH = 12;
const RADIUS = (CIRCLE_SIZE - STROKE_WIDTH) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export default function CreditScreen() {
  const colors = useColors();
  const router = useRouter();
  const { address } = useWalletStore();

  const [info, setInfo] = useState<CreditInfo | null>(null);
  const [history, setHistory] = useState<CreditHistory[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
  }, [address]);

  const loadData = async () => {
    if (!address) return;
    setLoading(true);
    try {
      const [infoData, historyData] = await Promise.all([
        getCreditInfo(address),
        getCreditHistory(address),
      ]);
      setInfo(infoData);
      setHistory(historyData);
    } catch (error) {
      console.error('Failed to load credit data:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading || !info) {
    return (
      <View style={[styles.container, { backgroundColor: colors.background }]}>
        <Text style={{ color: colors.textSecondary, textAlign: 'center', marginTop: 100 }}>
          加载中...
        </Text>
      </View>
    );
  }

  const scoreColor = getScoreColor(info.score);
  const levelConfig = getLevelConfig(info.level);
  const progress = info.score / 1000;
  const strokeDashoffset = CIRCUMFERENCE * (1 - progress);

  return (
    <ScrollView
      style={[styles.container, { backgroundColor: colors.background }]}
      contentContainerStyle={styles.content}
    >
      {/* Score Circle */}
      <View style={styles.scoreSection}>
        <View style={styles.circleContainer}>
          <Svg width={CIRCLE_SIZE} height={CIRCLE_SIZE}>
            <G rotation="-90" origin={`${CIRCLE_SIZE / 2}, ${CIRCLE_SIZE / 2}`}>
              {/* Background circle */}
              <Circle
                cx={CIRCLE_SIZE / 2}
                cy={CIRCLE_SIZE / 2}
                r={RADIUS}
                stroke={colors.border}
                strokeWidth={STROKE_WIDTH}
                fill="transparent"
              />
              {/* Progress circle */}
              <Circle
                cx={CIRCLE_SIZE / 2}
                cy={CIRCLE_SIZE / 2}
                r={RADIUS}
                stroke={scoreColor}
                strokeWidth={STROKE_WIDTH}
                fill="transparent"
                strokeDasharray={CIRCUMFERENCE}
                strokeDashoffset={strokeDashoffset}
                strokeLinecap="round"
              />
            </G>
          </Svg>
          <View style={styles.scoreContent}>
            <Text style={[styles.scoreValue, { color: scoreColor }]}>
              {info.score}
            </Text>
            <Text style={[styles.scoreLabel, { color: colors.textSecondary }]}>
              信用分
            </Text>
            <View style={[styles.levelBadge, { backgroundColor: scoreColor + '20' }]}>
              <Text style={[styles.levelText, { color: scoreColor }]}>
                {levelConfig.name}
              </Text>
            </View>
          </View>
        </View>
      </View>

      {/* Score Range */}
      <View style={styles.rangeContainer}>
        {CREDIT_LEVELS.map((level, index) => (
          <View key={level.level} style={styles.rangeItem}>
            <View
              style={[
                styles.rangeBar,
                {
                  backgroundColor: level.color,
                  opacity: info.level === level.level ? 1 : 0.3,
                },
              ]}
            />
            <Text style={[styles.rangeLabel, { color: colors.textTertiary }]}>
              {level.min}
            </Text>
          </View>
        ))}
        <Text style={[styles.rangeLabel, { color: colors.textTertiary }]}>
          1000
        </Text>
      </View>

      {/* Stats */}
      <View style={styles.statsGrid}>
        <Card style={styles.statCard}>
          <CheckCircle size={20} color={Colors.success} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {info.completedOrders}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            完成订单
          </Text>
        </Card>
        <Card style={styles.statCard}>
          <XCircle size={20} color={Colors.error} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {info.cancelledOrders}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            取消订单
          </Text>
        </Card>
        <Card style={styles.statCard}>
          <Clock size={20} color={Colors.primary} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {info.onTimeRate}%
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            准时率
          </Text>
        </Card>
        <Card style={styles.statCard}>
          <AlertTriangle size={20} color={Colors.warning} />
          <Text style={[styles.statValue, { color: colors.textPrimary }]}>
            {info.disputes}
          </Text>
          <Text style={[styles.statLabel, { color: colors.textSecondary }]}>
            争议次数
          </Text>
        </Card>
      </View>

      {/* Factors */}
      <Card style={styles.factorsCard}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          影响因素
        </Text>
        {CREDIT_FACTORS.map((factor, index) => (
          <View
            key={factor.factor}
            style={[
              styles.factorItem,
              index < CREDIT_FACTORS.length - 1 && {
                borderBottomWidth: 1,
                borderBottomColor: colors.border,
              },
            ]}
          >
            <View style={styles.factorLeft}>
              {factor.impact.startsWith('+') ? (
                <TrendingUp size={18} color={Colors.success} />
              ) : (
                <TrendingDown size={18} color={Colors.error} />
              )}
              <View style={styles.factorInfo}>
                <Text style={[styles.factorName, { color: colors.textPrimary }]}>
                  {factor.factor}
                </Text>
                <Text style={[styles.factorDesc, { color: colors.textTertiary }]}>
                  {factor.description}
                </Text>
              </View>
            </View>
            <Text
              style={[
                styles.factorImpact,
                {
                  color: factor.impact.startsWith('+')
                    ? Colors.success
                    : Colors.error,
                },
              ]}
            >
              {factor.impact}
            </Text>
          </View>
        ))}
      </Card>

      {/* History */}
      <Card style={styles.historyCard}>
        <Text style={[styles.cardTitle, { color: colors.textPrimary }]}>
          变动记录
        </Text>
        {history.length === 0 ? (
          <Text style={[styles.emptyText, { color: colors.textTertiary }]}>
            暂无记录
          </Text>
        ) : (
          history.slice(0, 10).map((record) => (
            <View
              key={record.id}
              style={[styles.historyItem, { borderBottomColor: colors.border }]}
            >
              <View style={styles.historyLeft}>
                {record.type === 'increase' ? (
                  <TrendingUp size={16} color={Colors.success} />
                ) : (
                  <TrendingDown size={16} color={Colors.error} />
                )}
                <Text style={[styles.historyReason, { color: colors.textPrimary }]}>
                  {record.reason}
                </Text>
              </View>
              <View style={styles.historyRight}>
                <Text
                  style={[
                    styles.historyAmount,
                    {
                      color:
                        record.type === 'increase' ? Colors.success : Colors.error,
                    },
                  ]}
                >
                  {record.type === 'increase' ? '+' : '-'}{record.amount}
                </Text>
                <Text style={[styles.historyTime, { color: colors.textTertiary }]}>
                  {new Date(record.createdAt).toLocaleDateString()}
                </Text>
              </View>
            </View>
          ))
        )}
      </Card>

      {/* Tips */}
      <Card style={[styles.tipsCard, { backgroundColor: Colors.info + '10' }]}>
        <Shield size={20} color={Colors.info} />
        <Text style={[styles.tipsText, { color: Colors.info }]}>
          保持良好的交易习惯，按时完成订单，可以持续提升信用分。高信用分用户享有更多交易特权！
        </Text>
      </Card>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 40,
  },
  scoreSection: {
    alignItems: 'center',
    marginBottom: 16,
  },
  circleContainer: {
    position: 'relative',
    alignItems: 'center',
    justifyContent: 'center',
  },
  scoreContent: {
    position: 'absolute',
    alignItems: 'center',
  },
  scoreValue: {
    fontSize: 48,
    fontWeight: '700',
  },
  scoreLabel: {
    fontSize: 14,
    marginTop: 4,
  },
  levelBadge: {
    marginTop: 8,
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
  levelText: {
    fontSize: 13,
    fontWeight: '600',
  },
  rangeContainer: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    marginBottom: 24,
    paddingHorizontal: 8,
  },
  rangeItem: {
    flex: 1,
    alignItems: 'flex-start',
  },
  rangeBar: {
    height: 6,
    width: '100%',
    borderRadius: 3,
  },
  rangeLabel: {
    fontSize: 10,
    marginTop: 4,
  },
  statsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
    marginBottom: 16,
  },
  statCard: {
    width: (width - 44) / 2,
    alignItems: 'center',
    paddingVertical: 16,
  },
  statValue: {
    fontSize: 24,
    fontWeight: '700',
    marginTop: 8,
  },
  statLabel: {
    fontSize: 12,
    marginTop: 4,
  },
  factorsCard: {
    marginBottom: 16,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  factorItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 12,
  },
  factorLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  factorInfo: {
    marginLeft: 10,
    flex: 1,
  },
  factorName: {
    fontSize: 14,
    fontWeight: '500',
  },
  factorDesc: {
    fontSize: 12,
    marginTop: 2,
  },
  factorImpact: {
    fontSize: 14,
    fontWeight: '600',
  },
  historyCard: {
    marginBottom: 16,
  },
  emptyText: {
    textAlign: 'center',
    fontSize: 14,
    paddingVertical: 20,
  },
  historyItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 10,
    borderBottomWidth: 1,
  },
  historyLeft: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  historyReason: {
    fontSize: 14,
    marginLeft: 8,
  },
  historyRight: {
    alignItems: 'flex-end',
  },
  historyAmount: {
    fontSize: 14,
    fontWeight: '600',
  },
  historyTime: {
    fontSize: 11,
    marginTop: 2,
  },
  tipsCard: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    borderWidth: 0,
  },
  tipsText: {
    flex: 1,
    fontSize: 13,
    lineHeight: 20,
    marginLeft: 10,
  },
});
