// 喵星宇宙类型定义

export type Element = 'normal' | 'fire' | 'water' | 'light' | 'shadow';
export type Rarity = 'common' | 'rare' | 'epic' | 'legendary' | 'mythic';
export type PetStatus = 'idle' | 'battle' | 'training' | 'listed';

export interface Pet {
  id: number;
  name: string;
  element: Element;
  rarity: Rarity;
  level: number;
  experience: number;
  expToNextLevel: number;
  evolutionStage: number;
  maxEvolutionStage: number;
  hp: number;
  maxHp: number;
  attack: number;
  defense: number;
  speed: number;
  critRate: number;
  status: PetStatus;
  personality: {
    extroversion: number;
    warmth: number;
    humor: number;
    curiosity: number;
    responsibility: number;
  };
  skills: Skill[];
  battleStats: {
    wins: number;
    losses: number;
    winRate: number;
  };
  createdAt: number;
}

export interface Skill {
  id: string;
  name: string;
  level: number;
  damage: number;
  manaCost: number;
  type: 'attack' | 'defense' | 'special';
  icon: string;
  description: string;
}

export interface User {
  address: string;
  balance: number;
  votePower: number;
  pets: number[];
  createdAt: number;
}

export interface MarketListing {
  id: number;
  petId: number;
  petName: string;
  element: Element;
  rarity: Rarity;
  level: number;
  price: number;
  seller: string;
  type: 'fixed' | 'auction';
  highestBid?: number;
  highestBidder?: string;
  endsAt: number;
  createdAt: number;
}

export interface Proposal {
  id: number;
  title: string;
  description: string;
  type: 'general' | 'parameter' | 'treasury' | 'emergency';
  proposer: string;
  status: 'active' | 'passed' | 'rejected' | 'executed';
  yesVotes: number;
  noVotes: number;
  totalVotes: number;
  quorum: number;
  endsAt: number;
  createdAt: number;
  voters: { [address: string]: 'yes' | 'no' };
}

export interface BattleRecord {
  id: number;
  petId: number;
  opponentId: number;
  won: boolean;
  reward: number;
  damage: number;
  damageTaken: number;
  timestamp: number;
}

export interface Transaction {
  id: number;
  type: 'level_up' | 'evolve' | 'buy' | 'sell' | 'battle_reward' | 'vote';
  amount: number;
  description: string;
  timestamp: number;
}
