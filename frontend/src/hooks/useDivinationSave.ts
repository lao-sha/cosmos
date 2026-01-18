/**
 * 占卜保存到链上的自定义Hook
 * 封装上链逻辑，供所有占卜页面复用
 */

import { useState } from 'react';
import { Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { divinationService, DivinationType } from '@/services/divination.service';
import { isSignerUnlocked, unlockWalletForSigning } from '@/lib/signer';

export interface UseDivinationSaveOptions {
  divinationType: DivinationType;
  historyRoute?: string; // 历史记录页面路由
}

export function useDivinationSave(options: UseDivinationSaveOptions) {
  const router = useRouter();
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showTxStatus, setShowTxStatus] = useState(false);
  const [txStatus, setTxStatus] = useState('准备中...');
  const [saving, setSaving] = useState(false);

  /**
   * 保存到链上
   */
  const saveToChain = async (resultData: any) => {
    if (!resultData) {
      Alert.alert('提示', '请先进行占卜');
      return;
    }

    try {
      // 检查钱包是否解锁
      if (!isSignerUnlocked()) {
        setShowUnlockDialog(true);
        return;
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
    } catch (error: any) {
      console.error('保存到链上失败:', error);
      setTxStatus('保存失败');
      setTimeout(() => {
        setShowTxStatus(false);
        Alert.alert('保存失败', error.message || '未知错误');
      }, 1500);
    } finally {
      setSaving(false);
    }
  };

  /**
   * 处理钱包解锁
   */
  const handleUnlockSuccess = async (password: string, resultData: any) => {
    try {
      await unlockWalletForSigning(password);
      setShowUnlockDialog(false);
      // 解锁成功后继续保存
      setTimeout(() => {
        saveToChain(resultData);
      }, 300);
    } catch (error: any) {
      Alert.alert('解锁失败', error.message || '密码错误');
    }
  };

  return {
    // 状态
    showUnlockDialog,
    showTxStatus,
    txStatus,
    saving,

    // 方法
    saveToChain,
    handleUnlockSuccess,
    setShowUnlockDialog,
    setShowTxStatus,
  };
}
