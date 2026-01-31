import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Modal, TextInput, Alert, Platform, ActivityIndicator } from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Plus, Flame, Droplets, Sun, Moon, Star, ChevronRight } from 'lucide-react-native';
import { useMeowstar, Pet } from '@/services/meowstar';

// 跨平台 Alert
const showAlert = (title: string, message: string, onOk?: () => void) => {
  if (Platform.OS === 'web') {
    window.alert(`${title}\n\n${message}`);
    onOk?.();
  } else {
    Alert.alert(title, message, [{ text: '确定', onPress: onOk }]);
  }
};

const ELEMENT_ICONS: Record<string, React.ReactNode> = {
  normal: <Star size={20} color="#888" />,
  fire: <Flame size={20} color="#FF6B6B" />,
  water: <Droplets size={20} color="#45B7D1" />,
  light: <Sun size={20} color="#F7DC6F" />,
  shadow: <Moon size={20} color="#BB8FCE" />,
};

const ELEMENT_COLORS: Record<string, string> = {
  normal: '#888',
  fire: '#FF6B6B',
  water: '#45B7D1',
  light: '#F7DC6F',
  shadow: '#BB8FCE',
};

const RARITY_COLORS: Record<string, string> = {
  common: '#888',
  rare: '#4ECDC4',
  epic: '#BB8FCE',
  legendary: '#F7DC6F',
  mythic: '#FF6B6B',
};

const RARITY_LABELS: Record<string, string> = {
  common: '普通',
  rare: '稀有',
  epic: '史诗',
  legendary: '传说',
  mythic: '神话',
};

function PetCard({ pet, onPress }: { pet: Pet; onPress: () => void }) {
  return (
    <TouchableOpacity style={styles.petCard} onPress={onPress}>
      <View style={[styles.petAvatar, { borderColor: ELEMENT_COLORS[pet.element] }]}>
        <Text style={styles.petEmoji}>🐱</Text>
        <View style={styles.levelBadge}>
          <Text style={styles.levelText}>Lv.{pet.level}</Text>
        </View>
      </View>
      <View style={styles.petInfo}>
        <View style={styles.petHeader}>
          <Text style={styles.petName}>{pet.name}</Text>
          {ELEMENT_ICONS[pet.element]}
        </View>
        <View style={styles.petMeta}>
          <View style={[styles.rarityBadge, { backgroundColor: RARITY_COLORS[pet.rarity] + '30' }]}>
            <Text style={[styles.rarityText, { color: RARITY_COLORS[pet.rarity] }]}>
              {RARITY_LABELS[pet.rarity]}
            </Text>
          </View>
          <Text style={styles.evolutionText}>进化阶段 {pet.evolutionStage}</Text>
        </View>
        <View style={styles.statsRow}>
          <Text style={styles.statText}>HP: {pet.hp}</Text>
          <Text style={styles.statText}>ATK: {pet.attack}</Text>
          <Text style={styles.statText}>DEF: {pet.defense}</Text>
        </View>
      </View>
      <ChevronRight size={24} color="#666" />
    </TouchableOpacity>
  );
}

export default function PetsScreen() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const [showHatchModal, setShowHatchModal] = useState(params.action === 'hatch');
  const [petName, setPetName] = useState('');
  const [isHatching, setIsHatching] = useState(false);

  // 使用全局状态
  const { pets, user, isLoading, updateBalance } = useMeowstar();

  // 孵化费用
  const HATCH_COST = 10;

  const handleHatch = async () => {
    if (!petName.trim()) {
      showAlert('提示', '请输入宠物名称');
      return;
    }

    if (!user || user.balance < HATCH_COST) {
      showAlert('余额不足', `孵化需要 ${HATCH_COST} COS，当前余额 ${user?.balance || 0} COS`);
      return;
    }

    setIsHatching(true);
    
    // 模拟孵化过程
    setTimeout(() => {
      const elements = ['normal', 'fire', 'water', 'light', 'shadow'] as const;
      const rarities = ['common', 'rare', 'epic', 'legendary', 'mythic'] as const;
      
      const newRarity = rarities[Math.floor(Math.random() * 3)];
      const newElement = elements[Math.floor(Math.random() * elements.length)];

      // 扣除费用
      updateBalance(-HATCH_COST);

      setShowHatchModal(false);
      setPetName('');
      setIsHatching(false);
      
      showAlert(
        '🎉 孵化成功！',
        `恭喜获得 ${RARITY_LABELS[newRarity]} ${newElement === 'fire' ? '火系' : newElement === 'water' ? '水系' : newElement === 'light' ? '光系' : newElement === 'shadow' ? '暗系' : '普通'} 宠物 "${petName}"！\n\n消耗: ${HATCH_COST} COS\n剩余余额: ${(user?.balance || 0) - HATCH_COST} COS`
      );
    }, 2000);
  };

  if (isLoading) {
    return (
      <View style={[styles.container, { justifyContent: 'center', alignItems: 'center' }]}>
        <ActivityIndicator size="large" color="#4ECDC4" />
        <Text style={{ color: '#888', marginTop: 16 }}>加载中...</Text>
      </View>
    );
  }

  return (
    <View style={styles.container}>
      <ScrollView style={styles.scrollView}>
        {pets.length === 0 ? (
          <View style={styles.emptyState}>
            <Text style={styles.emptyEmoji}>🥚</Text>
            <Text style={styles.emptyTitle}>还没有宠物</Text>
            <Text style={styles.emptyDescription}>孵化你的第一只喵星宠物吧！</Text>
          </View>
        ) : (
          <View style={styles.petList}>
            {pets.map((pet) => (
              <PetCard
                key={pet.id}
                pet={pet}
                onPress={() => router.push(`/meowstar/pet/${pet.id}` as any)}
              />
            ))}
          </View>
        )}
      </ScrollView>

      <TouchableOpacity
        style={styles.fab}
        onPress={() => setShowHatchModal(true)}
      >
        <Plus size={28} color="#fff" />
      </TouchableOpacity>

      <Modal
        visible={showHatchModal}
        animationType="slide"
        transparent={true}
        onRequestClose={() => setShowHatchModal(false)}
      >
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>🥚 孵化新宠物</Text>
            <Text style={styles.modalDescription}>
              消耗 10 COS 孵化一只随机宠物
            </Text>
            
            <TextInput
              style={styles.input}
              placeholder="给宠物起个名字..."
              placeholderTextColor="#666"
              value={petName}
              onChangeText={setPetName}
              maxLength={20}
            />

            <View style={styles.modalButtons}>
              <TouchableOpacity
                style={styles.cancelButton}
                onPress={() => setShowHatchModal(false)}
              >
                <Text style={styles.cancelButtonText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.confirmButton, isHatching && styles.disabledButton]}
                onPress={handleHatch}
                disabled={isHatching}
              >
                <Text style={styles.confirmButtonText}>
                  {isHatching ? '孵化中...' : '确认孵化'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  scrollView: {
    flex: 1,
  },
  petList: {
    padding: 16,
  },
  petCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 16,
    marginBottom: 12,
    flexDirection: 'row',
    alignItems: 'center',
  },
  petAvatar: {
    width: 64,
    height: 64,
    borderRadius: 32,
    backgroundColor: '#252540',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 2,
    marginRight: 12,
  },
  petEmoji: {
    fontSize: 32,
  },
  levelBadge: {
    position: 'absolute',
    bottom: -4,
    backgroundColor: '#333',
    paddingHorizontal: 6,
    paddingVertical: 2,
    borderRadius: 8,
  },
  levelText: {
    fontSize: 10,
    color: '#fff',
    fontWeight: 'bold',
  },
  petInfo: {
    flex: 1,
  },
  petHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  petName: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    marginRight: 8,
  },
  petMeta: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  rarityBadge: {
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
    marginRight: 8,
  },
  rarityText: {
    fontSize: 12,
    fontWeight: '600',
  },
  evolutionText: {
    fontSize: 12,
    color: '#888',
  },
  statsRow: {
    flexDirection: 'row',
  },
  statText: {
    fontSize: 12,
    color: '#666',
    marginRight: 12,
  },
  emptyState: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    paddingTop: 100,
  },
  emptyEmoji: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  emptyDescription: {
    fontSize: 14,
    color: '#888',
  },
  fab: {
    position: 'absolute',
    right: 20,
    bottom: 20,
    width: 60,
    height: 60,
    borderRadius: 30,
    backgroundColor: '#FF6B6B',
    justifyContent: 'center',
    alignItems: 'center',
    elevation: 4,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.25,
    shadowRadius: 4,
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    backgroundColor: '#1a1a2e',
    borderRadius: 20,
    padding: 24,
    width: '85%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    textAlign: 'center',
    marginBottom: 8,
  },
  modalDescription: {
    fontSize: 14,
    color: '#888',
    textAlign: 'center',
    marginBottom: 20,
  },
  input: {
    backgroundColor: '#252540',
    borderRadius: 12,
    padding: 16,
    fontSize: 16,
    color: '#fff',
    marginBottom: 20,
  },
  modalButtons: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  cancelButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#333',
    marginRight: 8,
    alignItems: 'center',
  },
  cancelButtonText: {
    fontSize: 16,
    color: '#888',
  },
  confirmButton: {
    flex: 1,
    padding: 14,
    borderRadius: 12,
    backgroundColor: '#FF6B6B',
    marginLeft: 8,
    alignItems: 'center',
  },
  confirmButtonText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
  },
  disabledButton: {
    opacity: 0.6,
  },
});
