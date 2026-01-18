// frontend/src/features/livestream/components/LiveControls.tsx

import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';

interface LiveControlsProps {
  isMuted: boolean;
  isCameraOn: boolean;
  onToggleMute: () => void;
  onToggleCamera: () => void;
  onSwitchCamera: () => void;
  onEndLive: () => void;
}

export function LiveControls({
  isMuted,
  isCameraOn,
  onToggleMute,
  onToggleCamera,
  onSwitchCamera,
  onEndLive,
}: LiveControlsProps) {
  return (
    <View style={styles.container}>
      <TouchableOpacity style={styles.controlBtn} onPress={onSwitchCamera}>
        <Ionicons name="camera-reverse" size={24} color="#FFF" />
        <Text style={styles.controlText}>翻转</Text>
      </TouchableOpacity>

      <TouchableOpacity
        style={[styles.controlBtn, isMuted && styles.controlBtnActive]}
        onPress={onToggleMute}
      >
        <Ionicons
          name={isMuted ? 'mic-off' : 'mic'}
          size={24}
          color={isMuted ? '#FF4757' : '#FFF'}
        />
        <Text style={[styles.controlText, isMuted && styles.controlTextActive]}>
          {isMuted ? '已静音' : '静音'}
        </Text>
      </TouchableOpacity>

      <TouchableOpacity
        style={[styles.controlBtn, !isCameraOn && styles.controlBtnActive]}
        onPress={onToggleCamera}
      >
        <Ionicons
          name={isCameraOn ? 'videocam' : 'videocam-off'}
          size={24}
          color={!isCameraOn ? '#FF4757' : '#FFF'}
        />
        <Text style={[styles.controlText, !isCameraOn && styles.controlTextActive]}>
          {isCameraOn ? '关闭' : '已关闭'}
        </Text>
      </TouchableOpacity>

      <TouchableOpacity style={styles.endBtn} onPress={onEndLive}>
        <Ionicons name="close-circle" size={24} color="#FFF" />
        <Text style={styles.endText}>结束</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    paddingVertical: 16,
    paddingHorizontal: 20,
    backgroundColor: 'rgba(0,0,0,0.5)',
  },
  controlBtn: {
    alignItems: 'center',
    padding: 12,
    borderRadius: 12,
    backgroundColor: 'rgba(255,255,255,0.1)',
    minWidth: 70,
  },
  controlBtnActive: {
    backgroundColor: 'rgba(255,71,87,0.2)',
  },
  controlText: {
    color: '#FFF',
    fontSize: 12,
    marginTop: 4,
  },
  controlTextActive: {
    color: '#FF4757',
  },
  endBtn: {
    alignItems: 'center',
    padding: 12,
    borderRadius: 12,
    backgroundColor: '#FF4757',
    minWidth: 70,
  },
  endText: {
    color: '#FFF',
    fontSize: 12,
    marginTop: 4,
    fontWeight: 'bold',
  },
});
