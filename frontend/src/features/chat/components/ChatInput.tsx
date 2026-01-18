/**
 * 聊天输入框组件
 */

import React, { useState } from 'react';
import {
  View,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Platform,
  Image,
  Alert,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { ImagePickerService, type ImagePickerResult } from '@/services/image-picker.service';

interface Props {
  onSend: (content: string, imageUri?: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

export function ChatInput({
  onSend,
  disabled,
  placeholder = '输入消息...',
}: Props) {
  const [text, setText] = useState('');
  const [selectedImage, setSelectedImage] = useState<ImagePickerResult | null>(null);

  const handleSend = () => {
    const trimmed = text.trim();
    if (trimmed || selectedImage) {
      onSend(trimmed, selectedImage?.uri);
      setText('');
      setSelectedImage(null);
    }
  };

  const handleAttach = async () => {
    const result = await ImagePickerService.showPicker();
    if (result) {
      setSelectedImage(result);
    }
  };

  const handleRemoveImage = () => {
    setSelectedImage(null);
  };

  const canSend = (text.trim().length > 0 || selectedImage) && !disabled;

  return (
    <View style={styles.container}>
      {selectedImage && (
        <View style={styles.imagePreviewContainer}>
          <Image source={{ uri: selectedImage.uri }} style={styles.imagePreview} />
          <TouchableOpacity
            style={styles.removeImageBtn}
            onPress={handleRemoveImage}
          >
            <Ionicons name="close-circle" size={24} color="#fff" />
          </TouchableOpacity>
        </View>
      )}

      <View style={styles.inputRow}>
        <TouchableOpacity
          style={styles.iconBtn}
          onPress={handleAttach}
          disabled={disabled}
        >
          <Ionicons
            name="image-outline"
            size={28}
            color={disabled ? '#ccc' : '#666'}
          />
        </TouchableOpacity>

        <View style={styles.inputWrapper}>
          <TextInput
            style={styles.input}
            value={text}
            onChangeText={setText}
            placeholder={placeholder}
            placeholderTextColor="#999"
            multiline
            maxLength={1000}
            editable={!disabled}
            onSubmitEditing={handleSend}
          />
        </View>

        <TouchableOpacity
          style={[styles.sendBtn, !canSend && styles.sendBtnDisabled]}
          onPress={handleSend}
          disabled={!canSend}
        >
          <Ionicons name="send" size={24} color={canSend ? '#007AFF' : '#ccc'} />
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: '#e0e0e0',
  },
  imagePreviewContainer: {
    padding: 8,
    position: 'relative',
  },
  imagePreview: {
    width: 100,
    height: 100,
    borderRadius: 8,
  },
  removeImageBtn: {
    position: 'absolute',
    top: 4,
    right: 4,
    backgroundColor: 'rgba(0,0,0,0.5)',
    borderRadius: 12,
  },
  inputRow: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    paddingHorizontal: 8,
    paddingVertical: 8,
  },
  iconBtn: {
    padding: 8,
  },
  inputWrapper: {
    flex: 1,
    backgroundColor: '#f5f5f5',
    borderRadius: 20,
    paddingHorizontal: 16,
    paddingVertical: Platform.OS === 'ios' ? 10 : 6,
    marginHorizontal: 8,
    maxHeight: 120,
  },
  input: {
    fontSize: 16,
    color: '#333',
    maxHeight: 100,
  },
  sendBtn: {
    padding: 8,
  },
  sendBtnDisabled: {
    opacity: 0.5,
  },
});
