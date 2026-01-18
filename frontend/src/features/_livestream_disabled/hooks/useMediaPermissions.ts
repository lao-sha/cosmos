// frontend/src/features/livestream/hooks/useMediaPermissions.ts

import { useState, useCallback, useEffect } from 'react';
import { Alert, Linking, Platform } from 'react-native';
import { Camera } from 'expo-camera';
import { Audio } from 'expo-av';

export type PermissionStatus = 'undetermined' | 'granted' | 'denied';

interface MediaPermissions {
  camera: PermissionStatus;
  microphone: PermissionStatus;
}

interface UseMediaPermissionsResult {
  permissions: MediaPermissions;
  isLoading: boolean;
  hasAllPermissions: boolean;
  checkPermissions: () => Promise<MediaPermissions>;
  requestPermissions: () => Promise<boolean>;
  openSettings: () => Promise<void>;
}

export function useMediaPermissions(): UseMediaPermissionsResult {
  const [permissions, setPermissions] = useState<MediaPermissions>({
    camera: 'undetermined',
    microphone: 'undetermined',
  });
  const [isLoading, setIsLoading] = useState(true);

  // 检查权限状态
  const checkPermissions = useCallback(async (): Promise<MediaPermissions> => {
    try {
      const [cameraStatus, audioStatus] = await Promise.all([
        Camera.getCameraPermissionsAsync(),
        Audio.getPermissionsAsync(),
      ]);

      const result: MediaPermissions = {
        camera: cameraStatus.granted
          ? 'granted'
          : cameraStatus.canAskAgain
          ? 'undetermined'
          : 'denied',
        microphone: audioStatus.granted
          ? 'granted'
          : audioStatus.canAskAgain
          ? 'undetermined'
          : 'denied',
      };

      setPermissions(result);
      return result;
    } catch (error) {
      console.error('Check permissions error:', error);
      return { camera: 'undetermined', microphone: 'undetermined' };
    }
  }, []);

  // 请求权限
  const requestPermissions = useCallback(async (): Promise<boolean> => {
    setIsLoading(true);

    try {
      // 先检查当前状态
      const current = await checkPermissions();

      // 如果已经全部授权
      if (current.camera === 'granted' && current.microphone === 'granted') {
        return true;
      }

      // 如果有权限被永久拒绝
      if (current.camera === 'denied' || current.microphone === 'denied') {
        showPermissionDeniedAlert(current);
        return false;
      }

      // 请求相机权限
      let cameraGranted = current.camera === 'granted';
      if (!cameraGranted) {
        const cameraResult = await Camera.requestCameraPermissionsAsync();
        cameraGranted = cameraResult.granted;

        if (!cameraGranted && !cameraResult.canAskAgain) {
          setPermissions((prev) => ({ ...prev, camera: 'denied' }));
        }
      }

      // 请求麦克风权限
      let microphoneGranted = current.microphone === 'granted';
      if (!microphoneGranted) {
        const audioResult = await Audio.requestPermissionsAsync();
        microphoneGranted = audioResult.granted;

        if (!microphoneGranted && !audioResult.canAskAgain) {
          setPermissions((prev) => ({ ...prev, microphone: 'denied' }));
        }
      }

      // 更新状态
      await checkPermissions();

      // 检查结果
      if (!cameraGranted || !microphoneGranted) {
        const deniedPermissions: string[] = [];
        if (!cameraGranted) deniedPermissions.push('相机');
        if (!microphoneGranted) deniedPermissions.push('麦克风');

        Alert.alert(
          '权限不足',
          `需要${deniedPermissions.join('和')}权限才能开播`,
          [{ text: '知道了' }]
        );
        return false;
      }

      return true;
    } catch (error) {
      console.error('Request permissions error:', error);
      Alert.alert('错误', '请求权限时发生错误');
      return false;
    } finally {
      setIsLoading(false);
    }
  }, [checkPermissions]);

  // 打开系统设置
  const openSettings = useCallback(async () => {
    try {
      await Linking.openSettings();
    } catch (error) {
      console.error('Open settings error:', error);
    }
  }, []);

  // 显示权限被拒绝的提示
  const showPermissionDeniedAlert = useCallback(
    (current: MediaPermissions) => {
      const deniedPermissions: string[] = [];
      if (current.camera === 'denied') deniedPermissions.push('相机');
      if (current.microphone === 'denied') deniedPermissions.push('麦克风');

      Alert.alert(
        '权限被拒绝',
        `${deniedPermissions.join('和')}权限已被拒绝，请在系统设置中开启`,
        [
          { text: '取消', style: 'cancel' },
          { text: '去设置', onPress: openSettings },
        ]
      );
    },
    [openSettings]
  );

  // 初始化时检查权限
  useEffect(() => {
    checkPermissions().finally(() => setIsLoading(false));
  }, [checkPermissions]);

  return {
    permissions,
    isLoading,
    hasAllPermissions:
      permissions.camera === 'granted' && permissions.microphone === 'granted',
    checkPermissions,
    requestPermissions,
    openSettings,
  };
}
