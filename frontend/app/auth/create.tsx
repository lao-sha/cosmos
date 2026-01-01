/**
 * 星尘玄鉴 - 创建钱包流程协调器
 * 管理 4 步创建流程：设置密码 → 备份助记词 → 验证助记词 → 创建成功
 */

import { useState } from 'react';
import { Alert } from 'react-native';
import { useRouter } from 'expo-router';
import { useWalletStore } from '@/stores';
import {
  SetPasswordPage,
  BackupMnemonicPage,
  VerifyMnemonicPage,
  WalletCreatedPage,
} from '@/features/auth';

type Step = 'password' | 'backup' | 'verify' | 'success';

export default function CreateWalletPage() {
  const router = useRouter();
  const { createWallet, isLoading, error } = useWalletStore();

  const [currentStep, setCurrentStep] = useState<Step>('password');
  const [password, setPassword] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [address, setAddress] = useState('');

  /**
   * 步骤1: 设置密码完成
   * - 保存密码
   * - 生成助记词和地址
   * - 进入备份助记词步骤
   */
  const handlePasswordSet = async (pwd: string) => {
    try {
      setPassword(pwd);

      // 调用 store 创建钱包，获取助记词
      const generatedMnemonic = await createWallet(pwd);
      setMnemonic(generatedMnemonic);

      // 获取地址（从 store 中）
      const { address: walletAddress } = useWalletStore.getState();
      setAddress(walletAddress || '');

      // 进入备份步骤
      setCurrentStep('backup');
    } catch (err) {
      Alert.alert('创建失败', error || '未知错误');
    }
  };

  /**
   * 步骤2: 备份助记词完成
   * - 进入验证步骤
   */
  const handleBackupComplete = () => {
    setCurrentStep('verify');
  };

  /**
   * 步骤3: 验证助记词成功
   * - 进入成功页面
   */
  const handleVerifySuccess = () => {
    setCurrentStep('success');
  };

  /**
   * 步骤4: 完成创建
   * - 跳转到主页
   */
  const handleComplete = () => {
    router.replace('/(tabs)');
  };

  /**
   * 返回上一步
   */
  const handleBack = () => {
    switch (currentStep) {
      case 'password':
        router.back();
        break;
      case 'backup':
        // 备份页面不允许返回（已生成钱包）
        Alert.alert(
          '确认返回',
          '返回将放弃当前创建的钱包，确定吗？',
          [
            { text: '取消', style: 'cancel' },
            {
              text: '确定',
              style: 'destructive',
              onPress: () => {
                setMnemonic('');
                setAddress('');
                setCurrentStep('password');
              },
            },
          ]
        );
        break;
      case 'verify':
        setCurrentStep('backup');
        break;
      case 'success':
        // 成功页面不允许返回
        break;
    }
  };

  // 渲染当前步骤
  switch (currentStep) {
    case 'password':
      return (
        <SetPasswordPage
          onPasswordSet={handlePasswordSet}
          onBack={handleBack}
          isLoading={isLoading}
        />
      );

    case 'backup':
      return (
        <BackupMnemonicPage
          mnemonic={mnemonic}
          address={address}
          onBackupComplete={handleBackupComplete}
          onBack={handleBack}
        />
      );

    case 'verify':
      return (
        <VerifyMnemonicPage
          mnemonic={mnemonic}
          onVerifySuccess={handleVerifySuccess}
          onBack={handleBack}
        />
      );

    case 'success':
      return (
        <WalletCreatedPage
          address={address}
          onComplete={handleComplete}
        />
      );

    default:
      return null;
  }
}
