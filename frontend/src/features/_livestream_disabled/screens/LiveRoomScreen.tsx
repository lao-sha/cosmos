// frontend/src/features/livestream/screens/LiveRoomScreen.tsx

import React, { useEffect, useState, useCallback } from 'react';
import {
  View,
  StyleSheet,
  StatusBar,
  Alert,
  BackHandler,
} from 'react-native';
import { useRouter } from 'expo-router';
import { Track, Participant } from '@livekit/react-native';
import { useLivestreamStore } from '@/stores/livestream.store';
import { useWalletStore } from '@/stores/wallet.store';
import {
  LivePlayer,
  RoomHeader,
  LiveChat,
  GiftPanel,
  TicketModal,
  GiftAnimation,
} from '../components';
import { LiveKitService } from '../services/livekit.service';
import {
  getLivestreamService,
  initLivestreamService,
} from '../services/livestream.service';
import type { LiveRoom, GiftAnimationItem, DataChannelMessage } from '../types';
import { LiveRoomType } from '../types';

interface LiveRoomScreenProps {
  roomId: number;
}

export function LiveRoomScreen({ roomId }: LiveRoomScreenProps) {
  const router = useRouter();
  const { currentWallet, balance } = useWalletStore();
  const {
    currentRoom,
    setCurrentRoom,
    setIsInRoom,
    gifts,
    setGifts,
    addMessage,
    clearMessages,
    setIsConnecting,
    setIsConnected,
    isConnecting,
    isConnected,
  } = useLivestreamStore();

  const [videoTrack, setVideoTrack] = useState<Track | null>(null);
  const [liveKitService, setLiveKitService] = useState<LiveKitService | null>(null);
  const [showGiftPanel, setShowGiftPanel] = useState(false);
  const [showTicketModal, setShowTicketModal] = useState(false);
  const [giftAnimations, setGiftAnimations] = useState<GiftAnimationItem[]>([]);

  // 加载直播间信息
  const loadRoomInfo = useCallback(async () => {
    if (!currentWallet?.address) return;

    try {
      const service = initLivestreamService(currentWallet.address);
      await service.init();

      const room = await service.getRoomInfo(roomId);
      setCurrentRoom(room);

      // 检查是否需要购票
      if (room.roomType === LiveRoomType.Paid) {
        const hasTicket = await service.checkTicket(roomId);
        if (!hasTicket) {
          setShowTicketModal(true);
          return;
        }
      }

      // 加载礼物列表
      const giftList = await service.getGifts();
      setGifts(giftList);

      // 连接 LiveKit
      await connectLiveKit(service);
    } catch (error) {
      console.error('加载直播间失败:', error);
      Alert.alert('错误', '加载直播间失败');
      router.back();
    }
  }, [currentWallet?.address, roomId]);

  // 连接 LiveKit
  const connectLiveKit = async (service: ReturnType<typeof getLivestreamService>) => {
    setIsConnecting(true);
    try {
      const token = await service.getViewerToken(roomId);
      const livekit = new LiveKitService();

      // 设置回调
      livekit.setOnTrackSubscribed((track, participant) => {
        if (track.kind === 'video') {
          setVideoTrack(track);
        }
      });

      livekit.setOnDataReceived((data, participant) => {
        handleDataMessage(data, participant);
      });

      livekit.setOnConnectionStateChanged((state) => {
        setIsConnected(state === 'connected');
      });

      await livekit.connect(token);
      setLiveKitService(livekit);
      setIsInRoom(true);
      setIsConnected(true);

      // 发送进入消息
      addMessage({
        id: `system-${Date.now()}`,
        type: 'system',
        content: '进入直播间',
        sender: currentWallet?.address || '',
        senderName: '系统',
        timestamp: Date.now(),
      });
    } catch (error) {
      console.error('连接 LiveKit 失败:', error);
      Alert.alert('错误', '连接直播失败，请重试');
    } finally {
      setIsConnecting(false);
    }
  };

  // 处理 DataChannel 消息
  const handleDataMessage = (data: Uint8Array, participant: Participant) => {
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

        case 'danmaku':
          addMessage({
            id: `danmaku-${Date.now()}-${Math.random()}`,
            type: 'danmaku',
            content: (message.payload as any).content,
            sender: message.sender,
            senderName: message.senderName || message.sender.slice(0, 8),
            timestamp: message.timestamp,
            color: (message.payload as any).color,
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
      }
    } catch (error) {
      console.error('解析消息失败:', error);
    }
  };

  // 发送聊天消息
  const handleSendMessage = async (content: string) => {
    if (!liveKitService || !currentWallet) return;

    const message: DataChannelMessage = {
      type: 'chat',
      payload: content,
      sender: currentWallet.address,
      senderName: currentWallet.name || currentWallet.address.slice(0, 8),
      timestamp: Date.now(),
    };

    await liveKitService.sendData(message);

    // 本地添加消息
    addMessage({
      id: `chat-${Date.now()}`,
      type: 'chat',
      content,
      sender: currentWallet.address,
      senderName: currentWallet.name || currentWallet.address.slice(0, 8),
      timestamp: Date.now(),
    });
  };

  // 发送弹幕
  const handleSendDanmaku = async (content: string) => {
    if (!liveKitService || !currentWallet) return;

    const message: DataChannelMessage = {
      type: 'danmaku',
      payload: { content, color: '#FFFFFF' },
      sender: currentWallet.address,
      senderName: currentWallet.name || currentWallet.address.slice(0, 8),
      timestamp: Date.now(),
    };

    await liveKitService.sendData(message);

    // 本地添加弹幕
    addMessage({
      id: `danmaku-${Date.now()}`,
      type: 'danmaku',
      content,
      sender: currentWallet.address,
      senderName: currentWallet.name || currentWallet.address.slice(0, 8),
      timestamp: Date.now(),
      color: '#FFFFFF',
    });
  };

  // 发送礼物
  const handleSendGift = async (giftId: number, quantity: number) => {
    if (!currentWallet) return;

    const service = getLivestreamService();
    await service.sendGift(roomId, giftId, quantity);

    // 发送礼物通知
    const gift = gifts.find((g) => g.id === giftId);
    if (gift && liveKitService) {
      const message: DataChannelMessage = {
        type: 'gift_notification',
        payload: {
          giftId,
          giftName: gift.name,
          quantity,
          totalValue: (Number(gift.price) * quantity).toString(),
        },
        sender: currentWallet.address,
        senderName: currentWallet.name || currentWallet.address.slice(0, 8),
        timestamp: Date.now(),
      };

      await liveKitService.sendData(message);
    }
  };

  // 购买门票
  const handleBuyTicket = async () => {
    if (!currentWallet) return;

    const service = getLivestreamService();
    await service.buyTicket(roomId);
    setShowTicketModal(false);

    // 购票成功后连接
    await connectLiveKit(service);
  };

  // 移除礼物动画
  const handleAnimationComplete = (id: string) => {
    setGiftAnimations((prev) => prev.filter((a) => a.id !== id));
  };

  // 返回处理
  useEffect(() => {
    const backHandler = BackHandler.addEventListener('hardwareBackPress', () => {
      handleLeaveRoom();
      return true;
    });

    return () => backHandler.remove();
  }, [liveKitService]);

  const handleLeaveRoom = async () => {
    if (liveKitService) {
      await liveKitService.disconnect();
    }
    setIsInRoom(false);
    setCurrentRoom(null);
    clearMessages();
    router.back();
  };

  useEffect(() => {
    loadRoomInfo();

    return () => {
      if (liveKitService) {
        liveKitService.disconnect();
      }
      setIsInRoom(false);
      clearMessages();
    };
  }, []);

  const viewerCount = liveKitService?.getParticipantCount() || 0;

  return (
    <View style={styles.container}>
      <StatusBar barStyle="light-content" backgroundColor="transparent" translucent />

      {/* 视频播放器 */}
      <LivePlayer
        videoTrack={videoTrack}
        isLoading={isConnecting}
        showDanmaku={true}
        roomId={roomId}
      />

      {/* 头部信息 */}
      {currentRoom && (
        <View style={styles.headerOverlay}>
          <RoomHeader room={currentRoom} viewerCount={viewerCount} />
        </View>
      )}

      {/* 聊天区域 */}
      <View style={styles.chatContainer}>
        <LiveChat
          onSendMessage={handleSendMessage}
          onSendDanmaku={handleSendDanmaku}
          onOpenGiftPanel={() => setShowGiftPanel(true)}
        />
      </View>

      {/* 礼物面板 */}
      <GiftPanel
        visible={showGiftPanel}
        onClose={() => setShowGiftPanel(false)}
        roomId={roomId}
        balance={balance}
        onSendGift={handleSendGift}
      />

      {/* 购票弹窗 */}
      {currentRoom && (
        <TicketModal
          visible={showTicketModal}
          onClose={() => {
            setShowTicketModal(false);
            router.back();
          }}
          ticketPrice={currentRoom.ticketPrice || '0'}
          balance={balance}
          onBuyTicket={handleBuyTicket}
        />
      )}

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
    backgroundColor: '#0F0F1A',
  },
  headerOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    paddingTop: 44,
  },
  chatContainer: {
    flex: 1,
  },
});
