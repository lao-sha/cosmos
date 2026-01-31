// 喵星宇宙全局状态管理
import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Pet, User, MarketListing, Proposal, Transaction, Rarity, Element } from './types';
import * as Storage from './storage';
import { createInitialUser, createInitialPets, createInitialMarket, createInitialProposals } from './initialData';

interface MeowstarContextType {
  // 用户
  user: User | null;
  updateBalance: (amount: number) => void;
  
  // 宠物
  pets: Pet[];
  getPetById: (id: number) => Pet | undefined;
  levelUpPet: (petId: number) => Promise<{ success: boolean; message: string }>;
  evolvePet: (petId: number) => Promise<{ success: boolean; message: string }>;
  updatePetStats: (petId: number, stats: Partial<Pet>) => void;
  
  // 市场
  marketListings: MarketListing[];
  buyPet: (listingId: number) => Promise<{ success: boolean; message: string }>;
  placeBid: (listingId: number, amount: number) => Promise<{ success: boolean; message: string }>;
  listPetForSale: (petId: number, price: number, type: 'fixed' | 'auction', duration: number) => Promise<{ success: boolean; message: string }>;
  
  // 治理
  proposals: Proposal[];
  vote: (proposalId: number, approve: boolean) => Promise<{ success: boolean; message: string }>;
  createProposal: (title: string, description: string, type: Proposal['type']) => Promise<{ success: boolean; message: string }>;
  
  // 交易记录
  transactions: Transaction[];
  
  // 战斗
  recordBattle: (petId: number, won: boolean, reward: number) => void;
  
  // 加载状态
  isLoading: boolean;
  
  // 刷新数据
  refreshData: () => Promise<void>;
}

const MeowstarContext = createContext<MeowstarContextType | undefined>(undefined);

// 费用常量
const LEVEL_UP_COST = 10;
const EVOLVE_COST = 50;
const PROPOSAL_DEPOSIT = 100;
const MIN_VOTE_POWER = 10000;

export function MeowstarProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [pets, setPets] = useState<Pet[]>([]);
  const [marketListings, setMarketListings] = useState<MarketListing[]>([]);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // 初始化数据
  useEffect(() => {
    initializeData();
  }, []);

  const initializeData = async () => {
    setIsLoading(true);
    try {
      // 尝试加载本地数据
      let loadedUser = await Storage.loadUser();
      let loadedPets = await Storage.loadPets();
      let loadedMarket = await Storage.loadMarket();
      let loadedProposals = await Storage.loadProposals();
      let loadedTransactions = await Storage.loadTransactions();

      // 如果没有数据，使用初始数据
      if (!loadedUser) {
        loadedUser = createInitialUser();
        await Storage.saveUser(loadedUser);
      }
      if (loadedPets.length === 0) {
        loadedPets = createInitialPets();
        await Storage.savePets(loadedPets);
      }
      if (loadedMarket.length === 0) {
        loadedMarket = createInitialMarket();
        await Storage.saveMarket(loadedMarket);
      }
      if (loadedProposals.length === 0) {
        loadedProposals = createInitialProposals();
        await Storage.saveProposals(loadedProposals);
      }

      setUser(loadedUser);
      setPets(loadedPets);
      setMarketListings(loadedMarket);
      setProposals(loadedProposals);
      setTransactions(loadedTransactions);
    } catch (error) {
      console.error('Failed to initialize data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // 刷新数据
  const refreshData = async () => {
    await initializeData();
  };

  // 更新余额
  const updateBalance = (amount: number) => {
    if (!user) return;
    const newUser = { ...user, balance: user.balance + amount };
    setUser(newUser);
    Storage.saveUser(newUser);
  };

  // 添加交易记录
  const addTransaction = (type: Transaction['type'], amount: number, description: string) => {
    const newTransaction: Transaction = {
      id: Date.now(),
      type,
      amount,
      description,
      timestamp: Date.now(),
    };
    const newTransactions = [newTransaction, ...transactions].slice(0, 100);
    setTransactions(newTransactions);
    Storage.saveTransactions(newTransactions);
  };

  // 获取宠物
  const getPetById = (id: number): Pet | undefined => {
    return pets.find(p => p.id === id);
  };

  // 升级宠物
  const levelUpPet = async (petId: number): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };
    if (user.balance < LEVEL_UP_COST) {
      return { success: false, message: `余额不足，升级需要 ${LEVEL_UP_COST} COS，当前余额 ${user.balance} COS` };
    }

    const pet = pets.find(p => p.id === petId);
    if (!pet) return { success: false, message: '宠物不存在' };

    // 属性提升
    const hpBonus = 5;
    const attackBonus = 3;
    const defenseBonus = 2;
    const speedBonus = 2;

    const updatedPet: Pet = {
      ...pet,
      level: pet.level + 1,
      hp: pet.hp + hpBonus,
      maxHp: pet.maxHp + hpBonus,
      attack: pet.attack + attackBonus,
      defense: pet.defense + defenseBonus,
      speed: pet.speed + speedBonus,
      experience: 0,
      expToNextLevel: Math.floor(pet.expToNextLevel * 1.2),
    };

    // 更新状态
    const newPets = pets.map(p => p.id === petId ? updatedPet : p);
    setPets(newPets);
    await Storage.savePets(newPets);

    // 扣除费用
    const newUser = { ...user, balance: user.balance - LEVEL_UP_COST };
    setUser(newUser);
    await Storage.saveUser(newUser);

    // 记录交易
    addTransaction('level_up', -LEVEL_UP_COST, `${pet.name} 升级到 ${updatedPet.level} 级`);

    return {
      success: true,
      message: `🎉 ${pet.name} 升到了 ${updatedPet.level} 级！\n\n属性提升:\n• 生命值 +${hpBonus} → ${updatedPet.hp}\n• 攻击力 +${attackBonus} → ${updatedPet.attack}\n• 防御力 +${defenseBonus} → ${updatedPet.defense}\n• 速度 +${speedBonus} → ${updatedPet.speed}\n\n消耗: ${LEVEL_UP_COST} COS\n剩余余额: ${newUser.balance} COS`,
    };
  };

  // 进化宠物
  const evolvePet = async (petId: number): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };
    
    const pet = pets.find(p => p.id === petId);
    if (!pet) return { success: false, message: '宠物不存在' };

    const requiredLevel = 10 * (pet.evolutionStage + 1);
    if (pet.level < requiredLevel) {
      return { success: false, message: `需要达到 ${requiredLevel} 级才能进化` };
    }

    if (pet.evolutionStage >= pet.maxEvolutionStage) {
      return { success: false, message: '已达到最高进化阶段' };
    }

    if (user.balance < EVOLVE_COST) {
      return { success: false, message: `余额不足，进化需要 ${EVOLVE_COST} COS，当前余额 ${user.balance} COS` };
    }

    // 属性提升 10%
    const evolutionBonus = 1.1;
    const updatedPet: Pet = {
      ...pet,
      evolutionStage: pet.evolutionStage + 1,
      hp: Math.floor(pet.hp * evolutionBonus),
      maxHp: Math.floor(pet.maxHp * evolutionBonus),
      attack: Math.floor(pet.attack * evolutionBonus),
      defense: Math.floor(pet.defense * evolutionBonus),
      speed: Math.floor(pet.speed * evolutionBonus),
    };

    // 更新状态
    const newPets = pets.map(p => p.id === petId ? updatedPet : p);
    setPets(newPets);
    await Storage.savePets(newPets);

    // 扣除费用
    const newUser = { ...user, balance: user.balance - EVOLVE_COST };
    setUser(newUser);
    await Storage.saveUser(newUser);

    // 记录交易
    addTransaction('evolve', -EVOLVE_COST, `${pet.name} 进化到第 ${updatedPet.evolutionStage} 阶段`);

    return {
      success: true,
      message: `✨ ${pet.name} 进化到了第 ${updatedPet.evolutionStage} 阶段！\n\n进化奖励:\n• 解锁新技能\n• 全属性 +10%\n• 外观变化\n\n消耗: ${EVOLVE_COST} COS\n剩余余额: ${newUser.balance} COS`,
    };
  };

  // 更新宠物属性
  const updatePetStats = (petId: number, stats: Partial<Pet>) => {
    const newPets = pets.map(p => p.id === petId ? { ...p, ...stats } : p);
    setPets(newPets);
    Storage.savePets(newPets);
  };

  // 购买宠物
  const buyPet = async (listingId: number): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };

    const listing = marketListings.find(l => l.id === listingId);
    if (!listing) return { success: false, message: '商品不存在' };

    if (listing.type === 'auction') {
      return { success: false, message: '拍卖商品请使用出价功能' };
    }

    if (user.balance < listing.price) {
      return { success: false, message: `余额不足，需要 ${listing.price} COS，当前余额 ${user.balance} COS` };
    }

    // 创建新宠物
    const newPet: Pet = {
      id: Date.now(),
      name: listing.petName,
      element: listing.element,
      rarity: listing.rarity,
      level: listing.level,
      experience: 0,
      expToNextLevel: listing.level * 200,
      evolutionStage: 1,
      maxEvolutionStage: 4,
      hp: 100 + listing.level * 5,
      maxHp: 100 + listing.level * 5,
      attack: 20 + listing.level * 2,
      defense: 15 + listing.level * 1.5,
      speed: 30 + listing.level * 1.5,
      critRate: 10,
      status: 'idle',
      personality: {
        extroversion: Math.floor(Math.random() * 100),
        warmth: Math.floor(Math.random() * 100),
        humor: Math.floor(Math.random() * 100),
        curiosity: Math.floor(Math.random() * 100),
        responsibility: Math.floor(Math.random() * 100),
      },
      skills: [],
      battleStats: { wins: 0, losses: 0, winRate: 0 },
      createdAt: Date.now(),
    };

    // 更新宠物列表
    const newPets = [...pets, newPet];
    setPets(newPets);
    await Storage.savePets(newPets);

    // 更新用户
    const newUser = {
      ...user,
      balance: user.balance - listing.price,
      pets: [...user.pets, newPet.id],
    };
    setUser(newUser);
    await Storage.saveUser(newUser);

    // 移除市场商品
    const newListings = marketListings.filter(l => l.id !== listingId);
    setMarketListings(newListings);
    await Storage.saveMarket(newListings);

    // 记录交易
    addTransaction('buy', -listing.price, `购买宠物 ${listing.petName}`);

    return {
      success: true,
      message: `🎉 成功购买 ${listing.petName}！\n\n花费: ${listing.price} COS\n剩余余额: ${newUser.balance} COS\n\n宠物已添加到你的背包中。`,
    };
  };

  // 出价
  const placeBid = async (listingId: number, amount: number): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };

    const listing = marketListings.find(l => l.id === listingId);
    if (!listing) return { success: false, message: '商品不存在' };

    if (listing.type !== 'auction') {
      return { success: false, message: '该商品不是拍卖商品' };
    }

    const minBid = (listing.highestBid || listing.price) + 1;
    if (amount < minBid) {
      return { success: false, message: `出价必须高于 ${minBid - 1} COS` };
    }

    if (user.balance < amount) {
      return { success: false, message: `余额不足，需要 ${amount} COS，当前余额 ${user.balance} COS` };
    }

    // 更新出价
    const updatedListing: MarketListing = {
      ...listing,
      highestBid: amount,
      highestBidder: user.address,
    };

    const newListings = marketListings.map(l => l.id === listingId ? updatedListing : l);
    setMarketListings(newListings);
    await Storage.saveMarket(newListings);

    return {
      success: true,
      message: `🎯 出价成功！\n\n出价金额: ${amount} COS\n\n如果在拍卖结束时你的出价最高，将自动完成交易。`,
    };
  };

  // 上架宠物
  const listPetForSale = async (
    petId: number,
    price: number,
    type: 'fixed' | 'auction',
    duration: number
  ): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };

    const pet = pets.find(p => p.id === petId);
    if (!pet) return { success: false, message: '宠物不存在' };

    if (pet.status !== 'idle') {
      return { success: false, message: '宠物当前状态无法上架' };
    }

    // 创建上架信息
    const newListing: MarketListing = {
      id: Date.now(),
      petId: pet.id,
      petName: pet.name,
      element: pet.element,
      rarity: pet.rarity,
      level: pet.level,
      price,
      seller: user.address,
      type,
      endsAt: Date.now() + duration,
      createdAt: Date.now(),
    };

    // 更新宠物状态
    const updatedPet = { ...pet, status: 'listed' as const };
    const newPets = pets.map(p => p.id === petId ? updatedPet : p);
    setPets(newPets);
    await Storage.savePets(newPets);

    // 添加到市场
    const newListings = [...marketListings, newListing];
    setMarketListings(newListings);
    await Storage.saveMarket(newListings);

    return {
      success: true,
      message: `✅ ${pet.name} 已成功上架！\n\n出售方式: ${type === 'fixed' ? '一口价' : '拍卖'}\n价格: ${price} COS`,
    };
  };

  // 投票
  const vote = async (proposalId: number, approve: boolean): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };

    const proposal = proposals.find(p => p.id === proposalId);
    if (!proposal) return { success: false, message: '提案不存在' };

    if (proposal.status !== 'active') {
      return { success: false, message: '提案已结束' };
    }

    if (proposal.voters[user.address]) {
      return { success: false, message: '你已经投过票了' };
    }

    // 更新投票
    const updatedProposal: Proposal = {
      ...proposal,
      yesVotes: approve ? proposal.yesVotes + user.votePower : proposal.yesVotes,
      noVotes: approve ? proposal.noVotes : proposal.noVotes + user.votePower,
      totalVotes: proposal.totalVotes + user.votePower,
      voters: { ...proposal.voters, [user.address]: approve ? 'yes' : 'no' },
    };

    const newProposals = proposals.map(p => p.id === proposalId ? updatedProposal : p);
    setProposals(newProposals);
    await Storage.saveProposals(newProposals);

    // 记录交易
    addTransaction('vote', 0, `对提案 "${proposal.title}" 投${approve ? '赞成' : '反对'}票`);

    return {
      success: true,
      message: `✅ 投票成功！\n\n你投了 ${approve ? '赞成' : '反对'} 票\n投票权重: ${user.votePower}`,
    };
  };

  // 创建提案
  const createProposal = async (
    title: string,
    description: string,
    type: Proposal['type']
  ): Promise<{ success: boolean; message: string }> => {
    if (!user) return { success: false, message: '用户未登录' };

    if (user.votePower < MIN_VOTE_POWER) {
      return { success: false, message: `创建提案需要至少 ${MIN_VOTE_POWER} 投票权重` };
    }

    if (user.balance < PROPOSAL_DEPOSIT) {
      return { success: false, message: `创建提案需要 ${PROPOSAL_DEPOSIT} COS 押金` };
    }

    // 创建提案
    const newProposal: Proposal = {
      id: Date.now(),
      title,
      description,
      type,
      proposer: user.address,
      status: 'active',
      yesVotes: 0,
      noVotes: 0,
      totalVotes: 0,
      quorum: 12500,
      endsAt: Date.now() + 86400000 * 7, // 7天
      createdAt: Date.now(),
      voters: {},
    };

    const newProposals = [newProposal, ...proposals];
    setProposals(newProposals);
    await Storage.saveProposals(newProposals);

    // 扣除押金
    const newUser = { ...user, balance: user.balance - PROPOSAL_DEPOSIT };
    setUser(newUser);
    await Storage.saveUser(newUser);

    // 记录交易
    addTransaction('vote', -PROPOSAL_DEPOSIT, `创建提案 "${title}"`);

    return {
      success: true,
      message: `✅ 提案创建成功！\n\n押金: ${PROPOSAL_DEPOSIT} COS\n投票期限: 7天`,
    };
  };

  // 记录战斗
  const recordBattle = (petId: number, won: boolean, reward: number) => {
    const pet = pets.find(p => p.id === petId);
    if (!pet) return;

    const updatedPet: Pet = {
      ...pet,
      battleStats: {
        wins: won ? pet.battleStats.wins + 1 : pet.battleStats.wins,
        losses: won ? pet.battleStats.losses : pet.battleStats.losses + 1,
        winRate: Math.round(
          ((won ? pet.battleStats.wins + 1 : pet.battleStats.wins) /
            (pet.battleStats.wins + pet.battleStats.losses + 1)) *
            100
        ),
      },
      experience: pet.experience + (won ? 100 : 30),
    };

    const newPets = pets.map(p => p.id === petId ? updatedPet : p);
    setPets(newPets);
    Storage.savePets(newPets);

    if (won && reward > 0 && user) {
      const newUser = { ...user, balance: user.balance + reward };
      setUser(newUser);
      Storage.saveUser(newUser);
      addTransaction('battle_reward', reward, `战斗胜利奖励`);
    }
  };

  return (
    <MeowstarContext.Provider
      value={{
        user,
        updateBalance,
        pets,
        getPetById,
        levelUpPet,
        evolvePet,
        updatePetStats,
        marketListings,
        buyPet,
        placeBid,
        listPetForSale,
        proposals,
        vote,
        createProposal,
        transactions,
        recordBattle,
        isLoading,
        refreshData,
      }}
    >
      {children}
    </MeowstarContext.Provider>
  );
}

export function useMeowstar() {
  const context = useContext(MeowstarContext);
  if (context === undefined) {
    throw new Error('useMeowstar must be used within a MeowstarProvider');
  }
  return context;
}
