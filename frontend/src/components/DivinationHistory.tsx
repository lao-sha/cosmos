/**
 * 通用占卜历史记录组件
 * 用于显示用户的占卜历史记录列表
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  RefreshControl,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useRouter } from 'expo-router';
import {
  divinationService,
  DivinationType,
  DivinationRecord,
} from '@/services/divination.service';
import { getCurrentSignerAddress } from '@/lib/signer';

const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

// 占卜类型中文名称映射
const DIVINATION_TYPE_NAMES: Record<DivinationType, string> = {
  [DivinationType.Bazi]: '八字',
  [DivinationType.Ziwei]: '紫微斗数',
  [DivinationType.Qimen]: '奇门遁甲',
  [DivinationType.Liuyao]: '六爻',
  [DivinationType.Meihua]: '梅花易数',
  [DivinationType.Tarot]: '塔罗',
  [DivinationType.Daliuren]: '大六壬',
  [DivinationType.Xiaoliuren]: '小六壬',
};

// 占卜类型图标映射
const DIVINATION_TYPE_ICONS: Record<DivinationType, string> = {
  [DivinationType.Bazi]: 'calendar-outline',
  [DivinationType.Ziwei]: 'star-outline',
  [DivinationType.Qimen]: 'grid-outline',
  [DivinationType.Liuyao]: 'layers-outline',
  [DivinationType.Meihua]: 'flower-outline',
  [DivinationType.Tarot]: 'card-outline',
  [DivinationType.Daliuren]: 'compass-outline',
  [DivinationType.Xiaoliuren]: 'hand-left-outline',
};

interface DivinationHistoryProps {
  divinationType?: DivinationType; // 可选：只显示特定类型的记录
  onRecordPress?: (record: DivinationRecord) => void; // 点击记录的回调
  emptyMessage?: string; // 空状态提示文字
}

export function DivinationHistory({
  divinationType,
  onRecordPress,
  emptyMessage = '暂无占卜记录',
}: DivinationHistoryProps) {
  const router = useRouter();
  const [records, setRecords] = useState<DivinationRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  const loadHistory = async () => {
    try {
      const address = getCurrentSignerAddress();
      if (!address) {
        console.log('No wallet address available');
        setRecords([]);
        return;
      }

      const history = await divinationService.getDivinationHistory(
        address,
        divinationType
      );
      setRecords(history);
    } catch (error) {
      console.error('Load history error:', error);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  useEffect(() => {
    loadHistory();
  }, [divinationType]);

  const handleRefresh = () => {
    setRefreshing(true);
    loadHistory();
  };

  const handleRecordPress = (record: DivinationRecord) => {
    if (onRecordPress) {
      onRecordPress(record);
    } else {
      // 默认行为：导航到详情页（如果存在）
      // router.push(`/divination/detail/${record.id}` as any);
      console.log('Record pressed:', record);
    }
  };

  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const renderRecord = ({ item }: { item: DivinationRecord }) => {
    const typeName = DIVINATION_TYPE_NAMES[item.divinationType];
    const iconName = DIVINATION_TYPE_ICONS[item.divinationType];

    return (
      <TouchableOpacity
        style={styles.recordCard}
        onPress={() => handleRecordPress(item)}
        activeOpacity={0.7}
      >
        <View style={styles.recordHeader}>
          <View style={styles.typeInfo}>
            <Ionicons name={iconName as any} size={20} color={THEME_COLOR} />
            <Text style={styles.typeName}>{typeName}</Text>
          </View>
          <Text style={styles.recordId}>#{item.id}</Text>
        </View>

        <View style={styles.recordBody}>
          <View style={styles.infoRow}>
            <Ionicons name="time-outline" size={14} color="#999" />
            <Text style={styles.infoText}>{formatTimestamp(item.timestamp)}</Text>
          </View>
          <View style={styles.infoRow}>
            <Ionicons name="cube-outline" size={14} color="#999" />
            <Text style={styles.infoText}>区块 {item.blockNumber}</Text>
          </View>
        </View>

        <View style={styles.recordFooter}>
          <Text style={styles.viewDetail}>查看详情</Text>
          <Ionicons name="chevron-forward" size={16} color={THEME_COLOR} />
        </View>
      </TouchableOpacity>
    );
  };

  const renderEmpty = () => (
    <View style={styles.emptyContainer}>
      <Ionicons name="document-text-outline" size={64} color="#CCC" />
      <Text style={styles.emptyText}>{emptyMessage}</Text>
    </View>
  );

  if (loading) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={THEME_COLOR} />
        <Text style={styles.loadingText}>加载中...</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <FlatList
        data={records}
        renderItem={renderRecord}
        keyExtractor={(item) => item.id.toString()}
        contentContainerStyle={styles.listContent}
        ListEmptyComponent={renderEmpty}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={handleRefresh}
            tintColor={THEME_COLOR}
            colors={[THEME_COLOR]}
          />
        }
        showsVerticalScrollIndicator={false}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  listContent: {
    padding: 16,
    paddingBottom: 100,
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: THEME_BG,
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#999',
  },
  recordCard: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.06,
    shadowRadius: 8,
    elevation: 2,
  },
  recordHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  typeInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  typeName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  recordId: {
    fontSize: 12,
    color: '#999',
  },
  recordBody: {
    gap: 8,
    marginBottom: 12,
  },
  infoRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  infoText: {
    fontSize: 13,
    color: '#666',
  },
  recordFooter: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  viewDetail: {
    fontSize: 14,
    color: THEME_COLOR,
    fontWeight: '500',
    marginRight: 4,
  },
  emptyContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 80,
  },
  emptyText: {
    marginTop: 16,
    fontSize: 15,
    color: '#999',
  },
});
