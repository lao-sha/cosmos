import React, { useState } from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Alert, Platform } from 'react-native';
import { useRouter } from 'expo-router';
import { Swords, Shield, Zap, Trophy, Users } from 'lucide-react-native';

const showAlert = (title: string, message: string) => {
  if (Platform.OS === 'web') {
    window.alert(`${title}\n\n${message}`);
  } else {
    Alert.alert(title, message);
  }
};

type BattleMode = 'pve' | 'pvp';
type Difficulty = 'easy' | 'normal' | 'hard' | 'nightmare';

interface BattleStats {
  wins: number;
  losses: number;
  elo: number;
  winStreak: number;
}

export default function BattleScreen() {
  const router = useRouter();
  const [selectedMode, setSelectedMode] = useState<BattleMode>('pve');
  const [selectedDifficulty, setSelectedDifficulty] = useState<Difficulty>('normal');
  const [selectedPetId, setSelectedPetId] = useState<number | null>(1);
  const [isSearching, setIsSearching] = useState(false);

  const stats: BattleStats = {
    wins: 42,
    losses: 18,
    elo: 1250,
    winStreak: 5,
  };

  const difficulties: { key: Difficulty; label: string; reward: string; color: string }[] = [
    { key: 'easy', label: '简单', reward: '10 COS', color: '#4ECDC4' },
    { key: 'normal', label: '普通', reward: '25 COS', color: '#45B7D1' },
    { key: 'hard', label: '困难', reward: '50 COS', color: '#F7DC6F' },
    { key: 'nightmare', label: '噩梦', reward: '100 COS', color: '#FF6B6B' },
  ];

  const handleStartBattle = () => {
    if (!selectedPetId) {
      showAlert('提示', '请先选择一只宠物');
      return;
    }

    if (selectedMode === 'pvp') {
      setIsSearching(true);
      // 模拟匹配
      setTimeout(() => {
        setIsSearching(false);
        showAlert('匹配成功', '找到对手！战斗即将开始...');
        // PVP 模式跳转到战斗场景
        router.push(`/meowstar/battle-arena?difficulty=normal&mode=pvp` as any);
      }, 2000);
    } else {
      // PVE 模式直接跳转到战斗场景
      router.push(`/meowstar/battle-arena?difficulty=${selectedDifficulty}&mode=pve` as any);
    }
  };

  return (
    <ScrollView style={styles.container}>
      {/* 战绩统计 */}
      <View style={styles.statsCard}>
        <View style={styles.statsRow}>
          <View style={styles.statItem}>
            <Trophy size={24} color="#F7DC6F" />
            <Text style={styles.statValue}>{stats.wins}</Text>
            <Text style={styles.statLabel}>胜场</Text>
          </View>
          <View style={styles.statItem}>
            <Shield size={24} color="#888" />
            <Text style={styles.statValue}>{stats.losses}</Text>
            <Text style={styles.statLabel}>败场</Text>
          </View>
          <View style={styles.statItem}>
            <Zap size={24} color="#4ECDC4" />
            <Text style={styles.statValue}>{stats.elo}</Text>
            <Text style={styles.statLabel}>ELO</Text>
          </View>
          <View style={styles.statItem}>
            <Swords size={24} color="#FF6B6B" />
            <Text style={styles.statValue}>{stats.winStreak}</Text>
            <Text style={styles.statLabel}>连胜</Text>
          </View>
        </View>
      </View>

      {/* 模式选择 */}
      <Text style={styles.sectionTitle}>选择模式</Text>
      <View style={styles.modeContainer}>
        <TouchableOpacity
          style={[styles.modeCard, selectedMode === 'pve' && styles.modeCardActive]}
          onPress={() => setSelectedMode('pve')}
        >
          <Shield size={32} color={selectedMode === 'pve' ? '#4ECDC4' : '#666'} />
          <Text style={[styles.modeTitle, selectedMode === 'pve' && styles.modeTitleActive]}>
            PVE 冒险
          </Text>
          <Text style={styles.modeDescription}>挑战 AI 怪物</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.modeCard, selectedMode === 'pvp' && styles.modeCardActive]}
          onPress={() => setSelectedMode('pvp')}
        >
          <Users size={32} color={selectedMode === 'pvp' ? '#FF6B6B' : '#666'} />
          <Text style={[styles.modeTitle, selectedMode === 'pvp' && styles.modeTitleActive]}>
            PVP 对战
          </Text>
          <Text style={styles.modeDescription}>与玩家对战</Text>
        </TouchableOpacity>
      </View>

      {/* PVE 难度选择 */}
      {selectedMode === 'pve' && (
        <>
          <Text style={styles.sectionTitle}>选择难度</Text>
          <View style={styles.difficultyContainer}>
            {difficulties.map((diff) => (
              <TouchableOpacity
                key={diff.key}
                style={[
                  styles.difficultyCard,
                  selectedDifficulty === diff.key && { borderColor: diff.color },
                ]}
                onPress={() => setSelectedDifficulty(diff.key)}
              >
                <Text style={[styles.difficultyLabel, { color: diff.color }]}>
                  {diff.label}
                </Text>
                <Text style={styles.difficultyReward}>奖励: {diff.reward}</Text>
              </TouchableOpacity>
            ))}
          </View>
        </>
      )}

      {/* 选择宠物 */}
      <Text style={styles.sectionTitle}>选择宠物</Text>
      <View style={styles.petSelector}>
        <TouchableOpacity
          style={[styles.petOption, selectedPetId === 1 && styles.petOptionActive]}
          onPress={() => setSelectedPetId(1)}
        >
          <Text style={styles.petEmoji}>🐱</Text>
          <Text style={styles.petName}>小火 Lv.15</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.petOption, selectedPetId === 2 && styles.petOptionActive]}
          onPress={() => setSelectedPetId(2)}
        >
          <Text style={styles.petEmoji}>🐱</Text>
          <Text style={styles.petName}>水灵 Lv.22</Text>
        </TouchableOpacity>
      </View>

      {/* 开始战斗按钮 */}
      <TouchableOpacity
        style={[styles.startButton, isSearching && styles.startButtonSearching]}
        onPress={handleStartBattle}
        disabled={isSearching}
      >
        <Swords size={24} color="#fff" />
        <Text style={styles.startButtonText}>
          {isSearching ? '匹配中...' : selectedMode === 'pve' ? '开始战斗' : '开始匹配'}
        </Text>
      </TouchableOpacity>

      {/* 入场费提示 */}
      {selectedMode === 'pvp' && (
        <Text style={styles.feeNote}>
          PVP 对战入场费: 0.5 COS
        </Text>
      )}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
    padding: 16,
  },
  statsCard: {
    backgroundColor: '#1a1a2e',
    borderRadius: 16,
    padding: 20,
    marginBottom: 24,
  },
  statsRow: {
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
  sectionTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 12,
  },
  modeContainer: {
    flexDirection: 'row',
    marginBottom: 24,
  },
  modeCard: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    padding: 20,
    alignItems: 'center',
    marginHorizontal: 4,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  modeCardActive: {
    borderColor: '#4ECDC4',
  },
  modeTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#888',
    marginTop: 8,
  },
  modeTitleActive: {
    color: '#fff',
  },
  modeDescription: {
    fontSize: 12,
    color: '#666',
    marginTop: 4,
  },
  difficultyContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    marginBottom: 24,
  },
  difficultyCard: {
    width: '48%',
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
    marginHorizontal: '1%',
    borderWidth: 2,
    borderColor: '#333',
  },
  difficultyLabel: {
    fontSize: 16,
    fontWeight: 'bold',
  },
  difficultyReward: {
    fontSize: 12,
    color: '#888',
    marginTop: 4,
  },
  petSelector: {
    flexDirection: 'row',
    marginBottom: 24,
  },
  petOption: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
    marginHorizontal: 4,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  petOptionActive: {
    borderColor: '#FF6B6B',
  },
  petEmoji: {
    fontSize: 32,
    marginBottom: 8,
  },
  petName: {
    fontSize: 14,
    color: '#fff',
  },
  startButton: {
    backgroundColor: '#FF6B6B',
    borderRadius: 16,
    padding: 18,
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
  },
  startButtonSearching: {
    backgroundColor: '#666',
  },
  startButtonText: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
    marginLeft: 8,
  },
  feeNote: {
    fontSize: 12,
    color: '#888',
    textAlign: 'center',
    marginTop: 12,
  },
});
