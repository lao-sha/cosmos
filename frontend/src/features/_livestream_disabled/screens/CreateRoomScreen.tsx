// frontend/src/features/livestream/screens/CreateRoomScreen.tsx

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TextInput,
  TouchableOpacity,
  ScrollView,
  Image,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import * as ImagePicker from 'expo-image-picker';
import { useWalletStore } from '@/stores/wallet.store';
import { initLivestreamService } from '../services/livestream.service';
import { LiveRoomType } from '../types';

const ROOM_TYPES: { key: LiveRoomType; label: string; description: string }[] = [
  { key: LiveRoomType.Normal, label: '普通直播', description: '免费观看' },
  { key: LiveRoomType.Paid, label: '付费直播', description: '需购买门票' },
  { key: LiveRoomType.Private, label: '私密直播', description: '仅邀请可见' },
  { key: LiveRoomType.MultiHost, label: '连麦直播', description: '支持多人连麦' },
];

export function CreateRoomScreen() {
  const router = useRouter();
  const { currentWallet } = useWalletStore();

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [roomType, setRoomType] = useState<LiveRoomType>(LiveRoomType.Normal);
  const [ticketPrice, setTicketPrice] = useState('');
  const [coverUri, setCoverUri] = useState<string | null>(null);
  const [coverCid, setCoverCid] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [isUploading, setIsUploading] = useState(false);

  const handlePickCover = async () => {
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [16, 9],
      quality: 0.8,
    });

    if (!result.canceled && result.assets[0]) {
      setCoverUri(result.assets[0].uri);
      // TODO: 上传到 IPFS
      // setIsUploading(true);
      // const cid = await uploadToIpfs(result.assets[0].uri);
      // setCoverCid(cid);
      // setIsUploading(false);
    }
  };

  const handleCreate = async () => {
    if (!title.trim()) {
      Alert.alert('提示', '请输入直播标题');
      return;
    }

    if (!currentWallet?.address) {
      Alert.alert('提示', '请先连接钱包');
      return;
    }

    if (roomType === LiveRoomType.Paid && !ticketPrice) {
      Alert.alert('提示', '请设置门票价格');
      return;
    }

    setIsCreating(true);
    try {
      const service = initLivestreamService(currentWallet.address);
      await service.init();

      const roomId = await service.createRoom({
        title: title.trim(),
        description: description.trim() || undefined,
        roomType,
        coverCid: coverCid || undefined,
        ticketPrice: roomType === LiveRoomType.Paid ? ticketPrice : undefined,
      });

      Alert.alert('成功', '直播间创建成功', [
        {
          text: '开始直播',
          onPress: () => router.replace(`/livestream/host?roomId=${roomId}`),
        },
      ]);
    } catch (error) {
      console.error('创建直播间失败:', error);
      Alert.alert('错误', '创建直播间失败，请重试');
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <SafeAreaView style={styles.container} edges={['top']}>
      <View style={styles.header}>
        <TouchableOpacity style={styles.backBtn} onPress={() => router.back()}>
          <Ionicons name="chevron-back" size={24} color="#FFF" />
        </TouchableOpacity>
        <Text style={styles.headerTitle}>创建直播间</Text>
        <View style={styles.placeholder} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        {/* 封面图 */}
        <TouchableOpacity
          style={styles.coverContainer}
          onPress={handlePickCover}
          disabled={isUploading}
        >
          {coverUri ? (
            <Image source={{ uri: coverUri }} style={styles.coverImage} />
          ) : (
            <View style={styles.coverPlaceholder}>
              {isUploading ? (
                <ActivityIndicator color="#FF4757" />
              ) : (
                <>
                  <Ionicons name="camera" size={32} color="#666" />
                  <Text style={styles.coverText}>点击上传封面图</Text>
                  <Text style={styles.coverHint}>推荐尺寸 16:9</Text>
                </>
              )}
            </View>
          )}
        </TouchableOpacity>

        {/* 直播标题 */}
        <View style={styles.inputGroup}>
          <Text style={styles.label}>直播标题</Text>
          <TextInput
            style={styles.input}
            placeholder="输入直播标题..."
            placeholderTextColor="#666"
            value={title}
            onChangeText={setTitle}
            maxLength={50}
          />
        </View>

        {/* 直播简介 */}
        <View style={styles.inputGroup}>
          <Text style={styles.label}>直播简介</Text>
          <TextInput
            style={[styles.input, styles.textArea]}
            placeholder="输入直播简介..."
            placeholderTextColor="#666"
            value={description}
            onChangeText={setDescription}
            multiline
            numberOfLines={3}
            maxLength={200}
          />
        </View>

        {/* 直播类型 */}
        <View style={styles.inputGroup}>
          <Text style={styles.label}>直播类型</Text>
          <View style={styles.typeGrid}>
            {ROOM_TYPES.map((type) => (
              <TouchableOpacity
                key={type.key}
                style={[
                  styles.typeItem,
                  roomType === type.key && styles.typeItemActive,
                ]}
                onPress={() => setRoomType(type.key)}
              >
                <Text
                  style={[
                    styles.typeLabel,
                    roomType === type.key && styles.typeLabelActive,
                  ]}
                >
                  {type.label}
                </Text>
                <Text style={styles.typeDesc}>{type.description}</Text>
              </TouchableOpacity>
            ))}
          </View>
        </View>

        {/* 门票价格 (付费直播) */}
        {roomType === LiveRoomType.Paid && (
          <View style={styles.inputGroup}>
            <Text style={styles.label}>门票价格</Text>
            <View style={styles.priceInputContainer}>
              <TextInput
                style={styles.priceInput}
                placeholder="0"
                placeholderTextColor="#666"
                value={ticketPrice}
                onChangeText={setTicketPrice}
                keyboardType="numeric"
              />
              <Text style={styles.priceUnit}>DUST</Text>
            </View>
          </View>
        )}

        {/* 押金提示 */}
        <View style={styles.depositNotice}>
          <Ionicons name="information-circle" size={16} color="#FFD700" />
          <Text style={styles.depositText}>创建直播间需要押金 100 DUST</Text>
        </View>

        {/* 创建按钮 */}
        <TouchableOpacity
          style={[styles.createBtn, isCreating && styles.createBtnDisabled]}
          onPress={handleCreate}
          disabled={isCreating}
        >
          {isCreating ? (
            <ActivityIndicator color="#FFF" />
          ) : (
            <Text style={styles.createBtnText}>创建直播间</Text>
          )}
        </TouchableOpacity>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0F0F1A',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  backBtn: {
    padding: 4,
  },
  headerTitle: {
    color: '#FFF',
    fontSize: 18,
    fontWeight: '600',
  },
  placeholder: {
    width: 32,
  },
  content: {
    flex: 1,
    padding: 16,
  },
  coverContainer: {
    width: '100%',
    aspectRatio: 16 / 9,
    borderRadius: 12,
    overflow: 'hidden',
    marginBottom: 20,
  },
  coverImage: {
    width: '100%',
    height: '100%',
  },
  coverPlaceholder: {
    flex: 1,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
  },
  coverText: {
    color: '#666',
    fontSize: 14,
    marginTop: 8,
  },
  coverHint: {
    color: '#444',
    fontSize: 12,
    marginTop: 4,
  },
  inputGroup: {
    marginBottom: 20,
  },
  label: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '500',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#252540',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    color: '#FFF',
    fontSize: 15,
  },
  textArea: {
    height: 100,
    textAlignVertical: 'top',
  },
  typeGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 10,
  },
  typeItem: {
    width: '48%',
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 14,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  typeItemActive: {
    borderColor: '#FF4757',
    backgroundColor: 'rgba(255, 71, 87, 0.1)',
  },
  typeLabel: {
    color: '#FFF',
    fontSize: 14,
    fontWeight: '600',
    marginBottom: 4,
  },
  typeLabelActive: {
    color: '#FF4757',
  },
  typeDesc: {
    color: '#666',
    fontSize: 12,
  },
  priceInputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#252540',
    borderRadius: 12,
    paddingHorizontal: 16,
  },
  priceInput: {
    flex: 1,
    paddingVertical: 14,
    color: '#FFF',
    fontSize: 15,
  },
  priceUnit: {
    color: '#FFD700',
    fontSize: 14,
    fontWeight: '600',
  },
  depositNotice: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(255, 215, 0, 0.1)',
    borderRadius: 8,
    padding: 12,
    marginBottom: 24,
    gap: 8,
  },
  depositText: {
    color: '#FFD700',
    fontSize: 13,
  },
  createBtn: {
    backgroundColor: '#FF4757',
    borderRadius: 24,
    paddingVertical: 16,
    alignItems: 'center',
    marginBottom: 40,
  },
  createBtnDisabled: {
    backgroundColor: '#555',
  },
  createBtnText: {
    color: '#FFF',
    fontSize: 16,
    fontWeight: 'bold',
  },
});
