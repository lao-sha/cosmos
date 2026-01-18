/**
 * 图片选择器组件
 * 支持从相册选择或拍照
 */

import React from 'react';
import { Alert, Platform } from 'react-native';
import * as ImagePicker from 'expo-image-picker';

export interface ImagePickerResult {
  uri: string;
  width: number;
  height: number;
  type: 'image';
  fileName?: string;
  fileSize?: number;
}

export class ImagePickerService {
  /**
   * 请求相机权限
   */
  static async requestCameraPermission(): Promise<boolean> {
    if (Platform.OS === 'web') {
      return true;
    }

    const { status } = await ImagePicker.requestCameraPermissionsAsync();
    if (status !== 'granted') {
      Alert.alert('权限不足', '需要相机权限才能拍照');
      return false;
    }
    return true;
  }

  /**
   * 请求相册权限
   */
  static async requestMediaLibraryPermission(): Promise<boolean> {
    if (Platform.OS === 'web') {
      return true;
    }

    const { status } = await ImagePicker.requestMediaLibraryPermissionsAsync();
    if (status !== 'granted') {
      Alert.alert('权限不足', '需要相册权限才能选择图片');
      return false;
    }
    return true;
  }

  /**
   * 从相册选择图片
   */
  static async pickFromLibrary(): Promise<ImagePickerResult | null> {
    const hasPermission = await this.requestMediaLibraryPermission();
    if (!hasPermission) {
      return null;
    }

    try {
      const result = await ImagePicker.launchImageLibraryAsync({
        mediaTypes: ['images'],
        allowsEditing: true,
        aspect: [4, 3],
        quality: 0.8,
        allowsMultipleSelection: false,
      });

      if (result.canceled || !result.assets || result.assets.length === 0) {
        return null;
      }

      const asset = result.assets[0];
      return {
        uri: asset.uri,
        width: asset.width,
        height: asset.height,
        type: 'image',
        fileName: asset.fileName,
        fileSize: asset.fileSize,
      };
    } catch (error) {
      Alert.alert('错误', '选择图片失败');
      return null;
    }
  }

  /**
   * 拍照
   */
  static async takePhoto(): Promise<ImagePickerResult | null> {
    const hasPermission = await this.requestCameraPermission();
    if (!hasPermission) {
      return null;
    }

    try {
      const result = await ImagePicker.launchCameraAsync({
        allowsEditing: true,
        aspect: [4, 3],
        quality: 0.8,
      });

      if (result.canceled || !result.assets || result.assets.length === 0) {
        return null;
      }

      const asset = result.assets[0];
      return {
        uri: asset.uri,
        width: asset.width,
        height: asset.height,
        type: 'image',
        fileName: asset.fileName,
        fileSize: asset.fileSize,
      };
    } catch (error) {
      Alert.alert('错误', '拍照失败');
      return null;
    }
  }

  /**
   * 显示选择器菜单
   */
  static async showPicker(): Promise<ImagePickerResult | null> {
    return new Promise((resolve) => {
      Alert.alert(
        '选择图片',
        '请选择图片来源',
        [
          {
            text: '拍照',
            onPress: async () => {
              const result = await this.takePhoto();
              resolve(result);
            },
          },
          {
            text: '从相册选择',
            onPress: async () => {
              const result = await this.pickFromLibrary();
              resolve(result);
            },
          },
          {
            text: '取消',
            style: 'cancel',
            onPress: () => resolve(null),
          },
        ],
        { cancelable: true, onDismiss: () => resolve(null) }
      );
    });
  }
}
