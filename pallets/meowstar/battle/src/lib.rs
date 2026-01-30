//! # Meowstar Battle Pallet
//!
//! 喵星宇宙战斗系统 Pallet
//!
//! ## 功能
//! - 回合制战斗引擎
//! - PVE/PVP 战斗
//! - ELO 匹配系统
//! - 技能系统
//! - 战斗奖励

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, Randomness},
    };
    use frame_system::pallet_prelude::*;
    use pallet_meowstar_pet::{Element, Pet, Rarity};
    use sp_runtime::traits::Saturating;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 战斗状态
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum BattleStatus {
        /// 等待开始
        #[default]
        Pending,
        /// 进行中
        InProgress,
        /// 已完成
        Completed,
        /// 已取消
        Cancelled,
    }

    /// 战斗类型
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum BattleType {
        /// PVE - 对战 AI
        #[default]
        PVE,
        /// PVP - 对战玩家
        PVP,
        /// 排位赛
        Ranked,
        /// 锦标赛
        Tournament,
    }

    /// 行动类型
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, codec::DecodeWithMemTracking)]
    pub enum ActionType {
        /// 普通攻击
        Attack,
        /// 使用技能
        Skill(u8),
        /// 防御
        Defend,
        /// 切换宠物
        Switch(u64),
    }

    /// 战斗行动
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct BattleAction<T: Config> {
        /// 行动者
        pub actor: T::AccountId,
        /// 行动类型
        pub action_type: ActionType,
        /// 回合数
        pub turn: u32,
        /// 提交区块
        pub submitted_at: BlockNumberFor<T>,
    }

    /// 战斗者状态
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub struct FighterState {
        /// 当前 HP
        pub current_hp: u32,
        /// 最大 HP
        pub max_hp: u32,
        /// 攻击力
        pub attack: u32,
        /// 防御力
        pub defense: u32,
        /// 速度
        pub speed: u32,
        /// 暴击率
        pub critical_rate: u32,
        /// 暴击伤害
        pub critical_damage: u32,
        /// 是否防御中
        pub is_defending: bool,
        /// Buff/Debuff 状态
        pub buffs: u32,
    }

    impl FighterState {
        /// 从宠物属性创建
        pub fn from_pet_attributes(attrs: &pallet_meowstar_pet::PetAttributes) -> Self {
            Self {
                current_hp: attrs.health,
                max_hp: attrs.health,
                attack: attrs.attack,
                defense: attrs.defense,
                speed: attrs.speed,
                critical_rate: attrs.critical_rate,
                critical_damage: attrs.critical_damage,
                is_defending: false,
                buffs: 0,
            }
        }

        /// 是否存活
        pub fn is_alive(&self) -> bool {
            self.current_hp > 0
        }

        /// 受到伤害
        pub fn take_damage(&mut self, damage: u32) {
            self.current_hp = self.current_hp.saturating_sub(damage);
        }

        /// 恢复 HP
        pub fn heal(&mut self, amount: u32) {
            self.current_hp = self.current_hp.saturating_add(amount).min(self.max_hp);
        }
    }

    /// 战斗信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Battle<T: Config> {
        /// 战斗ID
        pub id: u64,
        /// 战斗类型
        pub battle_type: BattleType,
        /// 玩家1
        pub player1: T::AccountId,
        /// 玩家2 (PVE 时为 None)
        pub player2: Option<T::AccountId>,
        /// 玩家1 的宠物ID
        pub pet1_id: u64,
        /// 玩家2 的宠物ID (PVE 时为虚拟宠物)
        pub pet2_id: u64,
        /// 玩家1 状态
        pub fighter1: FighterState,
        /// 玩家2 状态
        pub fighter2: FighterState,
        /// 当前回合
        pub current_turn: u32,
        /// 最大回合数
        pub max_turns: u32,
        /// 战斗状态
        pub status: BattleStatus,
        /// 胜利者 (None 表示未结束或平局)
        pub winner: Option<T::AccountId>,
        /// 创建区块
        pub created_at: BlockNumberFor<T>,
        /// 结束区块
        pub ended_at: Option<BlockNumberFor<T>>,
        /// 随机种子
        pub random_seed: [u8; 32],
    }

    /// ELO 评分
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub struct PlayerRating {
        /// ELO 分数
        pub elo: u32,
        /// 总场次
        pub total_matches: u32,
        /// 胜场
        pub wins: u32,
        /// 败场
        pub losses: u32,
        /// 连胜
        pub win_streak: u32,
        /// 最高连胜
        pub max_win_streak: u32,
        /// 赛季积分
        pub season_points: u32,
    }

    impl PlayerRating {
        pub fn new() -> Self {
            Self {
                elo: 1000,
                total_matches: 0,
                wins: 0,
                losses: 0,
                win_streak: 0,
                max_win_streak: 0,
                season_points: 0,
            }
        }

        /// 更新 ELO (使用整数近似)
        pub fn update_elo(&mut self, opponent_elo: u32, won: bool) {
            let k = 32u32; // K 因子
            
            // 简化 ELO 计算，使用整数运算
            // expected ≈ 0.5 + (self.elo - opponent_elo) / 800
            // 使用 1000 作为基准表示 1.0
            let elo_diff = self.elo as i32 - opponent_elo as i32;
            let expected_1000 = 500i32 + elo_diff * 1000 / 800;
            let expected_1000 = expected_1000.max(100).min(900); // 限制在 10%-90%
            
            let actual_1000 = if won { 1000i32 } else { 0i32 };
            let change = k as i32 * (actual_1000 - expected_1000) / 1000;

            if change > 0 {
                self.elo = self.elo.saturating_add(change as u32);
            } else {
                self.elo = self.elo.saturating_sub((-change) as u32);
            }

            self.total_matches = self.total_matches.saturating_add(1);
            if won {
                self.wins = self.wins.saturating_add(1);
                self.win_streak = self.win_streak.saturating_add(1);
                self.max_win_streak = self.max_win_streak.max(self.win_streak);
                self.season_points = self.season_points.saturating_add(10 + self.win_streak.min(10));
            } else {
                self.losses = self.losses.saturating_add(1);
                self.win_streak = 0;
            }
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_meowstar_pet::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId>;

        /// 随机数生成
        type BattleRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 战斗入场费
        #[pallet::constant]
        type BattleEntryFee: Get<BalanceOf<Self>>;

        /// 最大回合数
        #[pallet::constant]
        type MaxTurns: Get<u32>;

        /// 战斗超时区块数
        #[pallet::constant]
        type BattleTimeout: Get<BlockNumberFor<Self>>;

        /// 权重信息
        type BattleWeightInfo: WeightInfo;
    }

    /// 战斗存储
    #[pallet::storage]
    #[pallet::getter(fn battles)]
    pub type Battles<T: Config> = StorageMap<_, Blake2_128Concat, u64, Battle<T>, OptionQuery>;

    /// 玩家评分
    #[pallet::storage]
    #[pallet::getter(fn player_ratings)]
    pub type PlayerRatings<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, PlayerRating, ValueQuery>;

    /// 匹配队列
    #[pallet::storage]
    #[pallet::getter(fn match_queue)]
    pub type MatchQueue<T: Config> =
        StorageValue<_, BoundedVec<(T::AccountId, u64, u32), ConstU32<100>>, ValueQuery>;

    /// 下一个战斗ID
    #[pallet::storage]
    #[pallet::getter(fn next_battle_id)]
    pub type NextBattleId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 玩家当前战斗
    #[pallet::storage]
    #[pallet::getter(fn player_battles)]
    pub type PlayerBattles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 加入匹配队列
        JoinedQueue {
            player: T::AccountId,
            pet_id: u64,
        },
        /// 战斗开始
        BattleStarted {
            battle_id: u64,
            player1: T::AccountId,
            player2: Option<T::AccountId>,
            battle_type: BattleType,
        },
        /// 行动提交
        ActionSubmitted {
            battle_id: u64,
            player: T::AccountId,
            turn: u32,
        },
        /// 回合结算
        TurnResolved {
            battle_id: u64,
            turn: u32,
            damage1: u32,
            damage2: u32,
        },
        /// 战斗结束
        BattleEnded {
            battle_id: u64,
            winner: Option<T::AccountId>,
            reward: BalanceOf<T>,
        },
        /// ELO 更新
        RatingUpdated {
            player: T::AccountId,
            new_elo: u32,
            change: i32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 战斗不存在
        BattleNotFound,
        /// 不是战斗参与者
        NotParticipant,
        /// 战斗已结束
        BattleAlreadyEnded,
        /// 已在战斗中
        AlreadyInBattle,
        /// 已在队列中
        AlreadyInQueue,
        /// 宠物不存在
        PetNotFound,
        /// 不是宠物所有者
        NotPetOwner,
        /// 余额不足
        InsufficientBalance,
        /// 不是你的回合
        NotYourTurn,
        /// 无效的行动
        InvalidAction,
        /// 队列已满
        QueueFull,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 开始 PVE 战斗
        #[pallet::call_index(0)]
        #[pallet::weight(T::BattleWeightInfo::start_pve_battle())]
        pub fn start_pve_battle(
            origin: OriginFor<T>,
            pet_id: u64,
            difficulty: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查是否已在战斗中
            ensure!(
                PlayerBattles::<T>::get(&who).is_none(),
                Error::<T>::AlreadyInBattle
            );

            // 检查宠物
            let pet = pallet_meowstar_pet::Pallet::<T>::get_pet(pet_id)
                .ok_or(Error::<T>::PetNotFound)?;
            ensure!(pet.owner == who, Error::<T>::NotPetOwner);

            // 生成随机种子
            let (random_hash, _) = T::BattleRandomness::random(&b"battle_seed"[..]);
            let random_seed: [u8; 32] = random_hash.as_ref().try_into().unwrap_or([0u8; 32]);

            // 创建玩家状态
            let fighter1 = FighterState::from_pet_attributes(&pet.attributes);

            // 创建 AI 对手状态 (根据难度)
            let ai_multiplier = 80 + (difficulty as u32 * 10);
            let fighter2 = FighterState {
                current_hp: pet.attributes.health * ai_multiplier / 100,
                max_hp: pet.attributes.health * ai_multiplier / 100,
                attack: pet.attributes.attack * ai_multiplier / 100,
                defense: pet.attributes.defense * ai_multiplier / 100,
                speed: pet.attributes.speed * ai_multiplier / 100,
                critical_rate: 500,
                critical_damage: 15000,
                is_defending: false,
                buffs: 0,
            };

            // 创建战斗
            let battle_id = NextBattleId::<T>::get();
            let battle = Battle {
                id: battle_id,
                battle_type: BattleType::PVE,
                player1: who.clone(),
                player2: None,
                pet1_id: pet_id,
                pet2_id: 0, // AI 宠物
                fighter1,
                fighter2,
                current_turn: 1,
                max_turns: T::MaxTurns::get(),
                status: BattleStatus::InProgress,
                winner: None,
                created_at: frame_system::Pallet::<T>::block_number(),
                ended_at: None,
                random_seed,
            };

            Battles::<T>::insert(battle_id, battle);
            PlayerBattles::<T>::insert(&who, battle_id);
            NextBattleId::<T>::put(battle_id.saturating_add(1));

            Self::deposit_event(Event::BattleStarted {
                battle_id,
                player1: who,
                player2: None,
                battle_type: BattleType::PVE,
            });

            Ok(())
        }

        /// 加入 PVP 匹配队列
        #[pallet::call_index(1)]
        #[pallet::weight(T::BattleWeightInfo::join_pvp_queue())]
        pub fn join_pvp_queue(origin: OriginFor<T>, pet_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查是否已在战斗中
            ensure!(
                PlayerBattles::<T>::get(&who).is_none(),
                Error::<T>::AlreadyInBattle
            );

            // 检查宠物
            let pet = pallet_meowstar_pet::Pallet::<T>::get_pet(pet_id)
                .ok_or(Error::<T>::PetNotFound)?;
            ensure!(pet.owner == who, Error::<T>::NotPetOwner);

            // 获取玩家 ELO
            let rating = PlayerRatings::<T>::get(&who);

            // 加入队列
            MatchQueue::<T>::try_mutate(|queue| -> DispatchResult {
                // 检查是否已在队列中
                ensure!(
                    !queue.iter().any(|(p, _, _)| p == &who),
                    Error::<T>::AlreadyInQueue
                );

                queue
                    .try_push((who.clone(), pet_id, rating.elo))
                    .map_err(|_| Error::<T>::QueueFull)?;

                Ok(())
            })?;

            Self::deposit_event(Event::JoinedQueue {
                player: who,
                pet_id,
            });

            // 尝试匹配
            Self::try_match()?;

            Ok(())
        }

        /// 提交战斗行动
        #[pallet::call_index(2)]
        #[pallet::weight(T::BattleWeightInfo::submit_action())]
        pub fn submit_action(
            origin: OriginFor<T>,
            battle_id: u64,
            action: ActionType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Battles::<T>::try_mutate(battle_id, |maybe_battle| -> DispatchResult {
                let battle = maybe_battle.as_mut().ok_or(Error::<T>::BattleNotFound)?;

                // 检查战斗状态
                ensure!(
                    battle.status == BattleStatus::InProgress,
                    Error::<T>::BattleAlreadyEnded
                );

                // 检查是否是参与者
                let is_player1 = battle.player1 == who;
                let is_player2 = battle.player2.as_ref() == Some(&who);
                ensure!(is_player1 || is_player2, Error::<T>::NotParticipant);

                // PVE 模式直接结算
                if battle.battle_type == BattleType::PVE {
                    Self::resolve_pve_turn(battle, &action)?;
                }

                Self::deposit_event(Event::ActionSubmitted {
                    battle_id,
                    player: who,
                    turn: battle.current_turn,
                });

                // 检查战斗是否结束
                if !battle.fighter1.is_alive() || !battle.fighter2.is_alive() || battle.current_turn > battle.max_turns {
                    Self::end_battle(battle)?;
                }

                Ok(())
            })
        }

        /// 离开匹配队列
        #[pallet::call_index(3)]
        #[pallet::weight(T::BattleWeightInfo::leave_queue())]
        pub fn leave_queue(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            MatchQueue::<T>::mutate(|queue| {
                queue.retain(|(p, _, _)| p != &who);
            });

            Ok(())
        }

        /// 投降
        #[pallet::call_index(4)]
        #[pallet::weight(T::BattleWeightInfo::surrender())]
        pub fn surrender(origin: OriginFor<T>, battle_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Battles::<T>::try_mutate(battle_id, |maybe_battle| -> DispatchResult {
                let battle = maybe_battle.as_mut().ok_or(Error::<T>::BattleNotFound)?;

                ensure!(
                    battle.status == BattleStatus::InProgress,
                    Error::<T>::BattleAlreadyEnded
                );

                let is_player1 = battle.player1 == who;
                let is_player2 = battle.player2.as_ref() == Some(&who);
                ensure!(is_player1 || is_player2, Error::<T>::NotParticipant);

                // 设置对方为胜利者
                if is_player1 {
                    battle.winner = battle.player2.clone();
                } else {
                    battle.winner = Some(battle.player1.clone());
                }

                Self::end_battle(battle)?;

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// 尝试匹配玩家
        fn try_match() -> DispatchResult {
            MatchQueue::<T>::mutate(|queue| {
                if queue.len() < 2 {
                    return;
                }

                // 简单匹配：取前两个玩家
                // TODO: 实现 ELO 范围匹配
                let (player1, pet1_id, _elo1) = queue.remove(0);
                let (player2, pet2_id, _elo2) = queue.remove(0);

                // 获取宠物信息
                let pet1 = pallet_meowstar_pet::Pallet::<T>::get_pet(pet1_id);
                let pet2 = pallet_meowstar_pet::Pallet::<T>::get_pet(pet2_id);

                if let (Some(p1), Some(p2)) = (pet1, pet2) {
                    let fighter1 = FighterState::from_pet_attributes(&p1.attributes);
                    let fighter2 = FighterState::from_pet_attributes(&p2.attributes);

                    let (random_hash, _) = T::BattleRandomness::random(&b"pvp_battle"[..]);
                    let random_seed: [u8; 32] = random_hash.as_ref().try_into().unwrap_or([0u8; 32]);

                    let battle_id = NextBattleId::<T>::get();
                    let battle = Battle {
                        id: battle_id,
                        battle_type: BattleType::PVP,
                        player1: player1.clone(),
                        player2: Some(player2.clone()),
                        pet1_id,
                        pet2_id,
                        fighter1,
                        fighter2,
                        current_turn: 1,
                        max_turns: T::MaxTurns::get(),
                        status: BattleStatus::InProgress,
                        winner: None,
                        created_at: frame_system::Pallet::<T>::block_number(),
                        ended_at: None,
                        random_seed,
                    };

                    Battles::<T>::insert(battle_id, battle);
                    PlayerBattles::<T>::insert(&player1, battle_id);
                    PlayerBattles::<T>::insert(&player2, battle_id);
                    NextBattleId::<T>::put(battle_id.saturating_add(1));

                    Self::deposit_event(Event::BattleStarted {
                        battle_id,
                        player1,
                        player2: Some(player2),
                        battle_type: BattleType::PVP,
                    });
                }
            });

            Ok(())
        }

        /// 结算 PVE 回合
        fn resolve_pve_turn(battle: &mut Battle<T>, action: &ActionType) -> DispatchResult {
            // 计算玩家伤害
            let player_damage = Self::calculate_damage(
                &battle.fighter1,
                &battle.fighter2,
                action,
                &battle.random_seed,
                battle.current_turn,
            );

            // AI 行动 (简单 AI：总是攻击)
            let ai_damage = Self::calculate_damage(
                &battle.fighter2,
                &battle.fighter1,
                &ActionType::Attack,
                &battle.random_seed,
                battle.current_turn + 1000,
            );

            // 根据速度决定先后手
            if battle.fighter1.speed >= battle.fighter2.speed {
                battle.fighter2.take_damage(player_damage);
                if battle.fighter2.is_alive() {
                    battle.fighter1.take_damage(ai_damage);
                }
            } else {
                battle.fighter1.take_damage(ai_damage);
                if battle.fighter1.is_alive() {
                    battle.fighter2.take_damage(player_damage);
                }
            }

            Self::deposit_event(Event::TurnResolved {
                battle_id: battle.id,
                turn: battle.current_turn,
                damage1: player_damage,
                damage2: ai_damage,
            });

            battle.current_turn = battle.current_turn.saturating_add(1);

            Ok(())
        }

        /// 计算伤害
        fn calculate_damage(
            attacker: &FighterState,
            defender: &FighterState,
            action: &ActionType,
            seed: &[u8; 32],
            turn: u32,
        ) -> u32 {
            let base_damage = match action {
                ActionType::Attack => attacker.attack,
                ActionType::Skill(skill_id) => {
                    // 技能伤害倍率
                    let multiplier = match skill_id {
                        0 => 120,
                        1 => 150,
                        2 => 200,
                        _ => 100,
                    };
                    attacker.attack * multiplier / 100
                }
                ActionType::Defend => {
                    return 0;
                }
                ActionType::Switch(_) => {
                    return 0;
                }
            };

            // 防御减免
            let defense_reduction = if defender.is_defending {
                defender.defense * 2
            } else {
                defender.defense
            };

            let damage = base_damage.saturating_sub(defense_reduction / 2);

            // 暴击判定
            let crit_roll = (seed[turn as usize % 32] as u32 * 100) % 10000;
            let final_damage = if crit_roll < attacker.critical_rate {
                damage * attacker.critical_damage / 10000
            } else {
                damage
            };

            final_damage.max(1)
        }

        /// 结束战斗
        fn end_battle(battle: &mut Battle<T>) -> DispatchResult {
            battle.status = BattleStatus::Completed;
            battle.ended_at = Some(frame_system::Pallet::<T>::block_number());

            // 确定胜利者
            if battle.winner.is_none() {
                if battle.fighter1.is_alive() && !battle.fighter2.is_alive() {
                    battle.winner = Some(battle.player1.clone());
                } else if !battle.fighter1.is_alive() && battle.fighter2.is_alive() {
                    battle.winner = battle.player2.clone();
                }
            }

            // 清理玩家战斗状态
            PlayerBattles::<T>::remove(&battle.player1);
            if let Some(ref p2) = battle.player2 {
                PlayerBattles::<T>::remove(p2);
            }

            // 更新 ELO (仅 PVP)
            if battle.battle_type == BattleType::PVP || battle.battle_type == BattleType::Ranked {
                if let Some(ref player2) = battle.player2 {
                    let rating1 = PlayerRatings::<T>::get(&battle.player1);
                    let rating2 = PlayerRatings::<T>::get(player2);

                    let player1_won = battle.winner.as_ref() == Some(&battle.player1);

                    PlayerRatings::<T>::mutate(&battle.player1, |r| {
                        r.update_elo(rating2.elo, player1_won);
                    });
                    PlayerRatings::<T>::mutate(player2, |r| {
                        r.update_elo(rating1.elo, !player1_won);
                    });
                }
            }

            // 发放奖励
            let reward = T::BattleEntryFee::get();
            Self::deposit_event(Event::BattleEnded {
                battle_id: battle.id,
                winner: battle.winner.clone(),
                reward,
            });

            // 增加宠物经验
            let exp_gain = 100u64.saturating_add(battle.current_turn as u64 * 10);
            let _ = pallet_meowstar_pet::Pallet::<T>::add_experience(battle.pet1_id, exp_gain);

            Ok(())
        }
    }
}
