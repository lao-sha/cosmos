/**
 * 星尘玄鉴 - 做市商状态管理
 * 管理做市商申请、押金、提现、扣除等状态
 */

import { create } from 'zustand';
import {
  makerService,
  MakerApplication,
  WithdrawalRequest,
  PenaltyRecord,
  MakerInfoInput,
  ApplicationStatus,
} from '@/services/maker.service';
import { useWalletStore } from './wallet.store';

// ===== Store 定义 =====

interface MakerState {
  // 做市商信息
  makerId: number | null;
  makerApp: MakerApplication | null;
  isLoading: boolean;
  error: string | null;

  // 提现状态
  withdrawalRequest: WithdrawalRequest | null;
  loadingWithdrawal: boolean;

  // 扣除记录
  penalties: PenaltyRecord[];
  loadingPenalties: boolean;

  // 押金状态
  depositUsdValue: number;
  dustPrice: number;
  needsReplenishment: boolean;

  // 交易状态
  isSubmitting: boolean;
  txStatus: string | null;

  // Actions - 查询
  fetchMakerInfo: () => Promise<void>;
  fetchWithdrawalRequest: () => Promise<void>;
  fetchPenalties: () => Promise<void>;
  fetchDustPrice: () => Promise<void>;
  refreshAll: () => Promise<void>;

  // Actions - 申请流程
  lockDeposit: (onStatusChange?: (status: string) => void) => Promise<void>;
  submitInfo: (info: MakerInfoInput, onStatusChange?: (status: string) => void) => Promise<void>;
  cancelApplication: (onStatusChange?: (status: string) => void) => Promise<void>;

  // Actions - 押金管理
  replenishDeposit: (onStatusChange?: (status: string) => void) => Promise<void>;
  requestWithdrawal: (amount: bigint, onStatusChange?: (status: string) => void) => Promise<void>;
  executeWithdrawal: (onStatusChange?: (status: string) => void) => Promise<void>;
  cancelWithdrawal: (onStatusChange?: (status: string) => void) => Promise<void>;

  // Actions - 申诉
  appealPenalty: (penaltyId: number, evidenceCid: string, onStatusChange?: (status: string) => void) => Promise<void>;

  // Actions - 订阅
  subscribeToMaker: () => Promise<() => void>;

  // Actions - 工具
  clearError: () => void;
  reset: () => void;
  setTxStatus: (status: string | null) => void;
}

// 押金阈值常量 (USD)
const DEPOSIT_THRESHOLD_USD = 950;
const DEPOSIT_TARGET_USD = 1050;

/**
 * Maker Store
 */
export const useMakerStore = create<MakerState>()((set, get) => ({
  // 初始状态
  makerId: null,
  makerApp: null,
  isLoading: false,
  error: null,

  withdrawalRequest: null,
  loadingWithdrawal: false,

  penalties: [],
  loadingPenalties: false,

  depositUsdValue: 0,
  dustPrice: 0.1,
  needsReplenishment: false,

  isSubmitting: false,
  txStatus: null,

  // ===== 查询方法 =====

  /**
   * 获取做市商信息
   */
  fetchMakerInfo: async () => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ makerId: null, makerApp: null });
      return;
    }

    try {
      set({ isLoading: true, error: null });

      const makerApp = await makerService.getMakerByAccount(address);

      if (makerApp) {
        // 计算押金 USD 价值
        const depositUsdValue = await makerService.calculateDepositUsdValue(makerApp.deposit);
        const needsReplenishment = depositUsdValue < DEPOSIT_THRESHOLD_USD;

        set({
          makerId: makerApp.id,
          makerApp,
          depositUsdValue,
          needsReplenishment,
          isLoading: false,
        });
      } else {
        set({
          makerId: null,
          makerApp: null,
          depositUsdValue: 0,
          needsReplenishment: false,
          isLoading: false,
        });
      }
    } catch (error) {
      console.error('[MakerStore] Fetch maker info error:', error);
      set({
        error: '获取做市商信息失败',
        isLoading: false,
      });
    }
  },

  /**
   * 获取提现请求
   */
  fetchWithdrawalRequest: async () => {
    const { makerId } = get();
    if (makerId === null) {
      set({ withdrawalRequest: null });
      return;
    }

    try {
      set({ loadingWithdrawal: true });

      const request = await makerService.getWithdrawalRequest(makerId);
      set({ withdrawalRequest: request, loadingWithdrawal: false });
    } catch (error) {
      console.error('[MakerStore] Fetch withdrawal request error:', error);
      set({ loadingWithdrawal: false });
    }
  },

  /**
   * 获取扣除记录
   */
  fetchPenalties: async () => {
    const { makerId } = get();
    if (makerId === null) {
      set({ penalties: [] });
      return;
    }

    try {
      set({ loadingPenalties: true });

      const penalties = await makerService.getMakerPenalties(makerId);
      set({ penalties, loadingPenalties: false });
    } catch (error) {
      console.error('[MakerStore] Fetch penalties error:', error);
      set({ loadingPenalties: false });
    }
  },

  /**
   * 获取 DUST 价格
   */
  fetchDustPrice: async () => {
    try {
      const dustPrice = await makerService.getDustPrice();
      set({ dustPrice });

      // 重新计算押金 USD 价值
      const { makerApp } = get();
      if (makerApp) {
        const depositUsdValue = await makerService.calculateDepositUsdValue(makerApp.deposit);
        const needsReplenishment = depositUsdValue < DEPOSIT_THRESHOLD_USD;
        set({ depositUsdValue, needsReplenishment });
      }
    } catch (error) {
      console.error('[MakerStore] Fetch dust price error:', error);
    }
  },

  /**
   * 刷新所有数据
   */
  refreshAll: async () => {
    const { fetchMakerInfo, fetchWithdrawalRequest, fetchPenalties, fetchDustPrice } = get();
    await Promise.all([
      fetchMakerInfo(),
      fetchDustPrice(),
    ]);
    // 这些依赖 makerId，需要在 fetchMakerInfo 之后
    await Promise.all([
      fetchWithdrawalRequest(),
      fetchPenalties(),
    ]);
  },

  // ===== 申请流程 =====

  /**
   * 锁定押金
   */
  lockDeposit: async (onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.lockDeposit(address, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '押金锁定成功', isSubmitting: false });

      // 刷新做市商信息
      await get().fetchMakerInfo();
    } catch (error) {
      console.error('[MakerStore] Lock deposit error:', error);
      const errorMessage = error instanceof Error ? error.message : '锁定押金失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  /**
   * 提交资料
   */
  submitInfo: async (info, onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.submitInfo(address, info, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '资料提交成功', isSubmitting: false });

      // 刷新做市商信息
      await get().fetchMakerInfo();
    } catch (error) {
      console.error('[MakerStore] Submit info error:', error);
      const errorMessage = error instanceof Error ? error.message : '提交资料失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  /**
   * 取消申请
   */
  cancelApplication: async (onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.cancelMaker(address, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '申请已取消', isSubmitting: false });

      // 刷新做市商信息
      await get().fetchMakerInfo();
    } catch (error) {
      console.error('[MakerStore] Cancel application error:', error);
      const errorMessage = error instanceof Error ? error.message : '取消申请失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  // ===== 押金管理 =====

  /**
   * 补充押金
   */
  replenishDeposit: async (onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.replenishDeposit(address, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '押金补充成功', isSubmitting: false });

      // 刷新做市商信息
      await get().fetchMakerInfo();
    } catch (error) {
      console.error('[MakerStore] Replenish deposit error:', error);
      const errorMessage = error instanceof Error ? error.message : '补充押金失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  /**
   * 申请提现
   */
  requestWithdrawal: async (amount, onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.requestWithdrawal(address, amount, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '提现申请已提交', isSubmitting: false });

      // 刷新数据
      await Promise.all([
        get().fetchMakerInfo(),
        get().fetchWithdrawalRequest(),
      ]);
    } catch (error) {
      console.error('[MakerStore] Request withdrawal error:', error);
      const errorMessage = error instanceof Error ? error.message : '申请提现失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  /**
   * 执行提现
   */
  executeWithdrawal: async (onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.executeWithdrawal(address, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '提现执行成功', isSubmitting: false });

      // 刷新数据
      await Promise.all([
        get().fetchMakerInfo(),
        get().fetchWithdrawalRequest(),
      ]);
    } catch (error) {
      console.error('[MakerStore] Execute withdrawal error:', error);
      const errorMessage = error instanceof Error ? error.message : '执行提现失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  /**
   * 取消提现
   */
  cancelWithdrawal: async (onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.cancelWithdrawal(address, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '提现已取消', isSubmitting: false });

      // 刷新数据
      await Promise.all([
        get().fetchMakerInfo(),
        get().fetchWithdrawalRequest(),
      ]);
    } catch (error) {
      console.error('[MakerStore] Cancel withdrawal error:', error);
      const errorMessage = error instanceof Error ? error.message : '取消提现失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  // ===== 申诉 =====

  /**
   * 申诉扣除
   */
  appealPenalty: async (penaltyId, evidenceCid, onStatusChange) => {
    const address = useWalletStore.getState().address;
    if (!address) {
      set({ error: '请先连接钱包' });
      return;
    }

    try {
      set({ isSubmitting: true, error: null, txStatus: '准备交易...' });

      await makerService.appealPenalty(address, penaltyId, evidenceCid, (status) => {
        set({ txStatus: status });
        onStatusChange?.(status);
      });

      set({ txStatus: '申诉已提交', isSubmitting: false });

      // 刷新扣除记录
      await get().fetchPenalties();
    } catch (error) {
      console.error('[MakerStore] Appeal penalty error:', error);
      const errorMessage = error instanceof Error ? error.message : '申诉失败';
      set({ error: errorMessage, isSubmitting: false, txStatus: null });
      throw error;
    }
  },

  // ===== 订阅 =====

  /**
   * 订阅做市商状态变化
   */
  subscribeToMaker: async () => {
    const { makerId } = get();
    if (makerId === null) {
      return () => {};
    }

    const unsub = await makerService.subscribeToMaker(makerId, async (makerApp) => {
      if (makerApp) {
        const depositUsdValue = await makerService.calculateDepositUsdValue(makerApp.deposit);
        const needsReplenishment = depositUsdValue < DEPOSIT_THRESHOLD_USD;

        set({
          makerApp,
          depositUsdValue,
          needsReplenishment,
        });
      } else {
        set({
          makerId: null,
          makerApp: null,
          depositUsdValue: 0,
          needsReplenishment: false,
        });
      }
    });

    return unsub;
  },

  // ===== 工具方法 =====

  /**
   * 清除错误
   */
  clearError: () => {
    set({ error: null });
  },

  /**
   * 重置状态
   */
  reset: () => {
    set({
      makerId: null,
      makerApp: null,
      isLoading: false,
      error: null,
      withdrawalRequest: null,
      loadingWithdrawal: false,
      penalties: [],
      loadingPenalties: false,
      depositUsdValue: 0,
      dustPrice: 0.1,
      needsReplenishment: false,
      isSubmitting: false,
      txStatus: null,
    });
  },

  /**
   * 设置交易状态
   */
  setTxStatus: (status) => {
    set({ txStatus: status });
  },
}));

// ===== 选择器 =====

/**
 * 是否是做市商
 */
export const selectIsMaker = (state: MakerState) =>
  state.makerApp?.status === ApplicationStatus.Active;

/**
 * 是否在申请中
 */
export const selectIsApplying = (state: MakerState) =>
  state.makerApp?.status === ApplicationStatus.DepositLocked ||
  state.makerApp?.status === ApplicationStatus.PendingReview;

/**
 * 是否有待处理的提现
 */
export const selectHasPendingWithdrawal = (state: MakerState) =>
  state.withdrawalRequest !== null;

/**
 * 提现是否可执行
 */
export const selectCanExecuteWithdrawal = (state: MakerState) => {
  if (!state.withdrawalRequest) return false;
  const now = Math.floor(Date.now() / 1000);
  return now >= state.withdrawalRequest.executableAt;
};

/**
 * 未申诉的扣除记录数量
 */
export const selectUnappealedPenaltiesCount = (state: MakerState) =>
  state.penalties.filter((p) => !p.appealed).length;
