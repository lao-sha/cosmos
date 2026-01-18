// frontend/src/features/livestream/services/livekit.service.ts

import {
  Room,
  RoomEvent,
  Track,
  Participant,
  LocalParticipant,
  AudioSession,
  registerGlobals,
} from '@livekit/react-native';

const LIVEKIT_URL = process.env.EXPO_PUBLIC_LIVEKIT_URL || 'wss://your-livekit-server.com';

// 必须在应用启动时调用一次
let isInitialized = false;

export async function initializeLiveKit(): Promise<void> {
  if (isInitialized) return;

  // 注册 LiveKit 全局依赖 (WebRTC polyfills)
  registerGlobals();

  // 配置音频会话 (iOS 需要)
  await AudioSession.configureAudio({
    android: {
      preferredOutputList: ['speaker'],
      audioTypeOptions: {
        manageAudioFocus: true,
        audioMode: 'normal',
        audioFocusMode: 'gain',
      },
    },
    ios: {
      defaultOutput: 'speaker',
    },
  });

  // 启动音频会话
  await AudioSession.startAudioSession();

  isInitialized = true;
}

export class LiveKitService {
  private room: Room | null = null;
  private onTrackSubscribed?: (track: Track, participant: Participant) => void;
  private onParticipantConnected?: (participant: Participant) => void;
  private onParticipantDisconnected?: (participant: Participant) => void;
  private onDataReceived?: (data: Uint8Array, participant: Participant) => void;
  private onConnectionStateChanged?: (state: string) => void;

  /**
   * 连接到 LiveKit 房间
   */
  async connect(token: string): Promise<void> {
    // 确保 LiveKit 已初始化
    await initializeLiveKit();

    this.room = new Room();

    // 设置事件监听
    this.setupEventListeners();

    await this.room.connect(LIVEKIT_URL, token, {});
  }

  /**
   * 断开连接
   */
  async disconnect(): Promise<void> {
    if (this.room) {
      await this.room.disconnect();
      this.room = null;
    }
  }

  /**
   * 开启摄像头和麦克风 (主播)
   */
  async enableCameraAndMicrophone(): Promise<void> {
    if (!this.room) throw new Error('Not connected');
    await this.room.localParticipant.enableCameraAndMicrophone();
  }

  /**
   * 切换摄像头 (前置/后置)
   */
  async switchCamera(): Promise<void> {
    if (!this.room) return;

    const videoTrack = this.room.localParticipant
      .getTrackPublication(Track.Source.Camera)?.track;

    if (videoTrack) {
      // React Native 中使用 switchCamera 方法
      await (videoTrack as any).restartTrack({
        facingMode: (videoTrack as any).facingMode === 'user' ? 'environment' : 'user',
      });
    }
  }

  /**
   * 切换麦克风静音
   */
  async toggleMute(): Promise<boolean> {
    if (!this.room) return false;

    const enabled = this.room.localParticipant.isMicrophoneEnabled;
    await this.room.localParticipant.setMicrophoneEnabled(!enabled);
    return !enabled;
  }

  /**
   * 切换摄像头开关
   */
  async toggleCamera(): Promise<boolean> {
    if (!this.room) return false;

    const enabled = this.room.localParticipant.isCameraEnabled;
    await this.room.localParticipant.setCameraEnabled(!enabled);
    return !enabled;
  }

  /**
   * 发送数据消息 (聊天/弹幕)
   */
  async sendData(data: object): Promise<void> {
    if (!this.room) throw new Error('Not connected');

    const encoded = new TextEncoder().encode(JSON.stringify(data));
    await this.room.localParticipant.publishData(encoded, { reliable: true });
  }

  /**
   * 获取当前观众数
   */
  getParticipantCount(): number {
    if (!this.room) return 0;
    return this.room.remoteParticipants.size + 1;
  }

  /**
   * 获取本地参与者
   */
  getLocalParticipant(): LocalParticipant | null {
    return this.room?.localParticipant || null;
  }

  /**
   * 获取 Room 实例 (用于 React Hooks)
   */
  getRoom(): Room | null {
    return this.room;
  }

  /**
   * 检查是否已连接
   */
  isConnected(): boolean {
    return this.room?.state === 'connected';
  }

  /**
   * 设置轨道订阅回调
   */
  setOnTrackSubscribed(callback: (track: Track, participant: Participant) => void): void {
    this.onTrackSubscribed = callback;
  }

  /**
   * 设置参与者连接回调
   */
  setOnParticipantConnected(callback: (participant: Participant) => void): void {
    this.onParticipantConnected = callback;
  }

  /**
   * 设置参与者断开回调
   */
  setOnParticipantDisconnected(callback: (participant: Participant) => void): void {
    this.onParticipantDisconnected = callback;
  }

  /**
   * 设置数据接收回调
   */
  setOnDataReceived(callback: (data: Uint8Array, participant: Participant) => void): void {
    this.onDataReceived = callback;
  }

  /**
   * 设置连接状态变化回调
   */
  setOnConnectionStateChanged(callback: (state: string) => void): void {
    this.onConnectionStateChanged = callback;
  }

  private setupEventListeners(): void {
    if (!this.room) return;

    this.room.on(RoomEvent.TrackSubscribed, (track, publication, participant) => {
      this.onTrackSubscribed?.(track, participant);
    });

    this.room.on(RoomEvent.ParticipantConnected, (participant) => {
      this.onParticipantConnected?.(participant);
    });

    this.room.on(RoomEvent.ParticipantDisconnected, (participant) => {
      this.onParticipantDisconnected?.(participant);
    });

    this.room.on(RoomEvent.DataReceived, (payload, participant) => {
      this.onDataReceived?.(payload, participant!);
    });

    this.room.on(RoomEvent.Connected, () => {
      console.log('[LiveKit] Connected');
      this.onConnectionStateChanged?.('connected');
    });

    this.room.on(RoomEvent.Disconnected, () => {
      console.log('[LiveKit] Disconnected');
      this.onConnectionStateChanged?.('disconnected');
    });

    this.room.on(RoomEvent.Reconnecting, () => {
      console.log('[LiveKit] Reconnecting...');
      this.onConnectionStateChanged?.('reconnecting');
    });

    this.room.on(RoomEvent.Reconnected, () => {
      console.log('[LiveKit] Reconnected');
      this.onConnectionStateChanged?.('connected');
    });
  }
}
