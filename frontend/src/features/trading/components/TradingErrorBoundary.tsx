/**
 * Trading 错误边界组件
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
} from 'react-native';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class TradingErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // 记录错误日志
    console.error('[TradingErrorBoundary] Error caught:', error);
    console.error('[TradingErrorBoundary] Error info:', errorInfo);

    // 上报错误（如果有回调）
    this.props.onError?.(error, errorInfo);

    // TODO: 上报到错误监控服务
    // reportError({
    //   error: error.message,
    //   stack: error.stack,
    //   componentStack: errorInfo.componentStack,
    //   page: 'trading',
    // });
  }

  handleRetry = (): void => {
    this.setState({
      hasError: false,
      error: null,
    });
  };

  render(): ReactNode {
    if (this.state.hasError) {
      // 如果提供了自定义 fallback，使用它
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // 默认错误 UI
      return (
        <View style={styles.container}>
          <View style={styles.content}>
            <Text style={styles.icon}>⚠️</Text>
            <Text style={styles.title}>页面出错了</Text>
            <Text style={styles.message}>
              交易模块加载失败，请重试
            </Text>

            {/* 错误详情（开发环境显示） */}
            {__DEV__ && this.state.error && (
              <View style={styles.errorDetails}>
                <Text style={styles.errorTitle}>错误详情：</Text>
                <Text style={styles.errorText}>
                  {this.state.error.message}
                </Text>
              </View>
            )}

            <View style={styles.actions}>
              <TouchableOpacity
                style={styles.retryButton}
                onPress={this.handleRetry}
              >
                <Text style={styles.retryButtonText}>重试</Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      );
    }

    return this.props.children;
  }
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#F5F5F7',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  content: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 32,
    alignItems: 'center',
    maxWidth: 400,
    width: '100%',
  },
  icon: {
    fontSize: 64,
    marginBottom: 16,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  message: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
    marginBottom: 24,
  },
  errorDetails: {
    backgroundColor: '#FFF3F3',
    borderRadius: 8,
    padding: 12,
    marginBottom: 24,
    width: '100%',
  },
  errorTitle: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FF3B30',
    marginBottom: 4,
  },
  errorText: {
    fontSize: 12,
    color: '#666666',
    fontFamily: 'monospace',
  },
  actions: {
    width: '100%',
  },
  retryButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  retryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});

/**
 * 网络错误提示组件
 */
interface NetworkErrorProps {
  onRetry: () => void;
  onCancel?: () => void;
}

export const NetworkErrorView: React.FC<NetworkErrorProps> = ({
  onRetry,
  onCancel,
}) => {
  return (
    <View style={networkStyles.container}>
      <View style={networkStyles.content}>
        <Text style={networkStyles.icon}>📡</Text>
        <Text style={networkStyles.title}>网络连接失败</Text>
        <Text style={networkStyles.message}>
          无法连接到区块链网络{'\n'}请检查您的网络连接
        </Text>

        <View style={networkStyles.actions}>
          <TouchableOpacity
            style={networkStyles.retryButton}
            onPress={onRetry}
          >
            <Text style={networkStyles.retryButtonText}>重试连接</Text>
          </TouchableOpacity>

          {onCancel && (
            <TouchableOpacity
              style={networkStyles.cancelButton}
              onPress={onCancel}
            >
              <Text style={networkStyles.cancelButtonText}>稍后再试</Text>
            </TouchableOpacity>
          )}
        </View>
      </View>
    </View>
  );
};

const networkStyles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  content: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 32,
    alignItems: 'center',
    maxWidth: 400,
    width: '100%',
  },
  icon: {
    fontSize: 64,
    marginBottom: 16,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#000000',
    marginBottom: 8,
  },
  message: {
    fontSize: 14,
    color: '#666666',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 24,
  },
  actions: {
    width: '100%',
    gap: 12,
  },
  retryButton: {
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  retryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  cancelButton: {
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
});

/**
 * 交易错误提示组件
 */
interface TransactionErrorProps {
  errorCode: string;
  errorMessage: string;
  onRetry?: () => void;
  onCancel: () => void;
  retryable?: boolean;
}

export const TransactionErrorView: React.FC<TransactionErrorProps> = ({
  errorCode,
  errorMessage,
  onRetry,
  onCancel,
  retryable = false,
}) => {
  return (
    <View style={txStyles.container}>
      <View style={txStyles.content}>
        <Text style={txStyles.icon}>❌</Text>
        <Text style={txStyles.title}>交易失败</Text>

        <View style={txStyles.errorBox}>
          <Text style={txStyles.errorCode}>{errorCode}</Text>
          <Text style={txStyles.errorMessage}>{errorMessage}</Text>
        </View>

        <View style={txStyles.actions}>
          {retryable && onRetry && (
            <TouchableOpacity
              style={txStyles.retryButton}
              onPress={onRetry}
            >
              <Text style={txStyles.retryButtonText}>重试</Text>
            </TouchableOpacity>
          )}

          <TouchableOpacity
            style={txStyles.cancelButton}
            onPress={onCancel}
          >
            <Text style={txStyles.cancelButtonText}>
              {retryable ? '取消' : '确定'}
            </Text>
          </TouchableOpacity>
        </View>
      </View>
    </View>
  );
};

const txStyles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  content: {
    backgroundColor: '#FFFFFF',
    borderRadius: 16,
    padding: 24,
    alignItems: 'center',
    maxWidth: 400,
    width: '100%',
  },
  icon: {
    fontSize: 48,
    marginBottom: 16,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
    color: '#FF3B30',
    marginBottom: 16,
  },
  errorBox: {
    backgroundColor: '#FFF3F3',
    borderRadius: 8,
    padding: 16,
    width: '100%',
    marginBottom: 24,
  },
  errorCode: {
    fontSize: 12,
    fontWeight: '600',
    color: '#FF3B30',
    marginBottom: 4,
  },
  errorMessage: {
    fontSize: 14,
    color: '#666666',
    lineHeight: 20,
  },
  actions: {
    width: '100%',
    flexDirection: 'row',
    gap: 12,
  },
  retryButton: {
    flex: 1,
    backgroundColor: '#B2955D',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  retryButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  cancelButton: {
    flex: 1,
    backgroundColor: '#F5F5F7',
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#666666',
  },
});
