/**
 * 星尘玄鉴 - 生物识别服务
 * 封装 expo-local-authentication
 */

import * as LocalAuthentication from 'expo-local-authentication';

/**
 * 生物识别类型
 */
export enum BiometricType {
  None = 'none',
  Fingerprint = 'fingerprint',
  FaceId = 'face_id',
  Iris = 'iris',
}

/**
 * 生物识别服务
 */
export class BiometricService {
  /**
   * 检查设备是否支持生物识别
   */
  async isSupported(): Promise<boolean> {
    try {
      const compatible = await LocalAuthentication.hasHardwareAsync();
      const enrolled = await LocalAuthentication.isEnrolledAsync();
      return compatible && enrolled;
    } catch {
      return false;
    }
  }

  /**
   * 获取支持的生物识别类型
   */
  async getSupportedTypes(): Promise<BiometricType[]> {
    try {
      const types = await LocalAuthentication.supportedAuthenticationTypesAsync();

      return types
        .map((type) => {
          switch (type) {
            case LocalAuthentication.AuthenticationType.FINGERPRINT:
              return BiometricType.Fingerprint;
            case LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION:
              return BiometricType.FaceId;
            case LocalAuthentication.AuthenticationType.IRIS:
              return BiometricType.Iris;
            default:
              return BiometricType.None;
          }
        })
        .filter((t) => t !== BiometricType.None);
    } catch {
      return [];
    }
  }

  /**
   * 获取生物识别显示名称
   */
  async getBiometricName(): Promise<string> {
    const types = await this.getSupportedTypes();

    if (types.includes(BiometricType.FaceId)) {
      return 'Face ID';
    } else if (types.includes(BiometricType.Fingerprint)) {
      return '指纹';
    } else if (types.includes(BiometricType.Iris)) {
      return '虹膜';
    }

    return '生物识别';
  }

  /**
   * 请求生物识别认证
   */
  async authenticate(options?: {
    promptMessage?: string;
    cancelLabel?: string;
    fallbackLabel?: string;
    disableDeviceFallback?: boolean;
  }): Promise<{ success: boolean; error?: string }> {
    try {
      const result = await LocalAuthentication.authenticateAsync({
        promptMessage: options?.promptMessage || '验证身份以解锁钱包',
        cancelLabel: options?.cancelLabel || '取消',
        fallbackLabel: options?.fallbackLabel || '使用 PIN',
        disableDeviceFallback: options?.disableDeviceFallback ?? false,
      });

      if (result.success) {
        return { success: true };
      }

      // 处理错误
      let errorMessage = '认证失败';
      switch (result.error) {
        case 'user_cancel':
          errorMessage = '用户取消';
          break;
        case 'user_fallback':
          errorMessage = '用户选择备用方式';
          break;
        case 'system_cancel':
          errorMessage = '系统取消';
          break;
        case 'not_enrolled':
          errorMessage = '未设置生物识别';
          break;
        case 'lockout':
          errorMessage = '尝试次数过多，请稍后再试';
          break;
      }

      return { success: false, error: errorMessage };
    } catch (error) {
      return { success: false, error: (error as Error).message };
    }
  }

  /**
   * 检查安全级别
   */
  async getSecurityLevel(): Promise<'none' | 'weak' | 'strong'> {
    try {
      const level = await LocalAuthentication.getEnrolledLevelAsync();

      switch (level) {
        case LocalAuthentication.SecurityLevel.NONE:
          return 'none';
        case LocalAuthentication.SecurityLevel.SECRET:
          return 'weak';
        case LocalAuthentication.SecurityLevel.BIOMETRIC_STRONG:
        case LocalAuthentication.SecurityLevel.BIOMETRIC_WEAK:
          return 'strong';
        default:
          return 'none';
      }
    } catch {
      return 'none';
    }
  }
}

// 单例
export const biometricService = new BiometricService();
