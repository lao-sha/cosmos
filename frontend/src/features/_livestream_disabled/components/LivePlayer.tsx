// frontend/src/features/livestream/components/LivePlayer.tsx

import React from 'react';
import { View, StyleSheet, Dimensions, ActivityIndicator, Text } from 'react-native';
import { VideoView } from '@livekit/react-native';
import { Track } from '@livekit/react-native';
import { DanmakuOverlay } from './DanmakuOverlay';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

interface LivePlayerProps {
  videoTrack: Track | null;
  isLoading?: boolean;
  showDanmaku?: boolean;
  roomId: number;
}

export function LivePlayer({
  videoTrack,
  isLoading = false,
  showDanmaku = true,
  roomId,
}: LivePlayerProps) {
  if (isLoading) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size="large" color="#FF4757" />
        <Text style={styles.loadingText}>正在连接直播...</Text>
      </View>
    );
  }

  if (!videoTrack) {
    return (
      <View style={styles.container}>
        <Text style={styles.offlineText}>主播暂时离开</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <VideoView
        style={styles.video}
        videoTrack={videoTrack}
        objectFit="contain"
      />
      {showDanmaku && <DanmakuOverlay roomId={roomId} />}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    width: SCREEN_WIDTH,
    aspectRatio: 16 / 9,
    backgroundColor: '#000',
    justifyContent: 'center',
    alignItems: 'center',
  },
  video: {
    width: '100%',
    height: '100%',
  },
  loadingText: {
    color: '#FFF',
    marginTop: 12,
    fontSize: 14,
  },
  offlineText: {
    color: '#999',
    fontSize: 16,
  },
});
