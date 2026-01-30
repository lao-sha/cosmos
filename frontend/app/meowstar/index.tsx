import React from 'react';
import { View, Text, ScrollView, TouchableOpacity, StyleSheet, Image } from 'react-native';
import { useRouter } from 'expo-router';
import { Cat, Swords, Store, Coins, Vote, MessageCircle } from 'lucide-react-native';

interface FeatureCardProps {
  title: string;
  description: string;
  icon: React.ReactNode;
  route: string;
  color: string;
}

function FeatureCard({ title, description, icon, route, color }: FeatureCardProps) {
  const router = useRouter();
  
  return (
    <TouchableOpacity
      style={[styles.card, { borderLeftColor: color }]}
      onPress={() => router.push(route as any)}
    >
      <View style={[styles.iconContainer, { backgroundColor: color + '20' }]}>
        {icon}
      </View>
      <View style={styles.cardContent}>
        <Text style={styles.cardTitle}>{title}</Text>
        <Text style={styles.cardDescription}>{description}</Text>
      </View>
    </TouchableOpacity>
  );
}

export default function MeowstarHome() {
  const router = useRouter();

  const features: FeatureCardProps[] = [
    {
      title: '我的宠物',
      description: '查看、升级和进化你的喵星宠物',
      icon: <Cat size={28} color="#FF6B6B" />,
      route: '/meowstar/pets',
      color: '#FF6B6B',
    },
    {
      title: '战斗竞技',
      description: 'PVE 冒险或 PVP 对战',
      icon: <Swords size={28} color="#4ECDC4" />,
      route: '/meowstar/battle',
      color: '#4ECDC4',
    },
    {
      title: 'NFT 市场',
      description: '买卖稀有宠物 NFT',
      icon: <Store size={28} color="#45B7D1" />,
      route: '/meowstar/marketplace',
      color: '#45B7D1',
    },
    {
      title: 'COS 质押',
      description: '质押赚取收益，获得投票权',
      icon: <Coins size={28} color="#F7DC6F" />,
      route: '/meowstar/staking',
      color: '#F7DC6F',
    },
    {
      title: 'DAO 治理',
      description: '参与社区决策和提案投票',
      icon: <Vote size={28} color="#BB8FCE" />,
      route: '/meowstar/governance',
      color: '#BB8FCE',
    },
    {
      title: 'AI 陪伴',
      description: '与你的宠物聊天互动',
      icon: <MessageCircle size={28} color="#58D68D" />,
      route: '/meowstar/chat',
      color: '#58D68D',
    },
  ];

  return (
    <ScrollView style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>🐱 喵星宇宙</Text>
        <Text style={styles.subtitle}>Meowstar Universe</Text>
        <Text style={styles.description}>
          探索 AI 驱动的宠物养成世界，收集、培养、战斗！
        </Text>
      </View>

      <View style={styles.statsContainer}>
        <View style={styles.statItem}>
          <Text style={styles.statValue}>0</Text>
          <Text style={styles.statLabel}>我的宠物</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>0</Text>
          <Text style={styles.statLabel}>战斗胜场</Text>
        </View>
        <View style={styles.statDivider} />
        <View style={styles.statItem}>
          <Text style={styles.statValue}>0</Text>
          <Text style={styles.statLabel}>质押 COS</Text>
        </View>
      </View>

      <View style={styles.featuresContainer}>
        {features.map((feature, index) => (
          <FeatureCard key={index} {...feature} />
        ))}
      </View>

      <TouchableOpacity
        style={styles.hatchButton}
        onPress={() => router.push('/meowstar/pets?action=hatch' as any)}
      >
        <Text style={styles.hatchButtonText}>🥚 孵化新宠物</Text>
      </TouchableOpacity>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0f0f1a',
  },
  header: {
    padding: 24,
    alignItems: 'center',
  },
  title: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 16,
    color: '#888',
    marginBottom: 12,
  },
  description: {
    fontSize: 14,
    color: '#aaa',
    textAlign: 'center',
  },
  statsContainer: {
    flexDirection: 'row',
    backgroundColor: '#1a1a2e',
    marginHorizontal: 16,
    borderRadius: 16,
    padding: 20,
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statValue: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
  },
  statLabel: {
    fontSize: 12,
    color: '#888',
    marginTop: 4,
  },
  statDivider: {
    width: 1,
    backgroundColor: '#333',
  },
  featuresContainer: {
    padding: 16,
  },
  card: {
    backgroundColor: '#1a1a2e',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    flexDirection: 'row',
    alignItems: 'center',
    borderLeftWidth: 4,
  },
  iconContainer: {
    width: 56,
    height: 56,
    borderRadius: 12,
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 16,
  },
  cardContent: {
    flex: 1,
  },
  cardTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
    marginBottom: 4,
  },
  cardDescription: {
    fontSize: 14,
    color: '#888',
  },
  hatchButton: {
    backgroundColor: '#FF6B6B',
    marginHorizontal: 16,
    marginVertical: 24,
    padding: 18,
    borderRadius: 16,
    alignItems: 'center',
  },
  hatchButtonText: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#fff',
  },
});
