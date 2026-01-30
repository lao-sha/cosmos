/**
 * Meowstar Universe - React Hooks
 * 提供与链上 pallets 交互的 React hooks
 */

import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getMeowstarService,
  Pet,
  Battle,
  StakeInfo,
  Proposal,
  Listing,
  LockPeriod,
  ProposalType,
  PetElement,
} from '../services/meowstar';

// ============ Pet Hooks ============

export function usePets(owner: string | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'pets', owner],
    queryFn: async () => {
      if (!owner) return [];
      const service = getMeowstarService();
      return service.getPetsByOwner(owner);
    },
    enabled: !!owner,
  });
}

export function usePet(petId: number | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'pet', petId],
    queryFn: async () => {
      if (petId === undefined) return null;
      const service = getMeowstarService();
      return service.getPet(petId);
    },
    enabled: petId !== undefined,
  });
}

export function useHatchPet() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, name }: { signer: string; name: string }) => {
      const service = getMeowstarService();
      let newPetId: number | undefined;
      await service.hatchPet(signer, name, (petId) => {
        newPetId = petId;
      });
      return newPetId;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

export function useLevelUpPet() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, petId }: { signer: string; petId: number }) => {
      const service = getMeowstarService();
      return service.levelUpPet(signer, petId);
    },
    onSuccess: (_, { petId }) => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pet', petId] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

export function useEvolvePet() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      petId,
      targetElement,
    }: {
      signer: string;
      petId: number;
      targetElement?: PetElement;
    }) => {
      const service = getMeowstarService();
      return service.evolvePet(signer, petId, targetElement);
    },
    onSuccess: (_, { petId }) => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pet', petId] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

export function useTransferPet() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      petId,
      to,
    }: {
      signer: string;
      petId: number;
      to: string;
    }) => {
      const service = getMeowstarService();
      return service.transferPet(signer, petId, to);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

// ============ Battle Hooks ============

export function useBattle(battleId: number | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'battle', battleId],
    queryFn: async () => {
      if (battleId === undefined) return null;
      const service = getMeowstarService();
      return service.getBattle(battleId);
    },
    enabled: battleId !== undefined,
  });
}

export function usePlayerStats(player: string | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'playerStats', player],
    queryFn: async () => {
      if (!player) return { wins: 0, losses: 0 };
      const service = getMeowstarService();
      return service.getPlayerStats(player);
    },
    enabled: !!player,
  });
}

export function useCreatePveBattle() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      petId,
      difficulty,
    }: {
      signer: string;
      petId: number;
      difficulty: number;
    }) => {
      const service = getMeowstarService();
      return service.createPveBattle(signer, petId, difficulty);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'playerStats'] });
    },
  });
}

export function useCreatePvpChallenge() {
  return useMutation({
    mutationFn: async ({
      signer,
      petId,
      opponent,
      opponentPetId,
    }: {
      signer: string;
      petId: number;
      opponent: string;
      opponentPetId: number;
    }) => {
      const service = getMeowstarService();
      return service.createPvpChallenge(signer, petId, opponent, opponentPetId);
    },
  });
}

export function useAcceptChallenge() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, battleId }: { signer: string; battleId: number }) => {
      const service = getMeowstarService();
      return service.acceptChallenge(signer, battleId);
    },
    onSuccess: (_, { battleId }) => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'battle', battleId] });
    },
  });
}

// ============ Staking Hooks ============

export function useStakes(user: string | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'stakes', user],
    queryFn: async () => {
      if (!user) return [];
      const service = getMeowstarService();
      return service.getStakesByUser(user);
    },
    enabled: !!user,
  });
}

export function useVotePower(user: string | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'votePower', user],
    queryFn: async () => {
      if (!user) return '0';
      const service = getMeowstarService();
      return service.getVotePower(user);
    },
    enabled: !!user,
  });
}

export function useTotalStaked() {
  return useQuery({
    queryKey: ['meowstar', 'totalStaked'],
    queryFn: async () => {
      const service = getMeowstarService();
      return service.getTotalStaked();
    },
  });
}

export function useStake() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      amount,
      lockPeriod,
    }: {
      signer: string;
      amount: string;
      lockPeriod: LockPeriod;
    }) => {
      const service = getMeowstarService();
      return service.stake(signer, amount, lockPeriod);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'stakes'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'votePower'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'totalStaked'] });
    },
  });
}

export function useUnstake() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, stakeId }: { signer: string; stakeId: number }) => {
      const service = getMeowstarService();
      return service.unstake(signer, stakeId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'stakes'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'votePower'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'totalStaked'] });
    },
  });
}

export function useClaimRewards() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, stakeId }: { signer: string; stakeId: number }) => {
      const service = getMeowstarService();
      return service.claimRewards(signer, stakeId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'stakes'] });
    },
  });
}

// ============ Governance Hooks ============

export function useProposal(proposalId: number | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'proposal', proposalId],
    queryFn: async () => {
      if (proposalId === undefined) return null;
      const service = getMeowstarService();
      return service.getProposal(proposalId);
    },
    enabled: proposalId !== undefined,
  });
}

export function useActiveProposals() {
  return useQuery({
    queryKey: ['meowstar', 'activeProposals'],
    queryFn: async () => {
      const service = getMeowstarService();
      return service.getActiveProposals();
    },
  });
}

export function useCreateProposal() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      proposalType,
      titleHash,
      descriptionHash,
    }: {
      signer: string;
      proposalType: ProposalType;
      titleHash: string;
      descriptionHash: string;
    }) => {
      const service = getMeowstarService();
      return service.createProposal(signer, proposalType, titleHash, descriptionHash);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeProposals'] });
    },
  });
}

export function useVote() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      proposalId,
      approve,
    }: {
      signer: string;
      proposalId: number;
      approve: boolean;
    }) => {
      const service = getMeowstarService();
      return service.vote(signer, proposalId, approve);
    },
    onSuccess: (_, { proposalId }) => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'proposal', proposalId] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeProposals'] });
    },
  });
}

// ============ Marketplace Hooks ============

export function useListing(listingId: number | undefined) {
  return useQuery({
    queryKey: ['meowstar', 'listing', listingId],
    queryFn: async () => {
      if (listingId === undefined) return null;
      const service = getMeowstarService();
      return service.getListing(listingId);
    },
    enabled: listingId !== undefined,
  });
}

export function useActiveListings() {
  return useQuery({
    queryKey: ['meowstar', 'activeListings'],
    queryFn: async () => {
      const service = getMeowstarService();
      return service.getActiveListings();
    },
  });
}

export function useListFixedPrice() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      petId,
      price,
      duration,
    }: {
      signer: string;
      petId: number;
      price: string;
      duration: number;
    }) => {
      const service = getMeowstarService();
      return service.listFixedPrice(signer, petId, price, duration);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeListings'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

export function useListAuction() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      petId,
      startingPrice,
      duration,
    }: {
      signer: string;
      petId: number;
      startingPrice: string;
      duration: number;
    }) => {
      const service = getMeowstarService();
      return service.listAuction(signer, petId, startingPrice, duration);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeListings'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

export function useBuy() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, listingId }: { signer: string; listingId: number }) => {
      const service = getMeowstarService();
      return service.buy(signer, listingId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeListings'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}

export function usePlaceBid() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({
      signer,
      listingId,
      amount,
    }: {
      signer: string;
      listingId: number;
      amount: string;
    }) => {
      const service = getMeowstarService();
      return service.placeBid(signer, listingId, amount);
    },
    onSuccess: (_, { listingId }) => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'listing', listingId] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeListings'] });
    },
  });
}

export function useCancelListing() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ signer, listingId }: { signer: string; listingId: number }) => {
      const service = getMeowstarService();
      return service.cancelListing(signer, listingId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'activeListings'] });
      queryClient.invalidateQueries({ queryKey: ['meowstar', 'pets'] });
    },
  });
}
