// 喵星宇宙本地存储服务
import AsyncStorage from '@react-native-async-storage/async-storage';
import { Pet, User, MarketListing, Proposal, Transaction } from './types';

const STORAGE_KEYS = {
  USER: 'meowstar_user',
  PETS: 'meowstar_pets',
  MARKET: 'meowstar_market',
  PROPOSALS: 'meowstar_proposals',
  TRANSACTIONS: 'meowstar_transactions',
};

// 用户数据
export const saveUser = async (user: User): Promise<void> => {
  await AsyncStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user));
};

export const loadUser = async (): Promise<User | null> => {
  const data = await AsyncStorage.getItem(STORAGE_KEYS.USER);
  return data ? JSON.parse(data) : null;
};

// 宠物数据
export const savePets = async (pets: Pet[]): Promise<void> => {
  await AsyncStorage.setItem(STORAGE_KEYS.PETS, JSON.stringify(pets));
};

export const loadPets = async (): Promise<Pet[]> => {
  const data = await AsyncStorage.getItem(STORAGE_KEYS.PETS);
  return data ? JSON.parse(data) : [];
};

// 市场数据
export const saveMarket = async (listings: MarketListing[]): Promise<void> => {
  await AsyncStorage.setItem(STORAGE_KEYS.MARKET, JSON.stringify(listings));
};

export const loadMarket = async (): Promise<MarketListing[]> => {
  const data = await AsyncStorage.getItem(STORAGE_KEYS.MARKET);
  return data ? JSON.parse(data) : [];
};

// 提案数据
export const saveProposals = async (proposals: Proposal[]): Promise<void> => {
  await AsyncStorage.setItem(STORAGE_KEYS.PROPOSALS, JSON.stringify(proposals));
};

export const loadProposals = async (): Promise<Proposal[]> => {
  const data = await AsyncStorage.getItem(STORAGE_KEYS.PROPOSALS);
  return data ? JSON.parse(data) : [];
};

// 交易记录
export const saveTransactions = async (transactions: Transaction[]): Promise<void> => {
  await AsyncStorage.setItem(STORAGE_KEYS.TRANSACTIONS, JSON.stringify(transactions));
};

export const loadTransactions = async (): Promise<Transaction[]> => {
  const data = await AsyncStorage.getItem(STORAGE_KEYS.TRANSACTIONS);
  return data ? JSON.parse(data) : [];
};

// 清除所有数据
export const clearAllData = async (): Promise<void> => {
  await AsyncStorage.multiRemove(Object.values(STORAGE_KEYS));
};
