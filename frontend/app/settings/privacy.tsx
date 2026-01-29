import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Switch,
    Text,
    View,
} from 'react-native';

type PermissionLevel = 'open' | 'friends_only' | 'whitelist' | 'closed';

interface PrivacySettings {
  permissionLevel: PermissionLevel;
  rejectedSceneTypes: string[];
}

const PERMISSION_LEVELS: { value: PermissionLevel; label: string; desc: string }[] = [
  { value: 'open', label: '开放', desc: '任何人都可以向我发起聊天' },
  { value: 'friends_only', label: '仅好友', desc: '只有好友可以向我发起聊天（默认）' },
  { value: 'whitelist', label: '白名单', desc: '只有白名单中的用户可以向我发起聊天' },
  { value: 'closed', label: '关闭', desc: '不接受任何聊天消息' },
];

const SCENE_TYPES = [
  { id: 'divination_order', label: '占卜订单', desc: '与占卜师的订单相关聊天' },
  { id: 'otc_order', label: 'OTC交易', desc: '与交易对手的OTC订单聊天' },
  { id: 'matchmaking', label: '婚恋匹配', desc: '婚恋匹配相关的聊天' },
  { id: 'market_maker', label: '承兑商', desc: '与OTC承兑商的聊天' },
];

export default function PrivacySettingsScreen() {
  const router = useRouter();

  const [settings, setSettings] = useState<PrivacySettings>({
    permissionLevel: 'friends_only',
    rejectedSceneTypes: [],
  });

  const handlePermissionChange = (level: PermissionLevel) => {
    setSettings(prev => ({ ...prev, permissionLevel: level }));
    // TODO: 调用链上 set_permission_level
  };

  const handleToggleScene = (sceneId: string) => {
    setSettings(prev => {
      const rejected = prev.rejectedSceneTypes.includes(sceneId)
        ? prev.rejectedSceneTypes.filter(s => s !== sceneId)
        : [...prev.rejectedSceneTypes, sceneId];
      return { ...prev, rejectedSceneTypes: rejected };
    });
    // TODO: 调用链上 set_rejected_scene_types
  };

  const handleManageBlocklist = () => {
    router.push('/friends?tab=blocked' as any);
  };

  const handleManageWhitelist = () => {
    router.push('/friends?tab=whitelist' as any);
  };

  const handleSave = () => {
    const msg = '隐私设置已保存';
    if (Platform.OS === 'web') {
      alert(msg);
    } else {
      Alert.alert('成功', msg);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>隐私设置</Text>
        <Pressable style={styles.saveButton} onPress={handleSave}>
          <Text style={styles.saveText}>保存</Text>
        </Pressable>
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>聊天权限</Text>
          <Text style={styles.sectionDesc}>设置谁可以向你发起聊天</Text>
          <View style={styles.optionsList}>
            {PERMISSION_LEVELS.map((level) => (
              <Pressable
                key={level.value}
                style={[
                  styles.optionItem,
                  settings.permissionLevel === level.value && styles.optionItemSelected,
                ]}
                onPress={() => handlePermissionChange(level.value)}
              >
                <View style={styles.optionContent}>
                  <Text style={styles.optionLabel}>{level.label}</Text>
                  <Text style={styles.optionDesc}>{level.desc}</Text>
                </View>
                <View
                  style={[
                    styles.radio,
                    settings.permissionLevel === level.value && styles.radioSelected,
                  ]}
                >
                  {settings.permissionLevel === level.value && (
                    <View style={styles.radioInner} />
                  )}
                </View>
              </Pressable>
            ))}
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>场景授权</Text>
          <Text style={styles.sectionDesc}>
            关闭某类场景后，即使有订单关联也无法发起聊天
          </Text>
          <View style={styles.toggleList}>
            {SCENE_TYPES.map((scene) => (
              <View key={scene.id} style={styles.toggleItem}>
                <View style={styles.toggleContent}>
                  <Text style={styles.toggleLabel}>{scene.label}</Text>
                  <Text style={styles.toggleDesc}>{scene.desc}</Text>
                </View>
                <Switch
                  value={!settings.rejectedSceneTypes.includes(scene.id)}
                  onValueChange={() => handleToggleScene(scene.id)}
                  trackColor={{ false: '#e5e7eb', true: '#c4b5fd' }}
                  thumbColor={
                    !settings.rejectedSceneTypes.includes(scene.id)
                      ? '#6D28D9'
                      : '#f4f3f4'
                  }
                />
              </View>
            ))}
          </View>
        </View>

        <View style={styles.section}>
          <Text style={styles.sectionTitle}>用户管理</Text>
          <Pressable style={styles.linkItem} onPress={handleManageBlocklist}>
            <View style={styles.linkContent}>
              <Text style={styles.linkLabel}>黑名单管理</Text>
              <Text style={styles.linkDesc}>被屏蔽的用户无法向你发送消息</Text>
            </View>
            <Text style={styles.linkArrow}>›</Text>
          </Pressable>
          <Pressable style={styles.linkItem} onPress={handleManageWhitelist}>
            <View style={styles.linkContent}>
              <Text style={styles.linkLabel}>白名单管理</Text>
              <Text style={styles.linkDesc}>白名单模式下允许聊天的用户</Text>
            </View>
            <Text style={styles.linkArrow}>›</Text>
          </Pressable>
          <Pressable
            style={styles.linkItem}
            onPress={() => router.push('/friends' as any)}
          >
            <View style={styles.linkContent}>
              <Text style={styles.linkLabel}>好友管理</Text>
              <Text style={styles.linkDesc}>管理你的好友列表</Text>
            </View>
            <Text style={styles.linkArrow}>›</Text>
          </Pressable>
        </View>

        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>💡 关于聊天权限</Text>
          <Text style={styles.infoText}>
            • 好友之间无视权限设置，始终可以互相聊天{'\n'}
            • 黑名单优先级最高，即使是好友也无法发送消息{'\n'}
            • 场景授权聊天（如订单相关）遵循场景授权设置{'\n'}
            • 所有设置会同步到区块链，确保隐私安全
          </Text>
        </View>
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
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  saveButton: {
    padding: 4,
  },
  saveText: {
    fontSize: 15,
    color: '#6D28D9',
    fontWeight: '600',
  },
  content: {
    flex: 1,
    padding: 16,
  },
  section: {
    marginBottom: 24,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 4,
  },
  sectionDesc: {
    fontSize: 13,
    color: '#6b7280',
    marginBottom: 12,
  },
  optionsList: {
    gap: 8,
  },
  optionItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  optionItemSelected: {
    borderColor: '#6D28D9',
    backgroundColor: '#f5f3ff',
  },
  optionContent: {
    flex: 1,
  },
  optionLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
  },
  optionDesc: {
    fontSize: 13,
    color: '#6b7280',
    marginTop: 2,
  },
  radio: {
    width: 22,
    height: 22,
    borderRadius: 11,
    borderWidth: 2,
    borderColor: '#d1d5db',
    justifyContent: 'center',
    alignItems: 'center',
  },
  radioSelected: {
    borderColor: '#6D28D9',
  },
  radioInner: {
    width: 12,
    height: 12,
    borderRadius: 6,
    backgroundColor: '#6D28D9',
  },
  toggleList: {
    backgroundColor: '#fff',
    borderRadius: 12,
    overflow: 'hidden',
  },
  toggleItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  toggleContent: {
    flex: 1,
  },
  toggleLabel: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1f2937',
  },
  toggleDesc: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  linkItem: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
  },
  linkContent: {
    flex: 1,
  },
  linkLabel: {
    fontSize: 15,
    fontWeight: '500',
    color: '#1f2937',
  },
  linkDesc: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  linkArrow: {
    fontSize: 20,
    color: '#d1d5db',
  },
  infoCard: {
    backgroundColor: '#eff6ff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 32,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#1e40af',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#1e3a8a',
    lineHeight: 20,
  },
});
