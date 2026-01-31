import React, { useState, useEffect } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Alert, Platform, ActivityIndicator } from 'react-native';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { Flame, Droplets, Sun, Moon, Star, TrendingUp, Zap, Shield, Heart, ArrowUp, Sparkles } from 'lucide-react-native';
import { useMeowstar } from '@/services/meowstar';

// 跨平台 Alert
const showAlert = (title: string, message: string, onOk?: () => void) => {
  if (Platform.OS === 'web') {
    window.alert(`${title}\n\n${message}`);
    onOk?.();
  } else {
    Alert.alert(title, message, [{ text: '确定', onPress: onOk }]);
  }
};

const ELEMENT_CONFIG = {
  normal: { icon: Star, color: '#888', name: '普通' },
  fire: { icon: Flame, color: '#FF6B6B', name: '火焰' },
  water: { icon: Droplets, color: '#45B7D1', name: '水' },
  light: { icon: Sun, color: '#F7DC6F', name: '光明' },
  shadow: { icon: Moon, color: '#BB8FCE', name: '暗影' },
};

const RARITY_CONFIG = {
  common: { color: '#888', name: '普通' },
  rare: { color: '#4ECDC4', name: '稀有' },
  epic: { color: '#BB8FCE', name: '史诗' },
  legendary: { color: '#F7DC6F', name: '传说' },
  mythic: { color: '#FF6B6B', name: '神话' },
};

export default function PetDetailScreen() {
  const { id } = useLocalSearchParams();
  const router = useRouter();
  const [isLevelingUp, setIsLevelingUp] = useState(false);
  const [isEvolving, setIsEvolving] = useState(false);
  
  // 使用全局状态
  const { user, getPetById, levelUpPet, evolvePet, isLoading } = useMeowstar();
  
  // 获取宠物数据
  const pet = getPetById(Number(id));

  // 加载中或宠物不存在
  if (isLoading) {
    return (
      <View style={[styles.container, { justifyContent: 'center', alignItems: 'center' }]}>
        <ActivityIndicator size="large" color="#4ECDC4" />
        <Text style={{ color: '#888', marginTop: 16 }}>加载中...</Text>
      </View>
    );
  }

  if (!pet) {
    return (
      <View style={[styles.container, { justifyContent: 'center', alignItems: 'center' }]}>
        <Text style={{ color: '#888', fontSize: 18 }}>宠物不存在</Text>
      </View>
    );
  }

  const ElementIcon = ELEMENT_CONFIG[pet.element]?.icon || Star;
  const expPercent = (pet.experience / pet.expToNextLevel) * 100;

  const handleLevelUp = async () => {
    setIsLevelingUp(true);
    const result = await levelUpPet(pet.id);
    setIsLevelingUp(false);
    showAlert(result.success ? '升级成功！' : '升级失败', result.message);
  };

  const handleEvolve = async () => {
    setIsEvolving(true);
    const result = await evolvePet(pet.id);
    setIsEvolving(false);
    showAlert(result.success ? '进化成功！' : '进化失败', result.message);
  };

  return (
    <ScrollView style={styles.container}>
      {/* 用户余额 */}
      <View style={styles.balanceCard}>
        <Text style={styles.balanceLabel}>💰 我的余额</Text>
        <Text style={styles.balanceValue}>{user?.balance || 0} COS</Text>
      </View>
      
      {/* 宠物头像和基本信息 */}
      <View style={styles.header}>
        <View style={[styles.avatarContainer, { borderColor: ELEMENT_CONFIG[pet.element].color }]}>
          <Text style={styles.avatarEmoji}>🐱</Text>
          <View style={styles.elementBadge}>
            <ElementIcon size={16} color={ELEMENT_CONFIG[pet.element].color} />
          </View>
        </View>
        
        <Text style={styles.petName}>{pet.name}</Text>
        
        <View style={styles.badges}>
          <View style={[styles.rarityBadge, { backgroundColor: RARITY_CONFIG[pet.rarity].color + '30' }]}>
            <Text style={[styles.rarityText, { color: RARITY_CONFIG[pet.rarity].color }]}>
              {RARITY_CONFIG[pet.rarity].name}
            </Text>
          </View>
          <View style={styles.levelBadge}>
            <Text style={styles.levelText}>Lv.{pet.level}</Text>
          </View>
          <View style={styles.evolutionBadge}>
            <Sparkles size={12} color="#F7DC6F" />
            <Text style={styles.evolutionText}>阶段 {pet.evolutionStage}/{pet.maxEvolutionStage}</Text>
          </View>
        </View>
      </View>

      {/* 经验条 */}
      <View style={styles.expContainer}>
        <View style={styles.expHeader}>
          <Text style={styles.expLabel}>经验值</Text>
          <Text style={styles.expValue}>{pet.experience} / {pet.expToNextLevel}</Text>
        </View>
        <View style={styles.expBar}>
          <View style={[styles.expFill, { width: `${expPercent}%` }]} />
        </View>
      </View>

      {/* 属性 */}
      <View style={styles.statsCard}>
        <Text style={styles.sectionTitle}>属性</Text>
        <View style={styles.statsGrid}>
          <View style={styles.statItem}>
            <Heart size={20} color="#FF6B6B" />
            <Text style={styles.statValue}>{pet.hp}</Text>
            <Text style={styles.statLabel}>生命值</Text>
          </View>
          <View style={styles.statItem}>
            <Zap size={20} color="#F7DC6F" />
            <Text style={styles.statValue}>{pet.attack}</Text>
            <Text style={styles.statLabel}>攻击力</Text>
          </View>
          <View style={styles.statItem}>
            <Shield size={20} color="#4ECDC4" />
            <Text style={styles.statValue}>{pet.defense}</Text>
            <Text style={styles.statLabel}>防御力</Text>
          </View>
          <View style={styles.statItem}>
            <TrendingUp size={20} color="#BB8FCE" />
            <Text style={styles.statValue}>{pet.speed}</Text>
            <Text style={styles.statLabel}>速度</Text>
          </View>
        </View>
      </View>

      {/* 技能 */}
      <View style={styles.skillsCard}>
        <Text style={styles.sectionTitle}>技能</Text>
        {pet.skills.map((skill, index) => (
          <View key={index} style={styles.skillItem}>
            <View style={styles.skillIcon}>
              <Flame size={20} color="#FF6B6B" />
            </View>
            <View style={styles.skillInfo}>
              <Text style={styles.skillName}>{skill.name}</Text>
              <Text style={styles.skillLevel}>Lv.{skill.level}</Text>
            </View>
            {skill.damage > 0 && (
              <Text style={styles.skillDamage}>伤害: {skill.damage}</Text>
            )}
          </View>
        ))}
      </View>

      {/* 战绩 */}
      <View style={styles.battleCard}>
        <Text style={styles.sectionTitle}>战绩</Text>
        <View style={styles.battleStats}>
          <View style={styles.battleStatItem}>
            <Text style={styles.battleStatValue}>{pet.battleStats.wins}</Text>
            <Text style={styles.battleStatLabel}>胜场</Text>
          </View>
          <View style={styles.battleStatItem}>
            <Text style={styles.battleStatValue}>{pet.battleStats.losses}</Text>
            <Text style={styles.battleStatLabel}>败场</Text>
          </View>
          <View style={styles.battleStatItem}>
            <Text style={[styles.battleStatValue, { color: '#4ECDC4' }]}>
              {pet.battleStats.winRate}%
            </Text>
            <Text style={styles.battleStatLabel}>胜率</Text>
          </View>
        </View>
      </View>

      {/* 操作按钮 */}
      <View style={styles.actions}>
        <TouchableOpacity
          style={[styles.actionButton, styles.levelUpButton]}
          onPress={handleLevelUp}
          disabled={isLevelingUp}
        >
          <ArrowUp size={20} color="#fff" />
          <Text style={styles.actionButtonText}>
            {isLevelingUp ? '升级中...' : '升级 (10 COS)'}
          </Text>
        </TouchableOpacity>
        
        <TouchableOpacity
          style={[styles.actionButton, styles.evolveButton]}
          onPress={handleEvolve}
          disabled={isEvolving}
        >
          <Sparkles size={20} color="#fff" />
          <Text style={styles.actionButtonText}>
            {isEvolving ? '进化中...' : '进化'}
          </Text>
        </TouchableOpacity>
      </View>

      <View style={styles.secondaryActions}>
        <TouchableOpacity
          style={styles.secondaryButton}
          onPress={() => router.push('/meowstar/battle' as any)}
        >
          <Text style={styles.secondaryButtonText}>去战斗</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={styles.secondaryButton}
          onPress={() => router.push('/meowstar/chat' as any)}
        >
          <Text style={styles.secondaryButtonText}>聊天</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.secondaryButton, styles.sellButton]}
          onPress={() => router.push(`/meowstar/pet/sell?id=${id}` as any)}
        >
          <Text style={styles.sellButtonText}>出售</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  balanceCard: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    backgroundColor: '#1a1a2e',
    marginHorizontal: 16,
    marginTop: 16,
    padding: 16,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#4ECDC430',
  },
  balanceLabel: {
    fontSize: 14,
    color: '#888',
  },
  balanceValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#4ECDC4',
  },
  header: {
    alignItems: 'center',
    padding: 24,
  },
  avatarContainer: {
    width: 120,
    height: 120,
    borderRadius: 60,
    backgroundColor: '#1a1a2e',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 4,
    marginBottom: 16,
  },
  avatarEmoji: {
    fontSize: 64,
  },
  elementBadge: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    backgroundColor: '#1a1a2e',
    padding: 8,
    borderRadius: 16,
    borderWidth: 2,
    borderColor: '#0f0f1a',
  },
  petName: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 12,
  },
  badges: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  rarityBadge: {
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 8,
    marginRight: 8,
  },
  rarityText: {
    fontSize: 14,
    fontWeight: '600',
  },
  levelBadge: {
    backgroundColor: '#333',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 8,
    marginRight: 8,
  },
  levelText: {
    fontSize: 14,
    color: '#fff',
    fontWeight: '600',
  },
  evolutionBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#F7DC6F20',
    paddingHorizontal: 10,
    paddingVertical: 4,
    borderRadius: 8,
  },
  evolutionText: {
    fontSize: 12,
    color: '#F7DC6F',
    marginLeft: 4,
  },
  expContainer: {
    marginHorizontal: 16,
    marginBottom: 20,
  },
  expHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 8,
  },
  expLabel: {
    fontSize: 14,
    color: '#888',
  },
  expValue: {
    fontSize: 14,
    color: '#4ECDC4',
  },
  expBar: {
    height: 8,
    backgroundColor: '#252540',
    borderRadius: 4,
    overflow: 'hidden',
  },
  expFill: {
    height: '100%',
    backgroundColor: '#4ECDC4',
    borderRadius: 4,
  },
  statsCard: {
    backgroundColor: '#1a1a2e',
    marginHorizontal: 16,
    borderRadius: 16,
    padding: 16,
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 16,
  },
  statsGrid: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#fff',
    marginTop: 8,
  },
  statLabel: {
    fontSize: 12,
    color: '#888',
    marginTop: 4,
  },
  skillsCard: {
    backgroundColor: '#1a1a2e',
    marginHorizontal: 16,
    borderRadius: 16,
    padding: 16,
    marginBottom: 16,
  },
  skillItem: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#252540',
  },
  skillIcon: {
    width: 40,
    height: 40,
    borderRadius: 8,
    backgroundColor: '#FF6B6B20',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 12,
  },
  skillInfo: {
    flex: 1,
  },
  skillName: {
    fontSize: 16,
    color: '#fff',
    fontWeight: '600',
  },
  skillLevel: {
    fontSize: 12,
    color: '#888',
  },
  skillDamage: {
    fontSize: 14,
    color: '#FF6B6B',
  },
  battleCard: {
    backgroundColor: '#1a1a2e',
    marginHorizontal: 16,
    borderRadius: 16,
    padding: 16,
    marginBottom: 16,
  },
  battleStats: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  battleStatItem: {
    alignItems: 'center',
  },
  battleStatValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
  },
  battleStatLabel: {
    fontSize: 12,
    color: '#888',
    marginTop: 4,
  },
  actions: {
    flexDirection: 'row',
    marginHorizontal: 16,
    marginBottom: 12,
  },
  actionButton: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 16,
    borderRadius: 12,
    marginHorizontal: 4,
  },
  levelUpButton: {
    backgroundColor: '#4ECDC4',
  },
  evolveButton: {
    backgroundColor: '#BB8FCE',
  },
  actionButtonText: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginLeft: 8,
  },
  secondaryActions: {
    flexDirection: 'row',
    marginHorizontal: 16,
    marginBottom: 32,
  },
  secondaryButton: {
    flex: 1,
    backgroundColor: '#252540',
    padding: 14,
    borderRadius: 12,
    alignItems: 'center',
    marginHorizontal: 4,
  },
  secondaryButtonText: {
    fontSize: 14,
    color: '#888',
  },
  sellButton: {
    backgroundColor: '#FF6B6B20',
  },
  sellButtonText: {
    fontSize: 14,
    color: '#FF6B6B',
  },
});
