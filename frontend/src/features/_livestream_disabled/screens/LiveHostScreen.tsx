// frontend/src/features/livestream/screens/LiveHostScreen.tsx

import React, { useEffect, useState, useCallback, useRef } from 'react';
import {
  View,
  StyleSheet,
  StatusBar,
  Alert,
  BackHandler,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { VideoView, Track } from '@livekit/react-native';
import { useLivestreamStore } from '@/stores/livestream.store';
import { useWalletStore } from '@/stores/wallet.store';
import {
  LiveControls,
  LiveStats,
  LiveChat,
  GiftAnimation,
  CoHostPanel,
} from '../components';
import { LiveKitService } from '../services/livekit.service';
import {
  getLivestreamService,
  initLivestreamService,
} from '../services/livestream.service';
import type { GiftAnimationItem, DataChannelMessage, CoHostRequest } from '../types';

export function LiveHostScreen() {
  const router = useRouter();
  const { roomId: roomIdParam } = useLocalSearchParams<{ roomId: string }>();
  const roomId = Number(roomIdParam);

  const { currentWallet } = useWalletStore();
  const {
    currentRoom,
    setCurrentRoom,
    setIsInRoom,
    setIsHost,
    setIsLive,
    gifts,
    setGifts,
    addMessage,
    clearMessages,
    coHostRequests,
    addCoHostRequest,
    removeCoHostRequest,
    earnings,
    setEarnings,
  } = useLivestreamStore();

  const [liveKitService, setLiveKitService] = useState<LiveKitService | null>(null);
  const [localVideoTrack, setLocalVideoTrack] = useState<Track | null>(null);
  const [isMuted, setIsMuted] = useState(false);
  const [isCameraOn, setIsCameraOn] = useState(true);
  const [duration, setDuration] = useState(0);
  const [viewerCount, setViewerCount] = useState(0);
  const [giftAnimations, setGiftAnimations] = useState<GiftAnimationItem[]>([]);
  const [showCoHostPanel, setShowCoHostPanel] = useState(false);

  const durationTimerRef = useRef<NodeJS.Timeout | null>(null);

  // 初始化直播
  const initLive = useCallback(async () => {
    if (!currentWallet?.address || !roomId) return;

    try {
      const service = initLivestreamService(currentWallet.address);
      await service.init();

      // 获取直播间信息
      const room = await service.getRoomInfo(roomId);
      setCurrentRoom(room);
      setIsHost(true);

      // 加载礼物列表
      const giftList = await service.getGifts();
      setGifts(giftList);

      // 获取主播收益
      const hostEarnings = await service.getHostEarnings();
      setEarnings(hostEarnings);

      // 连接 LiveKit 并推流
      await startPublishing(service);

      // 开始直播
      await service.startLive(roomId);
      setIsLive(true);

      // 开始计时
      durationTimerRef.current = setInterval(() => {
        setDuration((prev) => prev + 1);
      }, 1000);
    } catch (error) {
      console.error('初始化直播失败:', error);
      Alert.alert('错误', '初始化直播失败');
      router.back();
    }
  }, [currentWallet?.address, roomId]);

  // 开始推流
  const startPublishing = async (service: ReturnType<typeof getLivestreamService>) => {
    try {
      const token = await service.getPublisherToken(roomId);
      const livekit = new LiveKitService();

      // 设置回调
      livekit.setOnParticipantConnected(() => {
        setViewerCount(livekit.getParticipantCount());
      });

      livekit.setOnParticipantDisconnected(() => {
        setViewerCount(livekit.getParticipantCount());
      });

      livekit.setOnDataReceived((data) => {
        handleDataMessage(data);
      });

      await livekit.connect(token);
      await livekit.enableCameraAndMicrophone();

      // 获取本地视频轨道
      const localParticipant = livekit.getLocalParticipant();
      if (localParticipant) {
        const videoPublication = localParticipant.getTrackPublication(Track.Source.Camera);
        if (videoPublication?.track) {
          setLocalVideoTrack(videoPublication.track);
        }
      }

      setLiveKitService(livekit);
      setIsInRoom(true);
    } catch (error) {
      console.error('推流失败:', error);
      throw error;
    }
  };

  // 处理 DataChannel 消息
  const handleDataMessage = (data: Uint8Array) => {
    try {
      const message: DataChannelMessage = JSON.parse(
        new TextDecoder().decode(data)
      );

      switch (message.type) {
        case 'chat':
          addMessage({
            id: `chat-${Date.now()}-${Math.random()}`,
            type: 'chat',
            content: message.payload as string,
            sender: message.sender,
            senderName: message.senderName || message.sender.slice(0, 8),
            timestamp: message.timestamp,
          });
          break;

        case 'gift_notification':
          const giftPayload = message.payload as any;
          addMessage({
            id: `gift-${Date.now()}-${Math.random()}`,
            type: 'gift',
            content: giftPayload.giftName,
            sender: message.sender,
            senderName: message.senderName || message.sender.slice(0, 8),
            timestamp: message.timestamp,
            giftId: giftPayload.giftId,
            giftCount: giftPayload.quantity,
            giftValue: giftPayload.totalValue,
          });

          // 更新收益
          setEarnings((prev) =>
            (Number(prev) + Number(giftPayload.totalValue)).toString()
          );

          // 添加礼物动画
          const gift = gifts.find((g) => g.id === giftPayload.giftId);
          const isFullScreen = gift && Number(gift.price) >= 100;
          setGiftAnimations((prev) => [
            ...prev,
            {
              id: `anim-${Date.now()}-${Math.random()}`,
              giftId: giftPayload.giftId,
              giftName: giftPayload.giftName,
              senderName: message.senderName || message.sender.slice(0, 8),
              quantity: giftPayload.quantity,
              isFullScreen: isFullScreen || false,
            },
          ]);
          break;

        case 'cohost_request':
          const requestPayload = message.payload as any;
          addCoHostRequest({
            address: message.sender,
            name: message.senderName,
            type: requestPayload.type,
            timestamp: message.timestamp,
          });
          break;
      }
    } catch (error) {
      console.error('解析消息失败:', error);
    }
  };

  // 切换静音
  const handleToggleMute = async () => {
    if (!liveKitService) return;
    const newMuted = await liveKitService.toggleMute();
    setIsMuted(!newMuted);
  };

  // 切换摄像头
  const handleToggleCamera = async () => {
    if (!liveKitService) return;
    const newEnabled = await liveKitService.toggleCamera();
    setIsCameraOn(newEnabled);
  };

  // 翻转摄像头
  const handleSwitchCamera = async () => {
    if (!liveKitService) return;
    await liveKitService.switchCamera();
  };

  // 结束直播
  const handleEndLive = () => {
    Alert.alert('结束直播', '确定要结束直播吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确定',
        style: 'destructive',
        onPress: async () => {
          try {
            const service = getLivestreamService();
            await service.endLive(roomId);

            if (liveKitService) {
              await liveKitService.disconnect();
            }

            if (durationTimerRef.current) {
              clearInterval(durationTimerRef.current);
            }

            setIsLive(false);
            setIsHost(false);
            setIsInRoom(false);
            clearMessages();

            router.replace('/livestream');
          } catch (error) {
            console.error('结束直播失败:', error);
            Alert.alert('错误', '结束直播失败');
          }
        },
      },
    ]);
  };

  // 接受连麦
  const handleAcceptCoHost = async (address: string) => {
    try {
      const service = getLivestreamService();
      await service.startCoHost(roomId, address);
      removeCoHostRequest(address);

      // 发送接受通知
      if (liveKitService) {
        const message: DataChannelMessage = {
          type: 'cohost_accept',
          payload: { address },
          sender: currentWallet?.address || '',
          timestamp: Date.now(),
        };
        await liveKitService.sendData(message);
      }
    } catch (error) {
      console.error('接受连麦失败:', error);
      Alert.alert('错误', '接受连麦失败');
    }
  };

  // 拒绝连麦
  const handleRejectCoHost = async (address: string) => {
    removeCoHostRequest(address);

    // 发送拒绝通知
    if (liveKitService) {
      const message: DataChannelMessage = {
        type: 'cohost_reject',
        payload: { address },
        sender: currentWallet?.address || '',
        timestamp: Date.now(),
      };
      await liveKitService.sendData(message);
    }
  };

  // 发送聊天消息
  const handleSendMessage = async (content: string) => {
    if (!liveKitService || !currentWallet) return;

    const message: DataChannelMessage = {
      type: 'chat',
      payload: content,
      sender: currentWallet.address,
      senderName: currentWallet.name || '主播',
      timestamp: Date.now(),
    };

    await liveKitService.sendData(message);

    addMessage({
      id: `chat-${Date.now()}`,
      type: 'chat',
      content,
      sender: currentWallet.address,
      senderName: currentWallet.name || '主播',
      timestamp: Date.now(),
    });
  };

  // 移除礼物动画
  const handleAnimationComplete = (id: string) => {
    setGiftAnimations((prev) => prev.filter((a) => a.id !== id));
  };

  // 返回处理
  useEffect(() => {
    const backHandler = BackHandler.addEventListener('hardwareBackPress', () => {
      handleEndLive();
      return true;
    });

    return () => backHandler.remove();
  }, [liveKitService]);

  useEffect(() => {
    initLive();

    return () => {
      if (liveKitService) {
        liveKitService.disconnect();
      }
      if (durationTimerRef.current) {
        clearInterval(durationTimerRef.current);
      }
      setIsLive(false);
      setIsHost(false);
      setIsInRoom(false);
      clearMessages();
    };
  }, []);

  return (
    <View style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="transparent" translucent />

      {/* 摄像头预览 */}
      <View style={styles.videoContainer}>
        {localVideoTrack ? (
          <VideoView
            style={styles.video}
            videoTrack={localVideoTrack}
            objectFit="cover"
            mirror={true}
          />
        ) : (
          <View style={styles.videoPlaceholder} />
        )}
      </View>

      {/* 统计信息 */}
      <View style={styles.statsOverlay}>
        <LiveStats
          viewerCount={viewerCount}
          earnings={earnings}
          duration={duration}
        />
      </View>

      {/* 控制面板 */}
      <View style={styles.controlsOverlay}>
        <LiveControls
          isMuted={isMuted}
          isCameraOn={isCameraOn}
          onToggleMute={handleToggleMute}
          onToggleCamera={handleToggleCamera}
          onSwitchCamera={handleSwitchCamera}
          onEndLive={handleEndLive}
        />
      </View>

      {/* 聊天/连麦面板 */}
      <View style={styles.bottomPanel}>
        {showCoHostPanel ? (
          <CoHostPanel
            requests={coHostRequests}
            onAccept={handleAcceptCoHost}
            onReject={handleRejectCoHost}
          />
        ) : (
          <LiveChat
            onSendMessage={handleSendMessage}
            onSendDanmaku={() => {}}
            onOpenGiftPanel={() => setShowCoHostPanel(true)}
          />
        )}
      </View>

      {/* 礼物动画 */}
      {giftAnimations.map((anim) => {
        const gift = gifts.find((g) => g.id === anim.giftId);
        return (
          <GiftAnimation
            key={anim.id}
            item={anim}
            giftIconCid={gift?.iconCid}
            onComplete={() => handleAnimationComplete(anim.id)}
          />
        );
      })}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#000',
  },
  videoContainer: {
    flex: 1,
  },
  video: {
    flex: 1,
  },
  videoPlaceholder: {
    flex: 1,
    backgroundColor: '#1A1A2E',
  },
  statsOverlay: {
    position: 'absolute',
    top: 44,
    left: 0,
    right: 0,
  },
  controlsOverlay: {
    position: 'absolute',
    bottom: 300,
    left: 0,
    right: 0,
  },
  bottomPanel: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    height: 280,
    backgroundColor: 'rgba(0,0,0,0.8)',
  },
});
