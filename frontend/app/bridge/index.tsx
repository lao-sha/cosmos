/**
 * 桥接首页
 * DUST → USDT 兑换入口
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { PageHeader } from '@/components/PageHeader';
import { BottomNavBar } from '@/components/BottomNavBar';
import { PriceDisplay } from '@/features/trading/components';

export default function BridgePage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [dustPrice, setDustPrice] = useState(0.10);
  const [userSwapsCount, setUserSwapsCount] = useState(0);

  useEffect(() => {
    // TODO: 从链上获取价格和用户兑换数量
    setDustPrice(0.10);
    setUserSwapsCount(0);
  }, []);

  const handleOfficialBridge = () => {
    router.push('/bridge/official');
  };

  const handleMakerBridge = () => {
    router.push('/bridge/maker');
  };

  const handleViewHistory = () => {
    router.push('/bridge/history');
  };

  return (
    <View style={styles.wrapper}>
      <PageHeader title="DUST 桥接" />

      <ScrollView style={styles.container} contentContainerStyle={styles.contentContainer}>
        {/* 价格显示 */}
        <View style={styles.section}>
          <PriceDisplay
            price={dustPrice}
            label="🌉 当前汇率"
          />
        </View>

        {/* 桥接说明 */}
        <View style={styles.section}>
          <View style={styles.infoCard}>
            <Text style={styles.infoTitle}>💡 什么是桥接？</Text>
            <Text style={styles.infoText}>
              桥接服务允许您将 DUST 代币兑换为 USDT (TRC20)，
              资金将转入您指定的 TRON 钱包地址。
            </Text>
          </View>
        </View>

        {/* 桥接方式选择 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>选择桥接方式</Text>

          {/* 官方桥接 */}
          <TouchableOpacity
            style={styles.bridgeOption}
            onPress={handleOfficialBridge}
            activeOpacity={0.7}
          >
            <View style={styles.optionIcon}>
              <Text style={styles.optionEmoji}>🏛️</Text>
            </View>
            <View style={styles.optionContent}>
              <Text style={styles.optionTitle}>官方桥接</Text>
              <Text style={styles.optionDesc}>
                由治理账户处理，安全可靠
              </Text>
              <View style={styles.optionTags}>
                <View style={styles.tag}>
                  <Text style={styles.tagText}>无溢价</Text>
                </View>
                <View style={styles.tag}>
                  <Text style={styles.tagText}>24h 内到账</Text>
                </View>
              </View>
            </View>
            <Text style={styles.arrow}>›</Text>
          </TouchableOpacity>

          {/* 做市商桥接 */}
          <TouchableOpacity
            style={styles.bridgeOption}
            onPress={handleMakerBridge}
            activeOpacity={0.7}
          >
            <View style={styles.optionIcon}>
              <Text style={styles.optionEmoji}>👥</Text>
            </View>
            <View style={styles.optionContent}>
              <Text style={styles.optionTitle}>做市商桥接</Text>
              <Text style={styles.optionDesc}>
                选择做市商，快速到账
              </Text>
              <View style={styles.optionTags}>
                <View style={[styles.tag, styles.tagGreen]}>
                  <Text style={[styles.tagText, styles.tagTextGreen]}>快速</Text>
                </View>
                <View style={styles.tag}>
                  <Text style={styles.tagText}>30分钟超时保护</Text>
                </View>
              </View>
            </View>
            <Text style={styles.arrow}>›</Text>
          </TouchableOpacity>
        </View>

        {/* 我的兑换记录 */}
        <View style={styles.section}>
          <TouchableOpacity
            style={styles.historyCard}
            onPress={handleViewHistory}
            activeOpacity={0.7}
          >
            <View style={styles.historyLeft}>
              <Text style={styles.historyTitle}>📋 我的兑换记录</Text>
              <Text style={styles.historyCount}>
                共 {userSwapsCount} 笔兑换
              </Text>
            </View>
            <Text style={styles.arrow}>›</Text>
          </TouchableOpacity>
        </View>

        {/* 注意事项 */}
        <View style={styles.section}>
          <Text style={styles.noticeTitle}>⚠️ 注意事项</Text>
          <Text style={styles.noticeText}>• 最小兑换金额：10 DUST</Text>
          <Text style={styles.noticeText}>• 请确保 TRON 地址正确，转账后无法撤回</Text>
          <Text style={styles.noticeText}>• 做市商兑换超时将自动退款</Text>
          <Text style={styles.noticeText}>• 如遇问题可发起举报，进入仲裁流程</Text>
        </View>
      </ScrollView>

      <BottomNavBar activeTab="profile" />
    </View>
  );
}

const styles = StyleSheet.create({
  wrapper: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  container: {
    flex: 1,
  },
  contentContainer: {
    paddingBottom: 20,
  },
  section: {
    padding: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 16,
  },
  infoCard: {
    backgroundColor: '#FFF9F0',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#B2955D',
  },
  infoTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  bridgeOption: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    flexDirection: 'row',
    alignItems: 'center',
  },
  optionIcon: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#F5F5F7',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  optionEmoji: {
    fontSize: 24,
  },
  optionContent: {
    flex: 1,
  },
  optionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 4,
  },
  optionDesc: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 8,
  },
  optionTags: {
    flexDirection: 'row',
    gap: 8,
  },
  tag: {
    backgroundColor: '#F5F5F7',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
  },
  tagGreen: {
    backgroundColor: '#E8F8EB',
  },
  tagText: {
    fontSize: 12,
    color: '#666666',
  },
  tagTextGreen: {
    color: '#4CD964',
  },
  arrow: {
    fontSize: 24,
    color: '#C7C7CC',
  },
  historyCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  historyLeft: {
    flex: 1,
  },
  historyTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 4,
  },
  historyCount: {
    fontSize: 14,
    color: '#666666',
  },
  noticeTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 12,
  },
  noticeText: {
    fontSize: 14,
    color: '#666666',
    marginBottom: 6,
  },
});
