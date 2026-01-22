/**
 * 星尘玄鉴 - 隐私与安全设置
 * 处理加密公钥注册和命盘授权管理
 */

import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  Pressable,
  Alert,
  ActivityIndicator,
  TextInput,
  Modal,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { divinationService, AuthorizationEntry } from '@/services/divination.service';
import { useWalletStore } from '@/stores/wallet.store';
import { UnlockWalletDialog } from '@/components/UnlockWalletDialog';
import { TransactionStatusDialog } from '@/components/TransactionStatusDialog';
import { u8aToHex } from '@polkadot/util';

// 主题色
const THEME_COLOR = '#B2955D';
const THEME_BG = '#F5F5F7';

export default function PrivacyPage() {
  const router = useRouter();
  const { address } = useWalletStore();
  
  const [loading, setLoading] = useState(true);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [userCharts, setUserCharts] = useState<any[]>([]);
  const [selectedChart, setSelectedChart] = useState<any | null>(null);
  const [authorizations, setAuthorizations] = useState<AuthorizationEntry[]>([]);
  
  // 上链状态
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('');
  const [pendingAction, setPendingAction] = useState<(() => Promise<void>) | null>(null);

  useEffect(() => {
    if (address) {
      loadPrivacyData();
    }
  }, [address]);

  const loadPrivacyData = async () => {
    setLoading(true);
    try {
      if (!address) return;
      
      // 1. 获取加密公钥
      const key = await divinationService.getUserEncryptionKey(address);
      setPublicKey(key ? u8aToHex(key) : null);
      
      // 2. 获取用户命盘列表
      const chartIds = await divinationService.getUserBaziCharts(address);
      const charts = await Promise.all(
        chartIds.map(id => divinationService.getBaziChart(id))
      );
      setUserCharts(charts.filter(c => c !== null));
    } catch (error) {
      console.error('加载隐私数据失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadAuthorizations = async (chartId: number) => {
    try {
      const list = await divinationService.getChartAuthorizations(chartId);
      setAuthorizations(list);
    } catch (error) {
      console.error('加载授权列表失败:', error);
    }
  };

  const handleRegisterKey = () => {
    // 模拟生成 X25519 密钥对（实际应在客户端生成并存储私钥）
    // 这里为了演示，生成一个随机 32 字节
    const randomKey = new Uint8Array(32);
    crypto.getRandomValues(randomKey);
    
    const action = async () => {
      setShowTxStatus(true);
      setTxStatus('正在注册公钥...');
      try {
        await divinationService.registerEncryptionKey(randomKey, (status) => {
          setTxStatus(status);
        });
        setTxStatus('注册成功！');
        setTimeout(() => {
          setShowTxStatus(false);
          loadPrivacyData();
        }, 1500);
      } catch (error: any) {
        setTxStatus('注册失败');
        setTimeout(() => setShowTxStatus(false), 1500);
        Alert.alert('错误', error.message || '注册失败');
      }
    };

    setPendingAction(() => action);
    setShowUnlockDialog(true);
  };

  const handleRevoke = (chartId: number, revokee: string) => {
    const action = async () => {
      setShowTxStatus(true);
      setTxStatus('正在撤销授权...');
      try {
        await divinationService.revokeChartAccess(chartId, revokee, (status) => {
          setTxStatus(status);
        });
        setTxStatus('撤销成功！');
        setTimeout(() => {
          setShowTxStatus(false);
          loadAuthorizations(chartId);
        }, 1500);
      } catch (error: any) {
        setTxStatus('撤销失败');
        setTimeout(() => setShowTxStatus(false), 1500);
        Alert.alert('错误', error.message || '撤销失败');
      }
    };

    setPendingAction(() => action);
    setShowUnlockDialog(true);
  };

  const handleUnlockSuccess = () => {
    setShowUnlockDialog(false);
    if (pendingAction) {
      pendingAction();
      setPendingAction(null);
    }
  };

  const renderKeySection = () => (
    <View style={styles.card}>
      <View style={styles.cardHeader}>
        <Ionicons name="key-outline" size={20} color={THEME_COLOR} />
        <Text style={styles.cardTitle}>加密公钥</Text>
      </View>
      <Text style={styles.cardDesc}>
        注册加密公钥后，您可以创建私密命盘并接受他人授权。公钥将存储在区块链上。
      </Text>
      
      {publicKey ? (
        <View style={styles.keyBox}>
          <Text style={styles.keyText} numberOfLines={2}>{publicKey}</Text>
          <View style={styles.statusBadge}>
            <Ionicons name="checkmark-circle" size={14} color="#27AE60" />
            <Text style={styles.statusText}>已注册</Text>
          </View>
        </View>
      ) : (
        <Pressable style={styles.primaryButton} onPress={handleRegisterKey}>
          <Text style={styles.primaryButtonText}>立即注册加密公钥</Text>
        </Pressable>
      )}
    </View>
  );

  const renderAuthorizationsSection = () => (
    <View style={styles.card}>
      <View style={styles.cardHeader}>
        <Ionicons name="people-outline" size={20} color={THEME_COLOR} />
        <Text style={styles.cardTitle}>授权管理</Text>
      </View>
      <Text style={styles.cardDesc}>
        管理您已授权给他人（命理师、AI服务等）的命盘访问权限。
      </Text>

      {userCharts.length === 0 ? (
        <Text style={styles.emptyText}>暂无命盘数据</Text>
      ) : (
        <View style={styles.chartSelector}>
          <Text style={styles.subTitle}>选择命盘：</Text>
          <ScrollView horizontal showsHorizontalScrollIndicator={false} style={styles.chartScroll}>
            {userCharts.map((chart) => (
              <Pressable
                key={chart.id}
                style={[styles.chartChip, selectedChart?.id === chart.id && styles.chartChipActive]}
                onPress={() => {
                  setSelectedChart(chart);
                  loadAuthorizations(chart.id);
                }}
              >
                <Text style={[styles.chartChipText, selectedChart?.id === chart.id && styles.chartChipTextActive]}>
                  {chart.name || `命盘 #${chart.id}`}
                </Text>
              </Pressable>
            ))}
          </ScrollView>
        </View>
      )}

      {selectedChart && (
        <View style={styles.authList}>
          <Text style={styles.subTitle}>已授权列表：</Text>
          {authorizations.length === 0 ? (
            <Text style={styles.emptyTextSmall}>该命盘尚未授权给任何人</Text>
          ) : (
            authorizations.map((auth, idx) => (
              <View key={idx} style={styles.authItem}>
                <View style={styles.authInfo}>
                  <Text style={styles.authAccount} numberOfLines={1}>{auth.account}</Text>
                  <View style={styles.authTags}>
                    <View style={styles.authTag}><Text style={styles.authTagText}>{auth.role}</Text></View>
                    <View style={[styles.authTag, { backgroundColor: '#E8F5E9' }]}><Text style={[styles.authTagText, { color: '#2E7D32' }]}>{auth.scope}</Text></View>
                  </View>
                </View>
                <Pressable 
                  style={styles.revokeBtn}
                  onPress={() => handleRevoke(selectedChart.id, auth.account)}
                >
                  <Text style={styles.revokeBtnText}>撤销</Text>
                </Pressable>
              </View>
            ))
          )}
        </View>
      )}
    </View>
  );

  return (
    <View style={styles.container}>
      <View style={styles.navBar}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#333" />
        </Pressable>
        <Text style={styles.navTitle}>隐私与授权</Text>
        <View style={{ width: 24 }} />
      </View>

      <ScrollView style={styles.content} contentContainerStyle={styles.scrollContent}>
        {loading ? (
          <ActivityIndicator color={THEME_COLOR} style={{ marginTop: 40 }} />
        ) : (
          <>
            {renderKeySection()}
            {renderAuthorizationsSection()}
            
            <View style={styles.notice}>
              <Ionicons name="information-circle-outline" size={16} color="#999" />
              <Text style={styles.noticeText}>
                提示：所有的授权和撤销操作均在区块链上完成，确保您的命理隐私不可篡改且完全受控。
              </Text>
            </View>
          </>
        )}
      </ScrollView>

      <UnlockWalletDialog
        visible={showUnlockDialog}
        onClose={() => setShowUnlockDialog(false)}
        onSuccess={handleUnlockSuccess}
      />

      <TransactionStatusDialog
        visible={showTxStatus}
        status={txStatus}
        onClose={() => setShowTxStatus(false)}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
  navBar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 50,
    paddingHorizontal: 16,
    paddingBottom: 12,
    backgroundColor: '#FFF',
    borderBottomWidth: 1,
    borderBottomColor: '#F0F0F0',
  },
  backButton: {
    padding: 4,
  },
  navTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
  },
  content: {
    flex: 1,
  },
  scrollContent: {
    padding: 16,
    paddingBottom: 40,
  },
  card: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.04,
    shadowRadius: 8,
    elevation: 2,
  },
  cardHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
    marginBottom: 12,
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#333',
  },
  cardDesc: {
    fontSize: 13,
    color: '#999',
    lineHeight: 18,
    marginBottom: 16,
  },
  keyBox: {
    backgroundColor: '#F9F9F9',
    padding: 12,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0',
  },
  keyText: {
    fontSize: 12,
    fontFamily: 'monospace',
    color: '#666',
    marginBottom: 8,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  statusText: {
    fontSize: 12,
    color: '#27AE60',
    fontWeight: '500',
  },
  primaryButton: {
    backgroundColor: THEME_COLOR,
    height: 44,
    borderRadius: 8,
    justifyContent: 'center',
    alignItems: 'center',
  },
  primaryButtonText: {
    color: '#FFF',
    fontSize: 15,
    fontWeight: '600',
  },
  subTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 10,
  },
  chartSelector: {
    marginBottom: 16,
  },
  chartScroll: {
    flexDirection: 'row',
  },
  chartChip: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#F5F5F7',
    borderRadius: 16,
    marginRight: 8,
    borderWidth: 1,
    borderColor: '#E0E0E0',
  },
  chartChipActive: {
    backgroundColor: THEME_COLOR + '20',
    borderColor: THEME_COLOR,
  },
  chartChipText: {
    fontSize: 13,
    color: '#666',
  },
  chartChipTextActive: {
    color: THEME_COLOR,
    fontWeight: '600',
  },
  authList: {
    marginTop: 8,
    paddingTop: 16,
    borderTopWidth: 1,
    borderTopColor: '#F0F0F0',
  },
  authItem: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#F9F9F9',
  },
  authInfo: {
    flex: 1,
    marginRight: 12,
  },
  authAccount: {
    fontSize: 13,
    color: '#333',
    fontFamily: 'monospace',
    marginBottom: 4,
  },
  authTags: {
    flexDirection: 'row',
    gap: 6,
  },
  authTag: {
    paddingHorizontal: 6,
    paddingVertical: 2,
    backgroundColor: '#E3F2FD',
    borderRadius: 4,
  },
  authTagText: {
    fontSize: 10,
    color: '#1976D2',
  },
  revokeBtn: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    backgroundColor: '#FFEBEE',
    borderRadius: 4,
  },
  revokeBtnText: {
    fontSize: 12,
    color: '#E74C3C',
    fontWeight: '500',
  },
  emptyText: {
    fontSize: 14,
    color: '#CCC',
    textAlign: 'center',
    marginVertical: 20,
  },
  emptyTextSmall: {
    fontSize: 12,
    color: '#CCC',
    textAlign: 'center',
    marginVertical: 10,
  },
  notice: {
    flexDirection: 'row',
    paddingHorizontal: 8,
    gap: 8,
  },
  noticeText: {
    flex: 1,
    fontSize: 12,
    color: '#999',
    lineHeight: 18,
  },
});

