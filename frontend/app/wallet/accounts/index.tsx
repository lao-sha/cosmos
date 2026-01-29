import { AccountInfo } from '@/src/lib/wallet';
import { useWalletStore } from '@/src/stores/wallet';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Alert,
    Platform,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

export default function AccountsScreen() {
  const router = useRouter();
  const { accounts, activeAccountId, switchAccount, deleteAccount, updateAccountName } = useWalletStore();
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');

  const handleSwitch = async (account: AccountInfo) => {
    await switchAccount(account.id);
  };

  const handleStartEdit = (account: AccountInfo) => {
    setEditingId(account.id);
    setEditName(account.name);
  };

  const handleSaveEdit = async () => {
    if (editingId && editName.trim()) {
      await updateAccountName(editingId, editName.trim());
    }
    setEditingId(null);
    setEditName('');
  };

  const handleDelete = (account: AccountInfo) => {
    if (!account.isImported && account.derivationIndex === 0) {
      if (Platform.OS === 'web') {
        alert('主账户不能删除');
      } else {
        Alert.alert('提示', '主账户不能删除');
      }
      return;
    }

    const doDelete = async () => {
      const success = await deleteAccount(account.id);
      if (!success) {
        if (Platform.OS === 'web') {
          alert('删除失败');
        } else {
          Alert.alert('错误', '删除失败');
        }
      }
    };

    if (Platform.OS === 'web') {
      if (confirm(`确定要删除账户 "${account.name}" 吗？此操作不可恢复。`)) {
        doDelete();
      }
    } else {
      Alert.alert(
        '确认删除',
        `确定要删除账户 "${account.name}" 吗？此操作不可恢复。`,
        [
          { text: '取消', style: 'cancel' },
          { text: '删除', style: 'destructive', onPress: doDelete },
        ]
      );
    }
  };

  const truncateAddress = (address: string) => {
    return `${address.slice(0, 8)}...${address.slice(-6)}`;
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable onPress={() => router.back()} style={styles.backButton}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>账户管理</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content}>
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>我的账户 ({accounts.length})</Text>
          
          {accounts.map((account) => (
            <View key={account.id} style={styles.accountCard}>
              <Pressable 
                style={styles.accountMain}
                onPress={() => handleSwitch(account)}
              >
                <View style={styles.accountAvatar}>
                  <Text style={styles.avatarText}>
                    {account.name[0].toUpperCase()}
                  </Text>
                </View>
                <View style={styles.accountInfo}>
                  {editingId === account.id ? (
                    <TextInput
                      style={styles.editInput}
                      value={editName}
                      onChangeText={setEditName}
                      onBlur={handleSaveEdit}
                      onSubmitEditing={handleSaveEdit}
                      autoFocus
                    />
                  ) : (
                    <View style={styles.nameRow}>
                      <Text style={styles.accountName}>{account.name}</Text>
                      {account.id === activeAccountId && (
                        <View style={styles.activeBadge}>
                          <Text style={styles.activeText}>当前</Text>
                        </View>
                      )}
                      {account.isImported && (
                        <View style={styles.importedBadge}>
                          <Text style={styles.importedText}>导入</Text>
                        </View>
                      )}
                    </View>
                  )}
                  <Text style={styles.accountAddress}>
                    {truncateAddress(account.address)}
                  </Text>
                </View>
              </Pressable>
              
              <View style={styles.accountActions}>
                <Pressable
                  style={styles.actionBtn}
                  onPress={() => handleStartEdit(account)}
                >
                  <Text style={styles.actionBtnText}>✏️</Text>
                </Pressable>
                {(account.isImported || account.derivationIndex !== 0) && (
                  <Pressable
                    style={styles.actionBtn}
                    onPress={() => handleDelete(account)}
                  >
                    <Text style={styles.actionBtnText}>🗑️</Text>
                  </Pressable>
                )}
              </View>
            </View>
          ))}
        </View>

        <View style={styles.addSection}>
          <Pressable
            style={({ pressed }) => [
              styles.addButton,
              pressed && styles.addButtonPressed,
            ]}
            onPress={() => router.push('/wallet/accounts/create')}
          >
            <Text style={styles.addIcon}>➕</Text>
            <Text style={styles.addText}>创建新账户</Text>
            <Text style={styles.addDesc}>从主助记词派生新账户</Text>
          </Pressable>

          <Pressable
            style={({ pressed }) => [
              styles.addButton,
              pressed && styles.addButtonPressed,
            ]}
            onPress={() => router.push('/wallet/accounts/import')}
          >
            <Text style={styles.addIcon}>📥</Text>
            <Text style={styles.addText}>导入账户</Text>
            <Text style={styles.addDesc}>使用其他助记词导入</Text>
          </Pressable>
        </View>

        <View style={styles.infoCard}>
          <Text style={styles.infoTitle}>💡 关于多账户</Text>
          <Text style={styles.infoText}>
            • HD派生账户使用同一助记词，只需备份一次{'\n'}
            • 导入账户需要单独备份其助记词{'\n'}
            • 主账户（索引0）不可删除{'\n'}
            • 切换账户后，所有操作将使用新账户
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
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  section: {
    padding: 16,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#6b7280',
    marginBottom: 12,
  },
  accountCard: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 12,
    marginBottom: 8,
  },
  accountMain: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
  },
  accountAvatar: {
    width: 48,
    height: 48,
    borderRadius: 24,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  avatarText: {
    fontSize: 20,
    fontWeight: '600',
    color: '#fff',
  },
  accountInfo: {
    flex: 1,
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    flexWrap: 'wrap',
    gap: 6,
  },
  accountName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#1f2937',
  },
  activeBadge: {
    backgroundColor: '#dcfce7',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  activeText: {
    fontSize: 10,
    color: '#16a34a',
    fontWeight: '500',
  },
  importedBadge: {
    backgroundColor: '#fef3c7',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  importedText: {
    fontSize: 10,
    color: '#d97706',
    fontWeight: '500',
  },
  accountAddress: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 2,
  },
  editInput: {
    fontSize: 16,
    color: '#1f2937',
    borderBottomWidth: 1,
    borderBottomColor: '#6D28D9',
    paddingVertical: 4,
  },
  accountActions: {
    flexDirection: 'row',
    gap: 8,
  },
  actionBtn: {
    padding: 8,
  },
  actionBtnText: {
    fontSize: 18,
  },
  addSection: {
    padding: 16,
    paddingTop: 0,
    gap: 12,
  },
  addButton: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#e5e7eb',
    borderStyle: 'dashed',
  },
  addButtonPressed: {
    backgroundColor: '#f9fafb',
  },
  addIcon: {
    fontSize: 24,
    marginBottom: 8,
  },
  addText: {
    fontSize: 16,
    fontWeight: '500',
    color: '#1f2937',
  },
  addDesc: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 4,
  },
  infoCard: {
    margin: 16,
    marginTop: 0,
    backgroundColor: '#f5f3ff',
    borderRadius: 12,
    padding: 16,
  },
  infoTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#6D28D9',
    marginBottom: 8,
  },
  infoText: {
    fontSize: 13,
    color: '#6b7280',
    lineHeight: 20,
  },
});
