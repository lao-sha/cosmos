/**
 * 做市商入口页面
 * 路径: /maker
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { useRouter } from 'expo-router';
import { useMakerStore, selectIsMaker, selectIsApplying } from '@/stores/maker.store';
import { ApplicationStatus } from '@/services/maker.service';
import { MakerStatusCard } from '@/features/maker/components';
import { PageHeader } from '@/components/PageHeader';

export default function MakerIndexPage() {
  const router = useRouter();
  const {
    makerApp,
    depositUsdValue,
    isLoading,
    fetchMakerInfo,
    fetchDustPrice,
  } = useMakerStore();

  const isMaker = useMakerStore(selectIsMaker);
  const isApplying = useMakerStore(selectIsApplying);

  useEffect(() => {
    fetchMakerInfo();
    fetchDustPrice();
  }, []);

  const handleApply = () => {
    router.push('/maker/apply/deposit');
  };

  const handleGoToDashboard = () => {
    router.push('/maker/dashboard');
  };

  const handleContinueApplication = () => {
    if (makerApp?.status === ApplicationStatus.DepositLocked) {
      router.push('/maker/apply/info');
    } else if (makerApp?.status === ApplicationStatus.PendingReview) {
      router.push('/maker/apply/pending');
    }
  };

  if (isLoading) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#B2955D" />
        <Text style={styles.loadingText}>加载中...</Text>
      </View>
    );
  }

  // 已是做市商，显示状态卡片
  if (isMaker && makerApp) {
    return (
      <View style={styles.container}>
        <PageHeader title="做市商中心" />
        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          <MakerStatusCard
            maker={makerApp}
            depositUsdValue={depositUsdValue}
            onPress={handleGoToDashboard}
          />

          <View style={styles.quickActions}>
            <Text style={styles.sectionTitle}>快捷操作</Text>
            <View style={styles.actionGrid}>
              <TouchableOpacity
                style={styles.actionItem}
                onPress={() => router.push('/maker/deposit')}
              >
                <Text style={styles.actionIcon}>💰</Text>
                <Text style={styles.actionText}>押金管理</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.actionItem}
                onPress={() => router.push('/maker/settings')}
              >
                <Text style={styles.actionIcon}>⚙️</Text>
                <Text style={styles.actionText}>设置</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.actionItem}
                onPress={() => router.push('/maker/penalties')}
              >
                <Text style={styles.actionIcon}>📜</Text>
                <Text style={styles.actionText}>扣除记录</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.actionItem}
                onPress={handleGoToDashboard}
              >
                <Text style={styles.actionIcon}>📊</Text>
                <Text style={styles.actionText}>控制台</Text>
              </TouchableOpacity>
            </View>
          </View>
        </ScrollView>
      </View>
    );
  }

  // 申请中，显示继续申请入口
  if (isApplying && makerApp) {
    return (
      <View style={styles.container}>
        <PageHeader title="做市商中心" />
        <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
          <View style={styles.applyingCard}>
            <Text style={styles.applyingIcon}>⏳</Text>
            <Text style={styles.applyingTitle}>申请进行中</Text>
            <Text style={styles.applyingStatus}>
              当前状态: {makerApp.status === ApplicationStatus.DepositLocked ? '押金已锁定，待提交资料' : '资料已提交，待审核'}
            </Text>
            <TouchableOpacity
              style={styles.continueButton}
              onPress={handleContinueApplication}
            >
              <Text style={styles.continueButtonText}>继续申请</Text>
            </TouchableOpacity>
          </View>
        </ScrollView>
      </View>
    );
  }

  // 未申请，显示申请入口
  return (
    <View style={styles.container}>
      <PageHeader title="做市商中心" />
      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 申请入口卡片 */}
        <View style={styles.applyCard}>
          <Text style={styles.applyIcon}>💼</Text>
          <Text style={styles.applyTitle}>成为做市商</Text>
          <Text style={styles.applyDesc}>
            提供 OTC 交易服务{'\n'}赚取交易溢价收益
          </Text>
          <View style={styles.applyRequirements}>
            <View style={styles.requirementItem}>
              <Text style={styles.requirementLabel}>押金要求</Text>
              <Text style={styles.requirementValue}>1000 USD 等值 DUST</Text>
            </View>
            <View style={styles.requirementItem}>
              <Text style={styles.requirementLabel}>提现冷却</Text>
              <Text style={styles.requirementValue}>7 天</Text>
            </View>
          </View>
          <TouchableOpacity style={styles.applyButton} onPress={handleApply}>
            <Text style={styles.applyButtonText}>立即申请</Text>
          </TouchableOpacity>
        </View>

        {/* 做市商权益 */}
        <Text style={styles.sectionTitle}>做市商权益</Text>

        <View style={styles.benefitCard}>
          <Text style={styles.benefitIcon}>💰</Text>
          <View style={styles.benefitContent}>
            <Text style={styles.benefitTitle}>交易溢价</Text>
            <Text style={styles.benefitDesc}>自定义买卖溢价，赚取差价</Text>
          </View>
        </View>

        <View style={styles.benefitCard}>
          <Text style={styles.benefitIcon}>🛡️</Text>
          <View style={styles.benefitContent}>
            <Text style={styles.benefitTitle}>押金保障</Text>
            <Text style={styles.benefitDesc}>动态押金机制，价格波动自动调整，保障交易安全</Text>
          </View>
        </View>

        <View style={styles.benefitCard}>
          <Text style={styles.benefitIcon}>⭐</Text>
          <View style={styles.benefitContent}>
            <Text style={styles.benefitTitle}>信用体系</Text>
            <Text style={styles.benefitDesc}>建立信用评分，获得更多订单</Text>
          </View>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
  },
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F5F5F7',
  },
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#8E8E93',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  // 申请入口卡片
  applyCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 24,
    alignItems: 'center',
    marginBottom: 24,
  },
  applyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  applyTitle: {
    fontSize: 22,
    fontWeight: '700',
    color: '#1C1C1E',
    marginBottom: 8,
  },
  applyDesc: {
    fontSize: 14,
    color: '#8E8E93',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 20,
  },
  applyRequirements: {
    width: '100%',
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    padding: 16,
    marginBottom: 20,
  },
  requirementItem: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  requirementLabel: {
    fontSize: 14,
    color: '#8E8E93',
  },
  requirementValue: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1C1C1E',
  },
  applyButton: {
    width: '100%',
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  applyButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  // 申请中卡片
  applyingCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 24,
    alignItems: 'center',
  },
  applyingIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  applyingTitle: {
    fontSize: 20,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 8,
  },
  applyingStatus: {
    fontSize: 14,
    color: '#8E8E93',
    marginBottom: 20,
  },
  continueButton: {
    width: '100%',
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 16,
    alignItems: 'center',
  },
  continueButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  // 权益卡片
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 12,
  },
  benefitCard: {
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  benefitIcon: {
    fontSize: 28,
    marginRight: 16,
  },
  benefitContent: {
    flex: 1,
  },
  benefitTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#1C1C1E',
    marginBottom: 4,
  },
  benefitDesc: {
    fontSize: 13,
    color: '#8E8E93',
    lineHeight: 18,
  },
  // 快捷操作
  quickActions: {
    marginTop: 24,
  },
  actionGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
  },
  actionItem: {
    width: '47%',
    backgroundColor: '#FFFFFF',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
  },
  actionIcon: {
    fontSize: 28,
    marginBottom: 8,
  },
  actionText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1C1C1E',
  },
});
