import { useTransaction } from '@/src/hooks/useTransaction';
import { useAuthStore } from '@/src/stores/auth';
import { useChainStore } from '@/src/stores/chain';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  Platform,
  Pressable,
  ScrollView,
  StyleSheet,
  Switch,
  Text,
  TextInput,
  View
} from 'react-native';

const GROUP_TYPES = [
  { id: 'bazi', name: '八字命理', icon: '📅' },
  { id: 'qimen', name: '奇门遁甲', icon: '🧭' },
  { id: 'liuyao', name: '六爻占卜', icon: '🎲' },
  { id: 'meihua', name: '梅花易数', icon: '🌸' },
  { id: 'ziwei', name: '紫微斗数', icon: '⭐' },
  { id: 'tarot', name: '塔罗占卜', icon: '🃏' },
  { id: 'general', name: '综合交流', icon: '💬' },
];

// 加密模式映射
const ENCRYPTION_MODES: Record<string, number> = {
  military: 0,
  business: 1,
  selective: 2,
  transparent: 3,
};

export default function CreateGroupScreen() {
  const router = useRouter();
  const { isLoggedIn } = useAuthStore();
  const { isConnected } = useChainStore();
  const { createGroup, isLoading: isTxLoading, status: txStatus } = useTransaction();
  
  const [groupName, setGroupName] = useState('');
  const [description, setDescription] = useState('');
  const [groupType, setGroupType] = useState<string>('general');
  const [isPublic, setIsPublic] = useState(true);
  const [encryptionMode, setEncryptionMode] = useState<string>('business');

  const showAlert = (title: string, message: string) => {
    if (Platform.OS === 'web') {
      window.alert(`${title}\n${message}`);
    } else {
      const { Alert } = require('react-native');
      Alert.alert(title, message);
    }
  };

  const handleCreate = async () => {
    if (!groupName.trim()) {
      showAlert('提示', '请输入群聊名称');
      return;
    }
    if (groupName.length < 2 || groupName.length > 20) {
      showAlert('提示', '群聊名称需要2-20个字符');
      return;
    }
    if (!isLoggedIn) {
      showAlert('提示', '请先登录钱包');
      return;
    }
    if (!isConnected) {
      showAlert('提示', '请先连接区块链网络');
      return;
    }
    
    // 组合群名：类型 + 名称
    const typeInfo = GROUP_TYPES.find(t => t.id === groupType);
    const fullName = typeInfo && groupType !== 'general' 
      ? `[${typeInfo.name}] ${groupName.trim()}`
      : groupName.trim();
    
    const txResult = await createGroup({
      name: fullName,
      description: description.trim() || undefined,
      encryption_mode: ENCRYPTION_MODES[encryptionMode] || 1,
      is_public: isPublic,
    });
    
    if (txResult?.success) {
      router.back();
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>取消</Text>
        </Pressable>
        <Text style={styles.headerTitle}>创建群聊</Text>
        <Pressable 
          style={[styles.createButton, (!groupName.trim() || isTxLoading) && styles.createButtonDisabled]}
          onPress={handleCreate}
          disabled={!groupName.trim() || isTxLoading}
        >
          <Text style={[styles.createButtonText, (!groupName.trim() || isTxLoading) && styles.createButtonTextDisabled]}>
            {isTxLoading ? (txStatus === 'signing' ? '签名中...' : '创建中...') : '创建'}
          </Text>
        </Pressable>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>基本信息</Text>
          
          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>群聊名称 *</Text>
            <TextInput
              style={styles.input}
              placeholder="输入群聊名称（2-20字）"
              placeholderTextColor="#9ca3af"
              value={groupName}
              onChangeText={setGroupName}
              maxLength={20}
            />
            <Text style={styles.charCount}>{groupName.length}/20</Text>
          </View>

          <View style={styles.inputGroup}>
            <Text style={styles.inputLabel}>群聊简介</Text>
            <TextInput
              style={[styles.input, styles.textArea]}
              placeholder="介绍一下你的群聊..."
              placeholderTextColor="#9ca3af"
              value={description}
              onChangeText={setDescription}
              multiline
              numberOfLines={3}
              maxLength={100}
            />
            <Text style={styles.charCount}>{description.length}/100</Text>
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>群聊类型</Text>
          <View style={styles.typeGrid}>
            {GROUP_TYPES.map((type) => (
              <Pressable
                key={type.id}
                style={[styles.typeButton, groupType === type.id && styles.typeButtonSelected]}
                onPress={() => setGroupType(type.id)}
              >
                <Text style={styles.typeIcon}>{type.icon}</Text>
                <Text style={[styles.typeText, groupType === type.id && styles.typeTextSelected]}>
                  {type.name}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>群聊设置</Text>
          
          <View style={styles.settingRow}>
            <View style={styles.settingInfo}>
              <Text style={styles.settingLabel}>公开群聊</Text>
              <Text style={styles.settingHint}>
                {isPublic ? '任何人可搜索加入' : '仅限邀请加入'}
              </Text>
            </View>
            <Switch
              value={isPublic}
              onValueChange={setIsPublic}
              trackColor={{ false: '#e5e7eb', true: '#a78bfa' }}
              thumbColor={isPublic ? '#6D28D9' : '#f4f3f4'}
            />
          </View>

          <View style={styles.settingRow}>
            <View style={styles.settingInfo}>
              <Text style={styles.settingLabel}>加密模式</Text>
              <Text style={styles.settingHint}>
                {encryptionMode === 'military' ? '军用级：量子抗性加密' : 
                 encryptionMode === 'business' ? '商用级：端到端加密' :
                 encryptionMode === 'selective' ? '选择性：用户自选加密' : '透明：公开存储'}
              </Text>
            </View>
          </View>
          <View style={styles.encryptionButtons}>
            {[
              { id: 'business', name: '商用' },
              { id: 'military', name: '军用' },
              { id: 'selective', name: '选择' },
              { id: 'transparent', name: '透明' },
            ].map((mode) => (
              <Pressable
                key={mode.id}
                style={[styles.limitButton, encryptionMode === mode.id && styles.limitButtonSelected]}
                onPress={() => setEncryptionMode(mode.id)}
              >
                <Text style={[styles.limitText, encryptionMode === mode.id && styles.limitTextSelected]}>
                  {mode.name}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>

        <View style={styles.tipsCard}>
          <Text style={styles.tipsTitle}>💡 创建须知</Text>
          <Text style={styles.tipsText}>
            • 群聊名称创建后可修改{'\n'}
            • 群主可设置管理员协助管理{'\n'}
            • 违规群聊可能被封禁{'\n'}
            • 群聊数据存储在链上
          </Text>
        </View>

        <View style={styles.bottomPadding} />
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 16,
    color: '#6b7280',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  createButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    backgroundColor: '#6D28D9',
    borderRadius: 8,
  },
  createButtonDisabled: {
    backgroundColor: '#e5e7eb',
  },
  createButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#fff',
  },
  createButtonTextDisabled: {
    color: '#9ca3af',
  },
  content: {
    flex: 1,
  },
  section: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
  },
  inputGroup: {
    marginBottom: 16,
  },
  inputLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#4b5563',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#f9fafb',
    borderRadius: 10,
    padding: 12,
    fontSize: 15,
    color: '#1f2937',
  },
  textArea: {
    minHeight: 80,
    textAlignVertical: 'top',
  },
  charCount: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'right',
    marginTop: 4,
  },
  typeGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
  },
  typeButton: {
    width: '30%',
    paddingVertical: 12,
    borderRadius: 10,
    backgroundColor: '#f9fafb',
    alignItems: 'center',
  },
  typeButtonSelected: {
    backgroundColor: '#6D28D9',
  },
  typeIcon: {
    fontSize: 24,
    marginBottom: 4,
  },
  typeText: {
    fontSize: 12,
    color: '#6b7280',
  },
  typeTextSelected: {
    color: '#fff',
  },
  settingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  settingInfo: {
    flex: 1,
  },
  settingLabel: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1f2937',
  },
  settingHint: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  memberLimitButtons: {
    flexDirection: 'row',
    gap: 6,
  },
  encryptionButtons: {
    flexDirection: 'row',
    gap: 6,
    marginTop: 8,
  },
  limitButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 6,
    backgroundColor: '#f3f4f6',
  },
  limitButtonSelected: {
    backgroundColor: '#6D28D9',
  },
  limitText: {
    fontSize: 13,
    color: '#6b7280',
  },
  limitTextSelected: {
    color: '#fff',
  },
  tipsCard: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
  },
  tipsTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  tipsText: {
    fontSize: 13,
    color: '#6b7280',
    lineHeight: 22,
  },
  bottomPadding: {
    height: 40,
  },
});
