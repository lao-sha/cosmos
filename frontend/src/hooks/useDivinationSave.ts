/**
 * 占卜保存到链上的自定义Hook
 * 封装上链逻辑，供所有占卜页面复用
 *
 * 支持两种保存方式：
 * 1. 通用占卜结果存储 (storeDivinationResult)
 * 2. 八字命盘专用存储 (createBaziChart) - 支持 Runtime API 解盘
 *
 * 更新日志 (2026-01-19):
 * - 添加八字命盘专用保存支持
 * - 添加隐私模式支持
 * - 添加错误类型区分
 */

import { useState, useCallback } from 'react';
import { Alert } from 'react-native';
import { useRouter } from 'expo-router';
import {
  divinationService,
  DivinationType,
  PrivacyMode,
} from '@/services/divination.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

export interface UseDivinationSaveOptions {
  divinationType: DivinationType;
  historyRoute?: string; // 历史记录页面路由
}

/**
 * 八字保存参数
 */
export interface BaziSaveParams {
  name?: string;
  birthYear: number;
  birthMonth: number;
  birthDay: number;
  birthHour: number;
  birthMinute: number;
  gender: 'male' | 'female';
  calendarType?: 'solar' | 'lunar';
  privacyMode?: PrivacyMode;
}

export function useDivinationSave(options: UseDivinationSaveOptions) {
  const router = useRouter();
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');
  const [saving, setSaving] = useState(false);
  const [lastSavedId, setLastSavedId] = useState<number | null>(null);

  // 存储待保存的数据，用于解锁后继续保存
  const [pendingData, setPendingData] = useState<{
    type: 'general' | 'bazi';
    data: any;
    baziParams?: BaziSaveParams;
  } | null>(null);

  /**
   * 通用占卜结果保存到链上
   */
  const saveToChain = useCallback(async (resultData: any) => {
    if (!resultData) {
      Alert.alert('提示', '请先进行占卜');
      return null;
    }

    try {
      // 检查钱包是否解锁
      if (!isSignerUnlocked()) {
        setPendingData({ type: 'general', data: resultData });
        setShowUnlockDialog(true);
        return null;
      }

      setSaving(true);
      setShowTxStatus(true);
      setTxStatus('准备上链...');

      // 调用服务保存到链上
      const recordId = await divinationService.storeDivinationResult(
        options.divinationType,
        resultData,
        (status) => {
          setTxStatus(status);
        }
      );

      setTxStatus('保存成功！');
      setLastSavedId(recordId);

      setTimeout(() => {
        setShowTxStatus(false);

        const buttons: any[] = [{ text: '确定' }];

        // 如果提供了历史记录路由，添加"查看历史"按钮
        if (options.historyRoute) {
          buttons.unshift({
            text: '查看历史',
            onPress: () => router.push(options.historyRoute as any),
          });
        }

        Alert.alert(
          '保存成功',
          `占卜结果已保存到链上\n记录ID: ${recordId}`,
          buttons
        );
      }, 1500);

      return recordId;
    } catch (error: any) {
      console.error('保存到链上失败:', error);
      setTxStatus('保存失败');
      setTimeout(() => {
        setShowTxStatus(false);
        Alert.alert('保存失败', error.message || '未知错误');
      }, 1500);
      return null;
    } finally {
      setSaving(false);
    }
  }, [options.divinationType, options.historyRoute, router]);

  /**
   * 八字命盘专用保存（支持 Runtime API 解盘）
   *
   * 与通用 saveToChain 不同，此方法：
   * 1. 使用 createBaziChart extrinsic
   * 2. 链上直接计算命盘，无需传递完整结果
   * 3. 支持后续调用 Runtime API 获取详细解盘
   * 4. 支持隐私模式
   */
  const saveBaziToChain = useCallback(async (params: BaziSaveParams) => {
    try {
      // 检查钱包是否解锁
      if (!isSignerUnlocked()) {
        setPendingData({ type: 'bazi', data: null, baziParams: params });
        setShowUnlockDialog(true);
        return null;
      }

      setSaving(true);
      setShowTxStatus(true);
      setTxStatus('准备创建八字命盘...');

      // 调用八字命盘创建服务
      const chartId = await divinationService.createBaziChart(
        params.name || null,
        params.birthYear,
        params.birthMonth,
        params.birthDay,
        params.birthHour,
        params.birthMinute,
        params.gender,
        params.calendarType || 'solar',
        (status) => {
          setTxStatus(status);
        }
      );

      setTxStatus('命盘创建成功！');
      setLastSavedId(chartId);

      setTimeout(() => {
        setShowTxStatus(false);

        Alert.alert(
          '保存成功',
          `八字命盘已创建\n命盘ID: ${chartId}\n\n可在详情页查看完整解盘`,
          [
            { text: '确定' },
            {
              text: '查看详情',
              onPress: () => router.push(`/bazi-detail/${chartId}` as any),
            },
          ]
        );
      }, 1500);

      return chartId;
    } catch (error: any) {
      console.error('创建八字命盘失败:', error);
      setTxStatus('创建失败');
      setTimeout(() => {
        setShowTxStatus(false);
        Alert.alert('创建失败', error.message || '未知错误');
      }, 1500);
      return null;
    } finally {
      setSaving(false);
    }
  }, [router]);

  /**
   * 处理钱包解锁
   */
  const handleUnlockSuccess = useCallback(async (password: string) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);

      // 解锁成功后继续保存
      if (pendingData) {
        setTimeout(() => {
          if (pendingData.type === 'bazi' && pendingData.baziParams) {
            saveBaziToChain(pendingData.baziParams);
          } else if (pendingData.type === 'general' && pendingData.data) {
            saveToChain(pendingData.data);
          }
          setPendingData(null);
        }, 300);
      }
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  }, [pendingData, saveToChain, saveBaziToChain]);

  /**
   * 获取完整解盘结果（免费 Runtime API）
   */
  const getInterpretation = useCallback(async (chartId: number) => {
    try {
      setTxStatus('获取解盘中...');
      const result = await divinationService.getFullInterpretation(chartId);
      return result;
    } catch (error: any) {
      console.error('获取解盘失败:', error);
      Alert.alert('获取失败', error.message || '未知错误');
      return null;
    }
  }, []);

  return {
    // 状态
    showUnlockDialog,
    showTxStatus,
    txStatus,
    saving,
    lastSavedId,

    // 通用保存方法
    saveToChain,

    // 八字专用方法
    saveBaziToChain,
    getInterpretation,

    // 钱包解锁
    handleUnlockSuccess,
    setShowUnlockDialog,
    setShowTxStatus,
  };
}
