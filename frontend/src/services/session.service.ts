/**
 * 星尘玄鉴 - 会话管理服务
 * 处理自动锁定、后台切换、超时等场景
 */

import { AppState, AppStateStatus } from 'react-native';

type LockCallback = () => void;

/**
 * 会话管理服务
 */
export class SessionService {
  private lastActiveTime: number = Date.now();
  private lockTimer: ReturnType<typeof setTimeout> | null = null;
  private appStateSubscription: ReturnType<typeof AppState.addEventListener> | null = null;
  private lockCallback: LockCallback | null = null;
  private settings = {
    autoLockMinutes: 5,
    lockOnBackground: false,
    maxBackgroundMinutes: 1,
  };

  /**
   * 启动会话管理
   */
  start(
    lockCallback: LockCallback,
    settings: {
      autoLockMinutes: number;
      lockOnBackground: boolean;
      maxBackgroundMinutes: number;
    }
  ): void {
    this.lockCallback = lockCallback;
    this.settings = settings;

    // 监听 App 状态变化
    this.appStateSubscription = AppState.addEventListener(
      'change',
      this.handleAppStateChange
    );

    // 启动活动检测
    this.resetActivityTimer();
  }

  /**
   * 停止会话管理
   */
  stop(): void {
    if (this.appStateSubscription) {
      this.appStateSubscription.remove();
      this.appStateSubscription = null;
    }
    this.clearLockTimer();
    this.lockCallback = null;
  }

  /**
   * 更新设置
   */
  updateSettings(settings: {
    autoLockMinutes: number;
    lockOnBackground: boolean;
    maxBackgroundMinutes: number;
  }): void {
    this.settings = settings;
    this.resetActivityTimer();
  }

  /**
   * 记录用户活动（重置计时器）
   */
  recordActivity(): void {
    this.lastActiveTime = Date.now();
    this.resetActivityTimer();
  }

  /**
   * 处理 App 状态变化
   */
  private handleAppStateChange = (nextState: AppStateStatus): void => {
    if (!this.lockCallback) return;

    switch (nextState) {
      case 'background':
      case 'inactive':
        // 进入后台时的处理
        if (this.settings.lockOnBackground) {
          // 立即锁定
          this.lockCallback();
        } else {
          // 记录进入后台时间
          this.lastActiveTime = Date.now();
        }
        break;

      case 'active':
        // 从后台返回
        const backgroundDuration = Date.now() - this.lastActiveTime;
        const maxBackgroundMs = this.settings.maxBackgroundMinutes * 60 * 1000;

        if (backgroundDuration > maxBackgroundMs) {
          this.lockCallback();
        } else {
          this.resetActivityTimer();
        }
        break;
    }
  };

  /**
   * 重置自动锁定计时器
   */
  private resetActivityTimer(): void {
    this.clearLockTimer();

    if (this.settings.autoLockMinutes <= 0 || !this.lockCallback) {
      return;
    }

    const timeoutMs = this.settings.autoLockMinutes * 60 * 1000;

    this.lockTimer = setTimeout(() => {
      if (this.lockCallback) {
        this.lockCallback();
      }
    }, timeoutMs);
  }

  /**
   * 清除计时器
   */
  private clearLockTimer(): void {
    if (this.lockTimer) {
      clearTimeout(this.lockTimer);
      this.lockTimer = null;
    }
  }

  /**
   * 获取剩余解锁时间（秒）
   */
  getRemainingTime(): number {
    if (this.settings.autoLockMinutes <= 0) return Infinity;

    const elapsed = Date.now() - this.lastActiveTime;
    const remaining = this.settings.autoLockMinutes * 60 * 1000 - elapsed;

    return Math.max(0, Math.floor(remaining / 1000));
  }
}

// 单例
export const sessionService = new SessionService();
