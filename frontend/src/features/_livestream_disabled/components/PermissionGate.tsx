// frontend/src/features/livestream/components/PermissionGate.tsx

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import {
  useMediaPermissions,
  PermissionStatus,
} from '../hooks/useMediaPermissions';

interface PermissionGateProps {
  children: React.ReactNode;
  onPermissionDenied?: () => void;
}

export function PermissionGate({
  children,
  onPermissionDenied,
}: PermissionGateProps) {
  const {
    permissions,
    isLoading,
    hasAllPermissions,
    requestPermissions,
    openSettings,
  } = useMediaPermissions();

  if (isLoading) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size="large" color="#FF4757" />
        <Text style={styles.text}>检查权限中...</Text>
      </View>
    );
  }

  if (hasAllPermissions) {
    return <>{children}</>;
  }

  const hasDenied =
    permissions.camera === 'denied' || permissions.microphone === 'denied';

  return (
    <View style={styles.container}>
      <Ionicons name="videocam-off" size={64} color="#666" />
      <Text style={styles.title}>需要相机和麦克风权限</Text>
      <Text style={styles.description}>
        开播需要使用相机和麦克风，请授予相关权限
      </Text>

      <View style={styles.permissionList}>
        <PermissionItem
          icon="camera"
          label="相机"
          status={permissions.camera}
        />
        <PermissionItem
          icon="mic"
          label="麦克风"
          status={permissions.microphone}
        />
      </View>

      {hasDenied ? (
        <TouchableOpacity style={styles.button} onPress={openSettings}>
          <Ionicons name="settings-outline" size={20} color="#FFF" />
          <Text style={styles.buttonText}>去系统设置开启</Text>
        </TouchableOpacity>
      ) : (
        <TouchableOpacity style={styles.button} onPress={requestPermissions}>
          <Text style={styles.buttonText}>授予权限</Text>
        </TouchableOpacity>
      )}

      <TouchableOpacity style={styles.cancelButton} onPress={onPermissionDenied}>
        <Text style={styles.cancelText}>暂不开播</Text>
      </TouchableOpacity>
    </View>
  );
}

function PermissionItem({
  icon,
  label,
  status,
}: {
  icon: string;
  label: string;
  status: PermissionStatus;
}) {
  const getStatusIcon = () => {
    switch (status) {
      case 'granted':
        return <Ionicons name="checkmark-circle" size={20} color="#4CAF50" />;
      case 'denied':
        return <Ionicons name="close-circle" size={20} color="#F44336" />;
      default:
        return <Ionicons name="help-circle" size={20} color="#FFC107" />;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'granted':
        return '已授权';
      case 'denied':
        return '已拒绝';
      default:
        return '未授权';
    }
  };

  return (
    <View style={styles.permissionItem}>
      <Ionicons name={icon as any} size={24} color="#FFF" />
      <Text style={styles.permissionLabel}>{label}</Text>
      <View style={styles.permissionStatus}>
        {getStatusIcon()}
        <Text
          style={[
            styles.permissionStatusText,
            status === 'granted' && styles.granted,
            status === 'denied' && styles.denied,
          ]}
        >
          {getStatusText()}
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#1A1A2E',
    padding: 24,
  },
  title: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#FFF',
    marginTop: 16,
    marginBottom: 8,
  },
  description: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
    marginBottom: 24,
  },
  text: {
    color: '#999',
    marginTop: 12,
  },
  permissionList: {
    width: '100%',
    marginBottom: 24,
  },
  permissionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#252540',
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
  },
  permissionLabel: {
    flex: 1,
    color: '#FFF',
    fontSize: 16,
    marginLeft: 12,
  },
  permissionStatus: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  permissionStatusText: {
    color: '#FFC107',
    fontSize: 14,
    marginLeft: 4,
  },
  granted: {
    color: '#4CAF50',
  },
  denied: {
    color: '#F44336',
  },
  button: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#FF4757',
    paddingHorizontal: 32,
    paddingVertical: 14,
    borderRadius: 24,
    width: '100%',
    gap: 8,
  },
  buttonText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: '600',
  },
  cancelButton: {
    marginTop: 16,
    padding: 12,
  },
  cancelText: {
    color: '#999',
    fontSize: 14,
  },
});
