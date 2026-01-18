// frontend/src/divination/market/components/ErrorBoundary.tsx

import React, { Component, ErrorInfo, ReactNode } from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { THEME } from '../theme';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught error:', error, errorInfo);
    this.props.onError?.(error, errorInfo);
  }

  private handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <View style={styles.container}>
          <Ionicons name="alert-circle-outline" size={64} color={THEME.error} />
          <Text style={styles.title}>出错了</Text>
          <Text style={styles.message}>
            {this.state.error?.message || '发生了未知错误'}
          </Text>
          <TouchableOpacity style={styles.button} onPress={this.handleRetry}>
            <Ionicons name="refresh-outline" size={18} color={THEME.textInverse} />
            <Text style={styles.buttonText}>重试</Text>
          </TouchableOpacity>
        </View>
      );
    }

    return this.props.children;
  }
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 32,
    backgroundColor: THEME.background,
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME.text,
    marginTop: 16,
  },
  message: {
    fontSize: 14,
    color: THEME.textSecondary,
    textAlign: 'center',
    marginTop: 8,
    lineHeight: 20,
  },
  button: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: THEME.primary,
    paddingVertical: 10,
    paddingHorizontal: 20,
    borderRadius: 8,
    marginTop: 24,
    gap: 6,
  },
  buttonText: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.textInverse,
  },
});
