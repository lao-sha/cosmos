/**
 * API 初始化 Hook
 * 只支持 React Native 原生平台
 */

import { useEffect, useState } from 'react';
import { initializeApi, isApiInitialized } from '@/lib/api';

export function useApiInitialization() {
  const [isInitializing, setIsInitializing] = useState(false);
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const initialize = async () => {
      // 如果已经初始化，直接返回
      if (isApiInitialized()) {
        setIsReady(true);
        return;
      }

      setIsInitializing(true);
      setError(null);

      try {
        console.log('[useApiInitialization] Initializing API...');

        // 初始化 API（仅 React Native）
        await initializeApi();
        console.log('[useApiInitialization] API initialized');

        setIsReady(true);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'API initialization failed';
        console.error('[useApiInitialization] Error:', errorMessage);
        setError(errorMessage);
      } finally {
        setIsInitializing(false);
      }
    };

    initialize();
  }, []);

  return {
    isInitializing,
    isReady,
    error,
  };
}
