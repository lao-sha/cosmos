/**
 * 消息气泡组件
 */

import React, { useMemo } from 'react';
import {
  View,
  Text,
  Image,
  StyleSheet,
  TouchableOpacity,
  ActivityIndicator,
} from 'react-native';
import { MessageType, type Message } from '../types';
import { useBlockTime } from '@/hooks/useBlockTime';

interface Props {
  message: Message;
  onRetry?: (tempId: string) => void;
  onDelete?: (tempId: string) => void;
}

export function ChatBubble({ message, onRetry, onDelete }: Props) {
  const isMine = message.isMine;
  const { currentBlock, blockTime } = useBlockTime();

  // 计算消息时间
  const messageTime = useMemo(() => {
    if (message.sentAt === 0) return '';
    const blockDiff = currentBlock - message.sentAt;
    const timeDiff = blockDiff * blockTime;
    const date = new Date(Date.now() - timeDiff);
    return `${date.getHours().toString().padStart(2, '0')}:${date
      .getMinutes()
      .toString()
      .padStart(2, '0')}`;
  }, [message.sentAt, currentBlock, blockTime]);

  const renderContent = () => {
    switch (message.msgType) {
      case MessageType.Text:
        return (
          <Text style={[styles.text, isMine && styles.textMine]}>
            {message.content}
          </Text>
        );

      case MessageType.Image:
        return (
          <Image
            source={{ uri: message.content }}
            style={styles.image}
            resizeMode="cover"
          />
        );

      case MessageType.Voice:
        return (
          <View style={styles.voice}>
            <Text style={[styles.text, isMine && styles.textMine]}>
              🎤 语音消息
            </Text>
          </View>
        );

      case MessageType.File:
        return (
          <View style={styles.file}>
            <Text style={[styles.text, isMine && styles.textMine]}>
              📎 文件
            </Text>
          </View>
        );

      case MessageType.System:
        return <Text style={styles.systemText}>{message.content}</Text>;

      default:
        return (
          <Text style={[styles.text, isMine && styles.textMine]}>
            {message.content}
          </Text>
        );
    }
  };

  // 系统消息居中显示
  if (message.msgType === MessageType.System) {
    return <View style={styles.systemContainer}>{renderContent()}</View>;
  }

  return (
    <View style={[styles.container, isMine ? styles.mine : styles.theirs]}>
      <View
        style={[
          styles.bubble,
          isMine ? styles.bubbleMine : styles.bubbleTheirs,
          message.status === 'failed' && styles.bubbleFailed,
        ]}
      >
        {renderContent()}
      </View>

      <View style={[styles.meta, isMine && styles.metaMine]}>
        {messageTime ? <Text style={styles.time}>{messageTime}</Text> : null}
        {isMine && (
          <MessageStatus
            status={message.status}
            isRead={message.isRead}
            error={message.error}
            onRetry={() => onRetry?.(message.tempId!)}
            onDelete={() => onDelete?.(message.tempId!)}
          />
        )}
      </View>
    </View>
  );
}

interface StatusProps {
  status: Message['status'];
  isRead: boolean;
  error?: string;
  onRetry: () => void;
  onDelete: () => void;
}

function MessageStatus({
  status,
  isRead,
  error,
  onRetry,
  onDelete,
}: StatusProps) {
  switch (status) {
    case 'sending':
      return (
        <View style={styles.statusRow}>
          <ActivityIndicator size="small" color="#999" />
          <Text style={styles.statusText}>发送中</Text>
        </View>
      );

    case 'sent':
      return (
        <Text style={styles.statusText}>
          {isRead ? '已读 ✓✓' : '已发送 ✓'}
        </Text>
      );

    case 'failed':
      return (
        <View style={styles.failedRow}>
          <Text style={styles.errorText}>发送失败</Text>
          <TouchableOpacity onPress={onRetry} style={styles.retryBtn}>
            <Text style={styles.retryText}>重试</Text>
          </TouchableOpacity>
          <TouchableOpacity onPress={onDelete} style={styles.deleteBtn}>
            <Text style={styles.deleteText}>删除</Text>
          </TouchableOpacity>
        </View>
      );

    default:
      return null;
  }
}

const styles = StyleSheet.create({
  container: {
    marginVertical: 4,
    marginHorizontal: 12,
    maxWidth: '80%',
  },
  mine: {
    alignSelf: 'flex-end',
  },
  theirs: {
    alignSelf: 'flex-start',
  },
  bubble: {
    padding: 10,
    borderRadius: 16,
  },
  bubbleMine: {
    backgroundColor: '#007AFF',
    borderBottomRightRadius: 4,
  },
  bubbleTheirs: {
    backgroundColor: '#E9E9EB',
    borderBottomLeftRadius: 4,
  },
  bubbleFailed: {
    opacity: 0.6,
    borderColor: '#FF3B30',
    borderWidth: 1,
  },
  text: {
    fontSize: 16,
    color: '#333',
    lineHeight: 22,
  },
  textMine: {
    color: '#fff',
  },
  image: {
    width: 200,
    height: 150,
    borderRadius: 8,
  },
  voice: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  file: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  meta: {
    flexDirection: 'row',
    marginTop: 4,
    gap: 8,
  },
  metaMine: {
    justifyContent: 'flex-end',
  },
  time: {
    fontSize: 11,
    color: '#999',
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  statusText: {
    fontSize: 11,
    color: '#999',
  },
  failedRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  errorText: {
    fontSize: 11,
    color: '#FF3B30',
  },
  retryBtn: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    backgroundColor: '#007AFF',
    borderRadius: 4,
  },
  retryText: {
    fontSize: 11,
    color: '#fff',
  },
  deleteBtn: {
    paddingHorizontal: 8,
    paddingVertical: 2,
  },
  deleteText: {
    fontSize: 11,
    color: '#999',
  },
  systemContainer: {
    alignItems: 'center',
    marginVertical: 8,
  },
  systemText: {
    fontSize: 12,
    color: '#999',
    backgroundColor: 'rgba(0,0,0,0.05)',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 12,
  },
});
