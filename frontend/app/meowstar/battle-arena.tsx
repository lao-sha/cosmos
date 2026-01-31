import React, { useState, useEffect, useRef } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Animated,
  Platform,
  Dimensions,
} from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Swords, Shield, Zap, Heart, ArrowLeft, Star } from 'lucide-react-native';

const { width: SCREEN_WIDTH } = Dimensions.get('window');

// 跨平台 Alert
const showAlert = (title: string, message: string, onOk?: () => void) => {
  if (Platform.OS === 'web') {
    window.alert(`${title}\n\n${message}`);
    onOk?.();
  } else {
    const { Alert } = require('react-native');
    Alert.alert(title, message, [{ text: '确定', onPress: onOk }]);
  }
};

// 技能定义
interface Skill {
  id: string;
  name: string;
  damage: number;
  manaCost: number;
  type: 'attack' | 'defense' | 'special';
  icon: string;
  color: string;
  description: string;
}

// 战斗单位
interface BattleUnit {
  id: string;
  name: string;
  emoji: string;
  level: number;
  maxHp: number;
  currentHp: number;
  maxMana: number;
  currentMana: number;
  attack: number;
  defense: number;
  skills: Skill[];
  isPlayer: boolean;
}

// 战斗日志
interface BattleLog {
  id: number;
  message: string;
  type: 'damage' | 'heal' | 'skill' | 'info';
}

const PLAYER_SKILLS: Skill[] = [
  { id: 'attack', name: '普通攻击', damage: 20, manaCost: 0, type: 'attack', icon: '⚔️', color: '#FF6B6B', description: '基础攻击' },
  { id: 'fireball', name: '火球术', damage: 45, manaCost: 20, type: 'special', icon: '🔥', color: '#FF4500', description: '释放火球造成大量伤害' },
  { id: 'thunder', name: '雷击', damage: 35, manaCost: 15, type: 'special', icon: '⚡', color: '#FFD700', description: '召唤雷电攻击敌人' },
  { id: 'shield', name: '护盾', damage: -30, manaCost: 25, type: 'defense', icon: '🛡️', color: '#4ECDC4', description: '恢复生命值' },
];

const ENEMY_SKILLS: Skill[] = [
  { id: 'bite', name: '撕咬', damage: 18, manaCost: 0, type: 'attack', icon: '🦷', color: '#FF6B6B', description: '野兽撕咬' },
  { id: 'claw', name: '利爪', damage: 30, manaCost: 10, type: 'special', icon: '🐾', color: '#FF4500', description: '锋利的爪击' },
];

export default function BattleArenaScreen() {
  const router = useRouter();
  const params = useLocalSearchParams();
  const difficulty = (params.difficulty as string) || 'normal';

  // 根据难度设置敌人属性
  const getDifficultyMultiplier = () => {
    switch (difficulty) {
      case 'easy': return 0.7;
      case 'normal': return 1.0;
      case 'hard': return 1.3;
      case 'nightmare': return 1.8;
      default: return 1.0;
    }
  };

  const multiplier = getDifficultyMultiplier();

  // 初始化玩家
  const [player, setPlayer] = useState<BattleUnit>({
    id: 'player',
    name: '小火',
    emoji: '🐱',
    level: 15,
    maxHp: 200,
    currentHp: 200,
    maxMana: 100,
    currentMana: 100,
    attack: 25,
    defense: 15,
    skills: PLAYER_SKILLS,
    isPlayer: true,
  });

  // 初始化敌人
  const [enemy, setEnemy] = useState<BattleUnit>({
    id: 'enemy',
    name: '暗影狼',
    emoji: '🐺',
    level: Math.floor(12 * multiplier),
    maxHp: Math.floor(150 * multiplier),
    currentHp: Math.floor(150 * multiplier),
    maxMana: 50,
    currentMana: 50,
    attack: Math.floor(20 * multiplier),
    defense: Math.floor(10 * multiplier),
    skills: ENEMY_SKILLS,
    isPlayer: false,
  });

  // 战斗状态
  const [isPlayerTurn, setIsPlayerTurn] = useState(true);
  const [battleLogs, setBattleLogs] = useState<BattleLog[]>([]);
  const [logIdCounter, setLogIdCounter] = useState(0);
  const [isBattleOver, setIsBattleOver] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);
  const [turn, setTurn] = useState(1);

  // 动画值
  const playerShake = useRef(new Animated.Value(0)).current;
  const enemyShake = useRef(new Animated.Value(0)).current;
  const playerScale = useRef(new Animated.Value(1)).current;
  const enemyScale = useRef(new Animated.Value(1)).current;
  const damageOpacity = useRef(new Animated.Value(0)).current;
  const [damageText, setDamageText] = useState('');
  const [damagePosition, setDamagePosition] = useState<'player' | 'enemy'>('enemy');

  // 添加战斗日志
  const addLog = (message: string, type: BattleLog['type']) => {
    setLogIdCounter(prev => {
      const newId = prev + 1;
      setBattleLogs(logs => [...logs.slice(-4), { id: newId, message, type }]);
      return newId;
    });
  };

  // 播放受击动画
  const playHitAnimation = (target: 'player' | 'enemy', damage: number) => {
    const shakeAnim = target === 'player' ? playerShake : enemyShake;
    setDamageText(damage > 0 ? `-${damage}` : `+${Math.abs(damage)}`);
    setDamagePosition(target);

    Animated.sequence([
      Animated.timing(damageOpacity, { toValue: 1, duration: 100, useNativeDriver: true }),
      Animated.parallel([
        Animated.sequence([
          Animated.timing(shakeAnim, { toValue: 10, duration: 50, useNativeDriver: true }),
          Animated.timing(shakeAnim, { toValue: -10, duration: 50, useNativeDriver: true }),
          Animated.timing(shakeAnim, { toValue: 5, duration: 50, useNativeDriver: true }),
          Animated.timing(shakeAnim, { toValue: 0, duration: 50, useNativeDriver: true }),
        ]),
        Animated.timing(damageOpacity, { toValue: 0, duration: 800, useNativeDriver: true }),
      ]),
    ]).start();
  };

  // 播放攻击动画
  const playAttackAnimation = (attacker: 'player' | 'enemy') => {
    const scaleAnim = attacker === 'player' ? playerScale : enemyScale;

    Animated.sequence([
      Animated.timing(scaleAnim, { toValue: 1.2, duration: 150, useNativeDriver: true }),
      Animated.timing(scaleAnim, { toValue: 1, duration: 150, useNativeDriver: true }),
    ]).start();
  };

  // 计算伤害
  const calculateDamage = (attacker: BattleUnit, defender: BattleUnit, skill: Skill): number => {
    if (skill.type === 'defense') {
      return skill.damage; // 负数表示治疗
    }
    const baseDamage = skill.damage + attacker.attack;
    const reduction = defender.defense * 0.5;
    const finalDamage = Math.max(1, Math.floor(baseDamage - reduction + (Math.random() * 10 - 5)));
    return finalDamage;
  };

  // 玩家使用技能
  const useSkill = (skill: Skill) => {
    if (!isPlayerTurn || isAnimating || isBattleOver) return;
    if (player.currentMana < skill.manaCost) {
      showAlert('魔力不足', `使用 ${skill.name} 需要 ${skill.manaCost} 魔力`);
      return;
    }

    setIsAnimating(true);

    // 扣除魔力
    setPlayer(prev => ({ ...prev, currentMana: prev.currentMana - skill.manaCost }));

    // 播放攻击动画
    playAttackAnimation('player');

    setTimeout(() => {
      if (skill.type === 'defense') {
        // 治疗技能
        const healAmount = Math.abs(skill.damage);
        setPlayer(prev => ({
          ...prev,
          currentHp: Math.min(prev.maxHp, prev.currentHp + healAmount),
        }));
        playHitAnimation('player', skill.damage);
        addLog(`${player.name} 使用 ${skill.icon}${skill.name}，恢复 ${healAmount} 生命！`, 'heal');
      } else {
        // 攻击技能
        const damage = calculateDamage(player, enemy, skill);
        setEnemy(prev => {
          const newHp = Math.max(0, prev.currentHp - damage);
          return { ...prev, currentHp: newHp };
        });
        playHitAnimation('enemy', damage);
        addLog(`${player.name} 使用 ${skill.icon}${skill.name}，造成 ${damage} 点伤害！`, 'damage');
      }

      setTimeout(() => {
        setIsAnimating(false);
        setIsPlayerTurn(false);
      }, 500);
    }, 300);
  };

  // 敌人AI行动
  useEffect(() => {
    if (isPlayerTurn || isAnimating || isBattleOver) return;

    const enemyAction = setTimeout(() => {
      setIsAnimating(true);

      // 简单AI：随机选择技能
      const availableSkills = enemy.skills.filter(s => enemy.currentMana >= s.manaCost);
      const skill = availableSkills.length > 0
        ? availableSkills[Math.floor(Math.random() * availableSkills.length)]
        : enemy.skills[0];

      // 扣除魔力
      setEnemy(prev => ({ ...prev, currentMana: Math.max(0, prev.currentMana - skill.manaCost) }));

      // 播放攻击动画
      playAttackAnimation('enemy');

      setTimeout(() => {
        const damage = calculateDamage(enemy, player, skill);
        setPlayer(prev => {
          const newHp = Math.max(0, prev.currentHp - damage);
          return { ...prev, currentHp: newHp };
        });
        playHitAnimation('player', damage);
        addLog(`${enemy.name} 使用 ${skill.icon}${skill.name}，造成 ${damage} 点伤害！`, 'damage');

        setTimeout(() => {
          setIsAnimating(false);
          setIsPlayerTurn(true);
          setTurn(prev => prev + 1);
          // 回合结束恢复少量魔力
          setPlayer(prev => ({ ...prev, currentMana: Math.min(prev.maxMana, prev.currentMana + 5) }));
          setEnemy(prev => ({ ...prev, currentMana: Math.min(prev.maxMana, prev.currentMana + 5) }));
        }, 500);
      }, 300);
    }, 1000);

    return () => clearTimeout(enemyAction);
  }, [isPlayerTurn, isAnimating, isBattleOver]);

  // 检查战斗结束
  useEffect(() => {
    if (isBattleOver) return;

    if (enemy.currentHp <= 0) {
      setIsBattleOver(true);
      const reward = difficulty === 'easy' ? 10 : difficulty === 'normal' ? 25 : difficulty === 'hard' ? 50 : 100;
      addLog(`🎉 战斗胜利！获得 ${reward} COS 奖励！`, 'info');
      setTimeout(() => {
        showAlert('战斗胜利！', `恭喜！你击败了 ${enemy.name}！\n获得奖励: ${reward} COS`, () => {
          router.back();
        });
      }, 1000);
    } else if (player.currentHp <= 0) {
      setIsBattleOver(true);
      addLog('💀 战斗失败...', 'info');
      setTimeout(() => {
        showAlert('战斗失败', `你被 ${enemy.name} 击败了...`, () => {
          router.back();
        });
      }, 1000);
    }
  }, [player.currentHp, enemy.currentHp, isBattleOver]);

  // 渲染血条
  const renderHealthBar = (current: number, max: number, color: string) => {
    const percentage = (current / max) * 100;
    return (
      <View style={styles.healthBarContainer}>
        <View style={styles.healthBarBg}>
          <View style={[styles.healthBarFill, { width: `${percentage}%`, backgroundColor: color }]} />
        </View>
        <Text style={styles.healthText}>{current}/{max}</Text>
      </View>
    );
  };

  // 渲染魔力条
  const renderManaBar = (current: number, max: number) => {
    const percentage = (current / max) * 100;
    return (
      <View style={styles.manaBarContainer}>
        <View style={styles.manaBarBg}>
          <View style={[styles.manaBarFill, { width: `${percentage}%` }]} />
        </View>
        <Text style={styles.manaText}>{current}/{max}</Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* 顶部信息栏 */}
      <View style={styles.header}>
        <TouchableOpacity onPress={() => router.back()} style={styles.backButton}>
          <ArrowLeft size={24} color="#fff" />
        </TouchableOpacity>
        <View style={styles.turnInfo}>
          <Text style={styles.turnText}>回合 {turn}</Text>
          <Text style={styles.difficultyText}>
            {difficulty === 'easy' ? '简单' : difficulty === 'normal' ? '普通' : difficulty === 'hard' ? '困难' : '噩梦'}
          </Text>
        </View>
        <View style={styles.turnIndicator}>
          <Text style={[styles.turnIndicatorText, { color: isPlayerTurn ? '#4ECDC4' : '#FF6B6B' }]}>
            {isPlayerTurn ? '你的回合' : '敌方回合'}
          </Text>
        </View>
      </View>

      {/* 战斗区域 */}
      <View style={styles.battleArea}>
        {/* 敌人 */}
        <View style={styles.enemySection}>
          <View style={styles.unitInfo}>
            <Text style={styles.unitName}>{enemy.name} Lv.{enemy.level}</Text>
            {renderHealthBar(enemy.currentHp, enemy.maxHp, '#FF6B6B')}
            {renderManaBar(enemy.currentMana, enemy.maxMana)}
          </View>
          <Animated.View
            style={[
              styles.unitAvatar,
              {
                transform: [
                  { translateX: enemyShake },
                  { scale: enemyScale },
                ],
              },
            ]}
          >
            <Text style={styles.unitEmoji}>{enemy.emoji}</Text>
            {damagePosition === 'enemy' && (
              <Animated.Text style={[styles.damageText, { opacity: damageOpacity }]}>
                {damageText}
              </Animated.Text>
            )}
          </Animated.View>
        </View>

        {/* VS 分隔 */}
        <View style={styles.vsContainer}>
          <Swords size={32} color="#FFD700" />
          <Text style={styles.vsText}>VS</Text>
        </View>

        {/* 玩家 */}
        <View style={styles.playerSection}>
          <Animated.View
            style={[
              styles.unitAvatar,
              {
                transform: [
                  { translateX: playerShake },
                  { scale: playerScale },
                ],
              },
            ]}
          >
            <Text style={styles.unitEmoji}>{player.emoji}</Text>
            {damagePosition === 'player' && (
              <Animated.Text style={[styles.damageText, { opacity: damageOpacity }]}>
                {damageText}
              </Animated.Text>
            )}
          </Animated.View>
          <View style={styles.unitInfo}>
            <Text style={styles.unitName}>{player.name} Lv.{player.level}</Text>
            {renderHealthBar(player.currentHp, player.maxHp, '#4ECDC4')}
            {renderManaBar(player.currentMana, player.maxMana)}
          </View>
        </View>
      </View>

      {/* 战斗日志 */}
      <View style={styles.logContainer}>
        {battleLogs.map((log) => (
          <Text
            key={log.id}
            style={[
              styles.logText,
              log.type === 'damage' && styles.logDamage,
              log.type === 'heal' && styles.logHeal,
              log.type === 'info' && styles.logInfo,
            ]}
          >
            {log.message}
          </Text>
        ))}
      </View>

      {/* 技能面板 */}
      <View style={styles.skillPanel}>
        <Text style={styles.skillPanelTitle}>技能</Text>
        <View style={styles.skillGrid}>
          {player.skills.map((skill) => (
            <TouchableOpacity
              key={skill.id}
              style={[
                styles.skillButton,
                { borderColor: skill.color },
                (!isPlayerTurn || isAnimating || player.currentMana < skill.manaCost) && styles.skillButtonDisabled,
              ]}
              onPress={() => useSkill(skill)}
              disabled={!isPlayerTurn || isAnimating || isBattleOver}
            >
              <Text style={styles.skillIcon}>{skill.icon}</Text>
              <Text style={styles.skillName}>{skill.name}</Text>
              <View style={styles.skillCost}>
                {skill.manaCost > 0 && (
                  <>
                    <Zap size={12} color="#45B7D1" />
                    <Text style={styles.skillCostText}>{skill.manaCost}</Text>
                  </>
                )}
              </View>
            </TouchableOpacity>
          ))}
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingHorizontal: 16,
    paddingTop: Platform.OS === 'web' ? 16 : 50,
    paddingBottom: 12,
    backgroundColor: '#1a1a2e',
  },
  backButton: {
    padding: 8,
  },
  turnInfo: {
    alignItems: 'center',
  },
  turnText: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
  },
  difficultyText: {
    fontSize: 12,
    color: '#888',
  },
  turnIndicator: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 12,
    backgroundColor: '#2a2a3e',
  },
  turnIndicatorText: {
    fontSize: 12,
    fontWeight: 'bold',
  },
  battleArea: {
    flex: 1,
    paddingHorizontal: 16,
    paddingVertical: 20,
  },
  enemySection: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-end',
    marginBottom: 20,
  },
  playerSection: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'flex-start',
    marginTop: 20,
  },
  unitInfo: {
    flex: 1,
    paddingHorizontal: 12,
  },
  unitName: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 8,
  },
  unitAvatar: {
    width: 100,
    height: 100,
    borderRadius: 50,
    backgroundColor: '#1a1a2e',
    justifyContent: 'center',
    alignItems: 'center',
    borderWidth: 3,
    borderColor: '#333',
  },
  unitEmoji: {
    fontSize: 48,
  },
  damageText: {
    position: 'absolute',
    top: -20,
    fontSize: 24,
    fontWeight: 'bold',
    color: '#FF6B6B',
    textShadowColor: '#000',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 2,
  },
  healthBarContainer: {
    marginBottom: 4,
  },
  healthBarBg: {
    height: 16,
    backgroundColor: '#333',
    borderRadius: 8,
    overflow: 'hidden',
  },
  healthBarFill: {
    height: '100%',
    borderRadius: 8,
  },
  healthText: {
    position: 'absolute',
    width: '100%',
    textAlign: 'center',
    fontSize: 10,
    color: '#fff',
    fontWeight: 'bold',
    lineHeight: 16,
  },
  manaBarContainer: {
    marginTop: 4,
  },
  manaBarBg: {
    height: 10,
    backgroundColor: '#333',
    borderRadius: 5,
    overflow: 'hidden',
  },
  manaBarFill: {
    height: '100%',
    backgroundColor: '#45B7D1',
    borderRadius: 5,
  },
  manaText: {
    position: 'absolute',
    width: '100%',
    textAlign: 'center',
    fontSize: 8,
    color: '#fff',
    fontWeight: 'bold',
    lineHeight: 10,
  },
  vsContainer: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 20,
  },
  vsText: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#FFD700',
    marginTop: 4,
  },
  logContainer: {
    backgroundColor: '#1a1a2e',
    padding: 12,
    marginHorizontal: 16,
    borderRadius: 12,
    minHeight: 100,
    maxHeight: 120,
  },
  logText: {
    fontSize: 12,
    color: '#888',
    marginBottom: 4,
  },
  logDamage: {
    color: '#FF6B6B',
  },
  logHeal: {
    color: '#4ECDC4',
  },
  logInfo: {
    color: '#FFD700',
    fontWeight: 'bold',
  },
  skillPanel: {
    backgroundColor: '#1a1a2e',
    padding: 16,
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
  },
  skillPanelTitle: {
    fontSize: 14,
    fontWeight: 'bold',
    color: '#888',
    marginBottom: 12,
    textAlign: 'center',
  },
  skillGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
  },
  skillButton: {
    width: '48%',
    backgroundColor: '#2a2a3e',
    borderRadius: 12,
    padding: 12,
    marginBottom: 8,
    alignItems: 'center',
    borderWidth: 2,
  },
  skillButtonDisabled: {
    opacity: 0.5,
  },
  skillIcon: {
    fontSize: 24,
    marginBottom: 4,
  },
  skillName: {
    fontSize: 12,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 4,
  },
  skillCost: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  skillCostText: {
    fontSize: 10,
    color: '#45B7D1',
    marginLeft: 2,
  },
});
