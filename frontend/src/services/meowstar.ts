/**
 * Meowstar Universe - 链上服务
 * 与 pallet-meowstar-pet, pallet-meowstar-battle, pallet-meowstar-staking,
 * pallet-meowstar-governance, pallet-meowstar-marketplace 交互
 */

import { ApiPromise } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import type { ISubmittableResult } from '@polkadot/types/types';

// ============ 类型定义 ============

export interface Pet {
  id: number;
  owner: string;
  name: string;
  element: PetElement;
  rarity: PetRarity;
  level: number;
  experience: number;
  evolutionStage: number;
  hp: number;
  attack: number;
  defense: number;
  speed: number;
  critRate: number;
  createdAt: number;
}

export type PetElement = 'Normal' | 'Fire' | 'Water' | 'Light' | 'Shadow';
export type PetRarity = 'Common' | 'Rare' | 'Epic' | 'Legendary' | 'Mythic';

export interface Battle {
  id: number;
  challenger: string;
  opponent: string;
  challengerPetId: number;
  opponentPetId: number;
  status: BattleStatus;
  winner: string | null;
  createdAt: number;
}

export type BattleStatus = 'Pending' | 'InProgress' | 'Completed' | 'Cancelled';

export interface StakeInfo {
  id: number;
  staker: string;
  amount: string;
  lockPeriod: LockPeriod;
  startBlock: number;
  endBlock: number;
  rewardsClaimed: string;
  isActive: boolean;
}

export type LockPeriod = 'Flexible' | 'Days30' | 'Days90' | 'Days180' | 'Days365';

export interface Proposal {
  id: number;
  proposer: string;
  proposalType: ProposalType;
  titleHash: string;
  descriptionHash: string;
  yesVotes: string;
  noVotes: string;
  status: ProposalStatus;
  createdAt: number;
  votingEndsAt: number;
}

export type ProposalType = 'General' | 'ParameterChange' | 'TreasurySpend' | 'Emergency';
export type ProposalStatus = 'Active' | 'Passed' | 'Rejected' | 'Executed' | 'Cancelled';

export interface Listing {
  id: number;
  seller: string;
  petId: number;
  listingType: ListingType;
  price: string;
  highestBid: string | null;
  highestBidder: string | null;
  createdAt: number;
  expiresAt: number;
  status: ListingStatus;
}

export type ListingType = 'FixedPrice' | 'Auction';
export type ListingStatus = 'Active' | 'Sold' | 'Cancelled' | 'Expired';

// ============ 服务类 ============

export class MeowstarService {
  private api: ApiPromise;
  private keyring: Keyring;

  constructor(api: ApiPromise) {
    this.api = api;
    this.keyring = new Keyring({ type: 'sr25519' });
  }

  // ============ Pet Pallet ============

  /**
   * 孵化新宠物
   */
  async hatchPet(
    signer: string,
    name: string,
    onSuccess?: (petId: number) => void
  ): Promise<string> {
    const tx = this.api.tx.meowstarPet.hatchPet(name);
    return this.signAndSend(tx, signer, (result) => {
      const event = result.events.find(
        ({ event }) => event.section === 'meowstarPet' && event.method === 'PetHatched'
      );
      if (event && onSuccess) {
        const petId = event.event.data[0].toNumber();
        onSuccess(petId);
      }
    });
  }

  /**
   * 升级宠物
   */
  async levelUpPet(signer: string, petId: number): Promise<string> {
    const tx = this.api.tx.meowstarPet.levelUp(petId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 进化宠物
   */
  async evolvePet(
    signer: string,
    petId: number,
    targetElement?: PetElement
  ): Promise<string> {
    const tx = this.api.tx.meowstarPet.evolve(petId, targetElement || null);
    return this.signAndSend(tx, signer);
  }

  /**
   * 转移宠物
   */
  async transferPet(
    signer: string,
    petId: number,
    to: string
  ): Promise<string> {
    const tx = this.api.tx.meowstarPet.transfer(petId, to);
    return this.signAndSend(tx, signer);
  }

  /**
   * 重命名宠物
   */
  async renamePet(
    signer: string,
    petId: number,
    newName: string
  ): Promise<string> {
    const tx = this.api.tx.meowstarPet.rename(petId, newName);
    return this.signAndSend(tx, signer);
  }

  /**
   * 获取宠物信息
   */
  async getPet(petId: number): Promise<Pet | null> {
    const result = await this.api.query.meowstarPet.pets(petId);
    if (result.isNone) return null;
    return this.decodePet(petId, result.unwrap());
  }

  /**
   * 获取用户所有宠物
   */
  async getPetsByOwner(owner: string): Promise<Pet[]> {
    const entries = await this.api.query.meowstarPet.pets.entries();
    const pets: Pet[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      const pet = value.unwrap();
      if (pet.owner.toString() === owner) {
        const petId = key.args[0].toNumber();
        pets.push(this.decodePet(petId, pet));
      }
    }
    
    return pets;
  }

  /**
   * 获取下一个宠物 ID
   */
  async getNextPetId(): Promise<number> {
    const result = await this.api.query.meowstarPet.nextPetId();
    return result.toNumber();
  }

  // ============ Battle Pallet ============

  /**
   * 创建 PVE 战斗
   */
  async createPveBattle(
    signer: string,
    petId: number,
    difficulty: number
  ): Promise<string> {
    const tx = this.api.tx.meowstarBattle.createPveBattle(petId, difficulty);
    return this.signAndSend(tx, signer);
  }

  /**
   * 创建 PVP 挑战
   */
  async createPvpChallenge(
    signer: string,
    petId: number,
    opponent: string,
    opponentPetId: number
  ): Promise<string> {
    const tx = this.api.tx.meowstarBattle.createPvpChallenge(petId, opponent, opponentPetId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 接受 PVP 挑战
   */
  async acceptChallenge(signer: string, battleId: number): Promise<string> {
    const tx = this.api.tx.meowstarBattle.acceptChallenge(battleId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 获取战斗信息
   */
  async getBattle(battleId: number): Promise<Battle | null> {
    const result = await this.api.query.meowstarBattle.battles(battleId);
    if (result.isNone) return null;
    return this.decodeBattle(battleId, result.unwrap());
  }

  /**
   * 获取用户战绩
   */
  async getPlayerStats(player: string): Promise<{ wins: number; losses: number }> {
    const wins = await this.api.query.meowstarBattle.playerWins(player);
    const losses = await this.api.query.meowstarBattle.playerLosses(player);
    return {
      wins: wins.toNumber(),
      losses: losses.toNumber(),
    };
  }

  // ============ Staking Pallet ============

  /**
   * 质押 COS
   */
  async stake(
    signer: string,
    amount: string,
    lockPeriod: LockPeriod
  ): Promise<string> {
    const tx = this.api.tx.meowstarStaking.stake(amount, lockPeriod);
    return this.signAndSend(tx, signer);
  }

  /**
   * 解除质押
   */
  async unstake(signer: string, stakeId: number): Promise<string> {
    const tx = this.api.tx.meowstarStaking.unstake(stakeId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 领取质押收益
   */
  async claimRewards(signer: string, stakeId: number): Promise<string> {
    const tx = this.api.tx.meowstarStaking.claimRewards(stakeId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 获取用户质押信息
   */
  async getStakesByUser(user: string): Promise<StakeInfo[]> {
    const entries = await this.api.query.meowstarStaking.stakes.entries();
    const stakes: StakeInfo[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      const stake = value.unwrap();
      if (stake.staker.toString() === user) {
        const stakeId = key.args[0].toNumber();
        stakes.push(this.decodeStake(stakeId, stake));
      }
    }
    
    return stakes;
  }

  /**
   * 获取用户投票权重
   */
  async getVotePower(user: string): Promise<string> {
    const result = await this.api.query.meowstarStaking.votePower(user);
    return result.toString();
  }

  /**
   * 获取总质押量
   */
  async getTotalStaked(): Promise<string> {
    const result = await this.api.query.meowstarStaking.totalStaked();
    return result.toString();
  }

  // ============ Governance Pallet ============

  /**
   * 创建提案
   */
  async createProposal(
    signer: string,
    proposalType: ProposalType,
    titleHash: string,
    descriptionHash: string
  ): Promise<string> {
    const tx = this.api.tx.meowstarGovernance.createProposal(
      proposalType,
      titleHash,
      descriptionHash
    );
    return this.signAndSend(tx, signer);
  }

  /**
   * 投票
   */
  async vote(
    signer: string,
    proposalId: number,
    approve: boolean
  ): Promise<string> {
    const tx = this.api.tx.meowstarGovernance.vote(proposalId, approve);
    return this.signAndSend(tx, signer);
  }

  /**
   * 获取提案
   */
  async getProposal(proposalId: number): Promise<Proposal | null> {
    const result = await this.api.query.meowstarGovernance.proposals(proposalId);
    if (result.isNone) return null;
    return this.decodeProposal(proposalId, result.unwrap());
  }

  /**
   * 获取所有活跃提案
   */
  async getActiveProposals(): Promise<Proposal[]> {
    const entries = await this.api.query.meowstarGovernance.proposals.entries();
    const proposals: Proposal[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      const proposal = value.unwrap();
      if (proposal.status.toString() === 'Active') {
        const proposalId = key.args[0].toNumber();
        proposals.push(this.decodeProposal(proposalId, proposal));
      }
    }
    
    return proposals;
  }

  // ============ Marketplace Pallet ============

  /**
   * 固定价格挂单
   */
  async listFixedPrice(
    signer: string,
    petId: number,
    price: string,
    duration: number
  ): Promise<string> {
    const tx = this.api.tx.meowstarMarketplace.listFixedPrice(petId, price, duration);
    return this.signAndSend(tx, signer);
  }

  /**
   * 拍卖挂单
   */
  async listAuction(
    signer: string,
    petId: number,
    startingPrice: string,
    duration: number
  ): Promise<string> {
    const tx = this.api.tx.meowstarMarketplace.listAuction(petId, startingPrice, duration);
    return this.signAndSend(tx, signer);
  }

  /**
   * 购买固定价格商品
   */
  async buy(signer: string, listingId: number): Promise<string> {
    const tx = this.api.tx.meowstarMarketplace.buy(listingId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 拍卖出价
   */
  async placeBid(
    signer: string,
    listingId: number,
    amount: string
  ): Promise<string> {
    const tx = this.api.tx.meowstarMarketplace.placeBid(listingId, amount);
    return this.signAndSend(tx, signer);
  }

  /**
   * 取消挂单
   */
  async cancelListing(signer: string, listingId: number): Promise<string> {
    const tx = this.api.tx.meowstarMarketplace.cancelListing(listingId);
    return this.signAndSend(tx, signer);
  }

  /**
   * 获取挂单信息
   */
  async getListing(listingId: number): Promise<Listing | null> {
    const result = await this.api.query.meowstarMarketplace.listings(listingId);
    if (result.isNone) return null;
    return this.decodeListing(listingId, result.unwrap());
  }

  /**
   * 获取所有活跃挂单
   */
  async getActiveListings(): Promise<Listing[]> {
    const entries = await this.api.query.meowstarMarketplace.listings.entries();
    const listings: Listing[] = [];
    
    for (const [key, value] of entries) {
      if (value.isNone) continue;
      const listing = value.unwrap();
      if (listing.status.toString() === 'Active') {
        const listingId = key.args[0].toNumber();
        listings.push(this.decodeListing(listingId, listing));
      }
    }
    
    return listings;
  }

  // ============ 辅助方法 ============

  private async signAndSend(
    tx: any,
    signer: string,
    onSuccess?: (result: ISubmittableResult) => void
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      tx.signAndSend(signer, (result: ISubmittableResult) => {
        if (result.status.isInBlock) {
          console.log(`Transaction included in block: ${result.status.asInBlock}`);
        }
        if (result.status.isFinalized) {
          const success = !result.dispatchError;
          if (success) {
            if (onSuccess) onSuccess(result);
            resolve(result.status.asFinalized.toString());
          } else {
            reject(new Error('Transaction failed'));
          }
        }
      }).catch(reject);
    });
  }

  private decodePet(id: number, data: any): Pet {
    return {
      id,
      owner: data.owner.toString(),
      name: data.name.toHuman(),
      element: data.element.toString() as PetElement,
      rarity: data.rarity.toString() as PetRarity,
      level: data.level.toNumber(),
      experience: data.experience.toNumber(),
      evolutionStage: data.evolutionStage.toNumber(),
      hp: data.hp.toNumber(),
      attack: data.attack.toNumber(),
      defense: data.defense.toNumber(),
      speed: data.speed.toNumber(),
      critRate: data.critRate.toNumber(),
      createdAt: data.createdAt.toNumber(),
    };
  }

  private decodeBattle(id: number, data: any): Battle {
    return {
      id,
      challenger: data.challenger.toString(),
      opponent: data.opponent.toString(),
      challengerPetId: data.challengerPetId.toNumber(),
      opponentPetId: data.opponentPetId.toNumber(),
      status: data.status.toString() as BattleStatus,
      winner: data.winner.isSome ? data.winner.unwrap().toString() : null,
      createdAt: data.createdAt.toNumber(),
    };
  }

  private decodeStake(id: number, data: any): StakeInfo {
    return {
      id,
      staker: data.staker.toString(),
      amount: data.amount.toString(),
      lockPeriod: data.lockPeriod.toString() as LockPeriod,
      startBlock: data.startBlock.toNumber(),
      endBlock: data.endBlock.toNumber(),
      rewardsClaimed: data.rewardsClaimed.toString(),
      isActive: data.isActive.isTrue,
    };
  }

  private decodeProposal(id: number, data: any): Proposal {
    return {
      id,
      proposer: data.proposer.toString(),
      proposalType: data.proposalType.toString() as ProposalType,
      titleHash: data.titleHash.toHex(),
      descriptionHash: data.descriptionHash.toHex(),
      yesVotes: data.yesVotes.toString(),
      noVotes: data.noVotes.toString(),
      status: data.status.toString() as ProposalStatus,
      createdAt: data.createdAt.toNumber(),
      votingEndsAt: data.votingEndsAt.toNumber(),
    };
  }

  private decodeListing(id: number, data: any): Listing {
    return {
      id,
      seller: data.seller.toString(),
      petId: data.petId.toNumber(),
      listingType: data.listingType.toString() as ListingType,
      price: data.price.toString(),
      highestBid: data.highestBid.isSome ? data.highestBid.unwrap().toString() : null,
      highestBidder: data.highestBidder.isSome ? data.highestBidder.unwrap().toString() : null,
      createdAt: data.createdAt.toNumber(),
      expiresAt: data.expiresAt.toNumber(),
      status: data.status.toString() as ListingStatus,
    };
  }
}

// ============ 单例导出 ============

let meowstarService: MeowstarService | null = null;

export function initMeowstarService(api: ApiPromise): MeowstarService {
  meowstarService = new MeowstarService(api);
  return meowstarService;
}

export function getMeowstarService(): MeowstarService {
  if (!meowstarService) {
    throw new Error('MeowstarService not initialized. Call initMeowstarService first.');
  }
  return meowstarService;
}
