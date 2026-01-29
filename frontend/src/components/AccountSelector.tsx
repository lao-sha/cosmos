import { useRouter } from 'expo-router';
import {
    Modal,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    View,
} from 'react-native';
import { AccountInfo } from '../lib/wallet';
import { useCurrentAccount, useWalletStore } from '../stores/wallet';

interface AccountSelectorProps {
  visible: boolean;
  onClose: () => void;
}

export function AccountSelector({ visible, onClose }: AccountSelectorProps) {
  const router = useRouter();
  const { accounts, activeAccountId, switchAccount } = useWalletStore();
  const currentAccount = useCurrentAccount();

  const handleSelectAccount = async (account: AccountInfo) => {
    await switchAccount(account.id);
    onClose();
  };

  const handleCreateAccount = () => {
    onClose();
    router.push('/wallet/accounts/create');
  };

  const handleImportAccount = () => {
    onClose();
    router.push('/wallet/accounts/import');
  };

  const handleManageAccounts = () => {
    onClose();
    router.push('/wallet/accounts');
  };

  const truncateAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <View style={styles.overlay}>
        <Pressable style={styles.backdrop} onPress={onClose} />
        <View style={styles.container}>
          <View style={styles.header}>
            <Text style={styles.title}>选择账户</Text>
            <Pressable onPress={onClose} style={styles.closeButton}>
              <Text style={styles.closeText}>✕</Text>
            </Pressable>
          </View>

          <ScrollView style={styles.accountList}>
            {accounts.map((account) => (
              <Pressable
                key={account.id}
                style={[
                  styles.accountItem,
                  account.id === activeAccountId && styles.accountItemActive,
                ]}
                onPress={() => handleSelectAccount(account)}
              >
                <View style={styles.accountAvatar}>
                  <Text style={styles.avatarText}>
                    {account.name[0].toUpperCase()}
                  </Text>
                </View>
                <View style={styles.accountInfo}>
                  <View style={styles.accountNameRow}>
                    <Text style={styles.accountName}>{account.name}</Text>
                    {account.isImported && (
                      <View style={styles.importedBadge}>
                        <Text style={styles.importedText}>导入</Text>
                      </View>
                    )}
                    {!account.isImported && account.derivationIndex === 0 && (
                      <View style={styles.primaryBadge}>
                        <Text style={styles.primaryText}>主账户</Text>
                      </View>
                    )}
                  </View>
                  <Text style={styles.accountAddress}>
                    {truncateAddress(account.address)}
                  </Text>
                </View>
                {account.id === activeAccountId && (
                  <Text style={styles.checkmark}>✓</Text>
                )}
              </Pressable>
            ))}
          </ScrollView>

          <View style={styles.actions}>
            <Pressable
              style={({ pressed }) => [
                styles.actionButton,
                pressed && styles.actionPressed,
              ]}
              onPress={handleCreateAccount}
            >
              <Text style={styles.actionIcon}>➕</Text>
              <Text style={styles.actionText}>创建账户</Text>
            </Pressable>

            <Pressable
              style={({ pressed }) => [
                styles.actionButton,
                pressed && styles.actionPressed,
              ]}
              onPress={handleImportAccount}
            >
              <Text style={styles.actionIcon}>📥</Text>
              <Text style={styles.actionText}>导入账户</Text>
            </Pressable>

            <Pressable
              style={({ pressed }) => [
                styles.actionButton,
                pressed && styles.actionPressed,
              ]}
              onPress={handleManageAccounts}
            >
              <Text style={styles.actionIcon}>⚙️</Text>
              <Text style={styles.actionText}>管理账户</Text>
            </Pressable>
          </View>
        </View>
      </View>
    </Modal>
  );
}

interface AccountHeaderProps {
  onPress: () => void;
}

export function AccountHeader({ onPress }: AccountHeaderProps) {
  const currentAccount = useCurrentAccount();

  if (!currentAccount) {
    return null;
  }

  const truncateAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  return (
    <Pressable
      style={({ pressed }) => [
        styles.headerButton,
        pressed && styles.headerPressed,
      ]}
      onPress={onPress}
    >
      <View style={styles.headerAvatar}>
        <Text style={styles.headerAvatarText}>
          {currentAccount.name[0].toUpperCase()}
        </Text>
      </View>
      <View style={styles.headerInfo}>
        <Text style={styles.headerName}>{currentAccount.name}</Text>
        <Text style={styles.headerAddress}>
          {truncateAddress(currentAccount.address)}
        </Text>
      </View>
      <Text style={styles.headerArrow}>▼</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    justifyContent: 'flex-end',
  },
  backdrop: {
    ...StyleSheet.absoluteFillObject,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
  },
  container: {
    backgroundColor: '#fff',
    borderTopLeftRadius: 20,
    borderTopRightRadius: 20,
    maxHeight: '70%',
    paddingBottom: 34,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  title: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
  },
  closeButton: {
    padding: 4,
  },
  closeText: {
    fontSize: 20,
    color: '#9ca3af',
  },
  accountList: {
    maxHeight: 300,
  },
  accountItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  accountItemActive: {
    backgroundColor: '#f5f3ff',
  },
  accountAvatar: {
    width: 44,
    height: 44,
    borderRadius: 22,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  avatarText: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
  },
  accountInfo: {
    flex: 1,
  },
  accountNameRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  accountName: {
    fontSize: 16,
    fontWeight: '500',
    color: '#1f2937',
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
  primaryBadge: {
    backgroundColor: '#dbeafe',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 4,
  },
  primaryText: {
    fontSize: 10,
    color: '#2563eb',
    fontWeight: '500',
  },
  accountAddress: {
    fontSize: 13,
    color: '#9ca3af',
    marginTop: 2,
  },
  checkmark: {
    fontSize: 18,
    color: '#6D28D9',
    fontWeight: '600',
  },
  actions: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  actionButton: {
    flex: 1,
    alignItems: 'center',
    padding: 12,
    backgroundColor: '#f3f4f6',
    borderRadius: 12,
  },
  actionPressed: {
    backgroundColor: '#e5e7eb',
  },
  actionIcon: {
    fontSize: 20,
    marginBottom: 4,
  },
  actionText: {
    fontSize: 12,
    color: '#374151',
    fontWeight: '500',
  },
  headerButton: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 8,
    borderRadius: 12,
    backgroundColor: '#f3f4f6',
  },
  headerPressed: {
    backgroundColor: '#e5e7eb',
  },
  headerAvatar: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: '#6D28D9',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 8,
  },
  headerAvatarText: {
    fontSize: 14,
    fontWeight: '600',
    color: '#fff',
  },
  headerInfo: {
    marginRight: 8,
  },
  headerName: {
    fontSize: 14,
    fontWeight: '500',
    color: '#1f2937',
  },
  headerAddress: {
    fontSize: 11,
    color: '#9ca3af',
  },
  headerArrow: {
    fontSize: 10,
    color: '#9ca3af',
  },
});
