/**
 * Meowstar Universe - Zustand Store
 * 全局状态管理
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import type { Pet, StakeInfo, Proposal, Listing } from '../services/meowstar';

interface MeowstarState {
  // 用户宠物
  pets: Pet[];
  selectedPetId: number | null;
  
  // 战斗
  currentBattleId: number | null;
  battleHistory: Array<{
    id: number;
    petId: number;
    opponent: string;
    won: boolean;
    reward: string;
    timestamp: number;
  }>;
  
  // 质押
  stakes: StakeInfo[];
  totalStaked: string;
  votePower: string;
  
  // 治理
  activeProposals: Proposal[];
  votedProposals: number[];
  
  // 市场
  activeListings: Listing[];
  myListings: Listing[];
  
  // UI 状态
  isLoading: boolean;
  error: string | null;
  
  // Actions
  setPets: (pets: Pet[]) => void;
  addPet: (pet: Pet) => void;
  updatePet: (petId: number, updates: Partial<Pet>) => void;
  removePet: (petId: number) => void;
  setSelectedPetId: (petId: number | null) => void;
  
  setCurrentBattleId: (battleId: number | null) => void;
  addBattleHistory: (battle: MeowstarState['battleHistory'][0]) => void;
  
  setStakes: (stakes: StakeInfo[]) => void;
  setTotalStaked: (amount: string) => void;
  setVotePower: (power: string) => void;
  
  setActiveProposals: (proposals: Proposal[]) => void;
  addVotedProposal: (proposalId: number) => void;
  
  setActiveListings: (listings: Listing[]) => void;
  setMyListings: (listings: Listing[]) => void;
  
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  
  reset: () => void;
}

const initialState = {
  pets: [],
  selectedPetId: null,
  currentBattleId: null,
  battleHistory: [],
  stakes: [],
  totalStaked: '0',
  votePower: '0',
  activeProposals: [],
  votedProposals: [],
  activeListings: [],
  myListings: [],
  isLoading: false,
  error: null,
};

export const useMeowstarStore = create<MeowstarState>()(
  persist(
    (set, get) => ({
      ...initialState,

      // Pet actions
      setPets: (pets) => set({ pets }),
      
      addPet: (pet) => set((state) => ({
        pets: [...state.pets, pet],
      })),
      
      updatePet: (petId, updates) => set((state) => ({
        pets: state.pets.map((pet) =>
          pet.id === petId ? { ...pet, ...updates } : pet
        ),
      })),
      
      removePet: (petId) => set((state) => ({
        pets: state.pets.filter((pet) => pet.id !== petId),
        selectedPetId: state.selectedPetId === petId ? null : state.selectedPetId,
      })),
      
      setSelectedPetId: (petId) => set({ selectedPetId: petId }),

      // Battle actions
      setCurrentBattleId: (battleId) => set({ currentBattleId: battleId }),
      
      addBattleHistory: (battle) => set((state) => ({
        battleHistory: [battle, ...state.battleHistory].slice(0, 50), // 保留最近 50 条
      })),

      // Staking actions
      setStakes: (stakes) => set({ stakes }),
      setTotalStaked: (amount) => set({ totalStaked: amount }),
      setVotePower: (power) => set({ votePower: power }),

      // Governance actions
      setActiveProposals: (proposals) => set({ activeProposals: proposals }),
      
      addVotedProposal: (proposalId) => set((state) => ({
        votedProposals: [...state.votedProposals, proposalId],
      })),

      // Marketplace actions
      setActiveListings: (listings) => set({ activeListings: listings }),
      setMyListings: (listings) => set({ myListings: listings }),

      // UI actions
      setLoading: (loading) => set({ isLoading: loading }),
      setError: (error) => set({ error }),

      // Reset
      reset: () => set(initialState),
    }),
    {
      name: 'meowstar-storage',
      storage: createJSONStorage(() => AsyncStorage),
      partialize: (state) => ({
        selectedPetId: state.selectedPetId,
        battleHistory: state.battleHistory,
        votedProposals: state.votedProposals,
      }),
    }
  )
);

// 选择器
export const selectPets = (state: MeowstarState) => state.pets;
export const selectSelectedPet = (state: MeowstarState) =>
  state.pets.find((pet) => pet.id === state.selectedPetId) || null;
export const selectPetById = (petId: number) => (state: MeowstarState) =>
  state.pets.find((pet) => pet.id === petId) || null;
export const selectStakes = (state: MeowstarState) => state.stakes;
export const selectActiveProposals = (state: MeowstarState) => state.activeProposals;
export const selectActiveListings = (state: MeowstarState) => state.activeListings;
export const selectIsLoading = (state: MeowstarState) => state.isLoading;
export const selectError = (state: MeowstarState) => state.error;
