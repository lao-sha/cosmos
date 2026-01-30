//! # Meowstar Pet Pallet
//!
//! 喵星宇宙宠物系统 Pallet
//!
//! ## 功能
//! - 宠物铸造（孵化）
//! - 属性系统（HP/ATK/DEF/SPD/CRIT）
//! - 稀有度系统（Common-Mythic）
//! - 升级系统
//! - 进化系统

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
        traits::{Currency, Randomness, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Hash, Saturating};
    use sp_std::vec::Vec;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 宠物稀有度
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum Rarity {
        #[default]
        Common,
        Rare,
        Epic,
        Legendary,
        Mythic,
    }

    /// 宠物元素类型
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default, codec::DecodeWithMemTracking)]
    pub enum Element {
        #[default]
        Normal,
        Fire,
        Water,
        Shadow,
        Light,
    }

    /// 宠物属性
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct PetAttributes {
        /// 生命值
        pub health: u32,
        /// 攻击力
        pub attack: u32,
        /// 防御力
        pub defense: u32,
        /// 速度
        pub speed: u32,
        /// 暴击率 (百分比 * 100, 如 500 = 5%)
        pub critical_rate: u32,
        /// 暴击伤害 (百分比 * 100, 如 15000 = 150%)
        pub critical_damage: u32,
    }

    impl PetAttributes {
        /// 计算总属性点
        pub fn total(&self) -> u32 {
            self.health
                .saturating_add(self.attack)
                .saturating_add(self.defense)
                .saturating_add(self.speed)
        }

        /// 根据稀有度生成基础属性
        pub fn generate_base(rarity: &Rarity, seed: u32) -> Self {
            let multiplier = match rarity {
                Rarity::Common => 100,
                Rarity::Rare => 120,
                Rarity::Epic => 150,
                Rarity::Legendary => 200,
                Rarity::Mythic => 300,
            };

            let base_health = 100 + (seed % 50);
            let base_attack = 20 + (seed % 20);
            let base_defense = 15 + (seed % 15);
            let base_speed = 10 + (seed % 10);

            Self {
                health: base_health * multiplier / 100,
                attack: base_attack * multiplier / 100,
                defense: base_defense * multiplier / 100,
                speed: base_speed * multiplier / 100,
                critical_rate: 500,  // 5%
                critical_damage: 15000,  // 150%
            }
        }

        /// 升级时增加属性
        pub fn level_up(&mut self, rarity: &Rarity) {
            let growth = match rarity {
                Rarity::Common => 5,
                Rarity::Rare => 6,
                Rarity::Epic => 8,
                Rarity::Legendary => 10,
                Rarity::Mythic => 15,
            };

            self.health = self.health.saturating_add(growth * 2);
            self.attack = self.attack.saturating_add(growth);
            self.defense = self.defense.saturating_add(growth);
            self.speed = self.speed.saturating_add(growth / 2);
        }
    }

    /// 宠物信息
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Pet<T: Config> {
        /// 宠物ID
        pub id: u64,
        /// 所有者
        pub owner: T::AccountId,
        /// 名称
        pub name: BoundedVec<u8, ConstU32<32>>,
        /// 元素类型
        pub element: Element,
        /// 稀有度
        pub rarity: Rarity,
        /// 等级
        pub level: u32,
        /// 当前经验值
        pub experience: u64,
        /// 属性
        pub attributes: PetAttributes,
        /// 进化阶段 (0-4)
        pub evolution_stage: u8,
        /// 创建区块
        pub created_at: BlockNumberFor<T>,
        /// 基因哈希 (用于繁殖)
        pub gene_hash: [u8; 32],
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 随机数生成
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 每个账户最大宠物数量
        #[pallet::constant]
        type MaxPetsPerAccount: Get<u32>;

        /// 孵化费用
        #[pallet::constant]
        type HatchingFee: Get<BalanceOf<Self>>;

        /// 升级基础费用
        #[pallet::constant]
        type LevelUpBaseFee: Get<BalanceOf<Self>>;

        /// 进化费用倍数
        #[pallet::constant]
        type EvolutionFeeMultiplier: Get<u32>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    /// 宠物存储
    #[pallet::storage]
    #[pallet::getter(fn pets)]
    pub type Pets<T: Config> = StorageMap<_, Blake2_128Concat, u64, Pet<T>, OptionQuery>;

    /// 账户拥有的宠物列表
    #[pallet::storage]
    #[pallet::getter(fn pet_owners)]
    pub type PetOwners<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxPetsPerAccount>,
        ValueQuery,
    >;

    /// 下一个宠物ID
    #[pallet::storage]
    #[pallet::getter(fn next_pet_id)]
    pub type NextPetId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 宠物总数
    #[pallet::storage]
    #[pallet::getter(fn total_pets)]
    pub type TotalPets<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 宠物已创建
        PetCreated {
            pet_id: u64,
            owner: T::AccountId,
            rarity: Rarity,
            element: Element,
        },
        /// 宠物已升级
        PetLeveledUp {
            pet_id: u64,
            new_level: u32,
        },
        /// 宠物已进化
        PetEvolved {
            pet_id: u64,
            new_stage: u8,
            new_element: Element,
        },
        /// 宠物已转移
        PetTransferred {
            pet_id: u64,
            from: T::AccountId,
            to: T::AccountId,
        },
        /// 宠物已重命名
        PetRenamed {
            pet_id: u64,
            new_name: BoundedVec<u8, ConstU32<32>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 宠物不存在
        PetNotFound,
        /// 不是宠物所有者
        NotOwner,
        /// 达到最大宠物数量
        MaxPetsReached,
        /// 余额不足
        InsufficientBalance,
        /// 等级不足以进化
        LevelTooLow,
        /// 已达到最高进化阶段
        MaxEvolutionReached,
        /// 名称过长
        NameTooLong,
        /// 无法转移给自己
        CannotTransferToSelf,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 孵化新宠物
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::hatch_pet())]
        pub fn hatch_pet(
            origin: OriginFor<T>,
            name: BoundedVec<u8, ConstU32<32>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查宠物数量限制
            let mut owned_pets = PetOwners::<T>::get(&who);
            ensure!(
                owned_pets.len() < T::MaxPetsPerAccount::get() as usize,
                Error::<T>::MaxPetsReached
            );

            // 扣除孵化费用
            let fee = T::HatchingFee::get();
            T::Currency::withdraw(
                &who,
                fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // 生成随机数
            let (random_hash, _) = T::Randomness::random(&b"pet_hatch"[..]);
            let random_bytes = random_hash.as_ref();
            let random_seed = u32::from_le_bytes([
                random_bytes[0],
                random_bytes[1],
                random_bytes[2],
                random_bytes[3],
            ]);

            // 确定稀有度 (基于概率)
            let rarity_roll = random_seed % 100;
            let rarity = if rarity_roll < 60 {
                Rarity::Common
            } else if rarity_roll < 85 {
                Rarity::Rare
            } else if rarity_roll < 95 {
                Rarity::Epic
            } else if rarity_roll < 99 {
                Rarity::Legendary
            } else {
                Rarity::Mythic
            };

            // 确定元素类型
            let element_roll = (random_seed >> 8) % 5;
            let element = match element_roll {
                0 => Element::Normal,
                1 => Element::Fire,
                2 => Element::Water,
                3 => Element::Shadow,
                _ => Element::Light,
            };

            // 生成属性
            let attributes = PetAttributes::generate_base(&rarity, random_seed);

            // 生成基因哈希
            let gene_input = (who.clone(), random_seed, frame_system::Pallet::<T>::block_number());
            let gene_hash: [u8; 32] = T::Hashing::hash_of(&gene_input).as_ref().try_into().unwrap_or([0u8; 32]);

            // 创建宠物
            let pet_id = NextPetId::<T>::get();
            let pet = Pet {
                id: pet_id,
                owner: who.clone(),
                name: name.clone(),
                element: element.clone(),
                rarity: rarity.clone(),
                level: 1,
                experience: 0,
                attributes,
                evolution_stage: 0,
                created_at: frame_system::Pallet::<T>::block_number(),
                gene_hash,
            };

            // 存储宠物
            Pets::<T>::insert(pet_id, pet);
            owned_pets.try_push(pet_id).map_err(|_| Error::<T>::MaxPetsReached)?;
            PetOwners::<T>::insert(&who, owned_pets);
            NextPetId::<T>::put(pet_id.saturating_add(1));
            TotalPets::<T>::mutate(|n| *n = n.saturating_add(1));

            // 发送事件
            Self::deposit_event(Event::PetCreated {
                pet_id,
                owner: who,
                rarity,
                element,
            });

            Ok(())
        }

        /// 升级宠物
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::level_up())]
        pub fn level_up(origin: OriginFor<T>, pet_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Pets::<T>::try_mutate(pet_id, |maybe_pet| -> DispatchResult {
                let pet = maybe_pet.as_mut().ok_or(Error::<T>::PetNotFound)?;
                ensure!(pet.owner == who, Error::<T>::NotOwner);

                // 计算升级费用 (基础费用 * 等级 * 0.1)
                let base_fee = T::LevelUpBaseFee::get();
                let level_multiplier = pet.level.saturating_mul(10) / 100 + 1;
                let fee = base_fee.saturating_mul(level_multiplier.into());

                T::Currency::withdraw(
                    &who,
                    fee,
                    frame_support::traits::WithdrawReasons::FEE,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;

                // 升级
                pet.level = pet.level.saturating_add(1);
                pet.attributes.level_up(&pet.rarity);

                Self::deposit_event(Event::PetLeveledUp {
                    pet_id,
                    new_level: pet.level,
                });

                Ok(())
            })
        }

        /// 进化宠物
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::evolve())]
        pub fn evolve(
            origin: OriginFor<T>,
            pet_id: u64,
            target_element: Option<Element>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Pets::<T>::try_mutate(pet_id, |maybe_pet| -> DispatchResult {
                let pet = maybe_pet.as_mut().ok_or(Error::<T>::PetNotFound)?;
                ensure!(pet.owner == who, Error::<T>::NotOwner);
                ensure!(pet.evolution_stage < 4, Error::<T>::MaxEvolutionReached);

                // 检查等级要求 (每阶段需要 20 级)
                let required_level = (pet.evolution_stage as u32 + 1) * 20;
                ensure!(pet.level >= required_level, Error::<T>::LevelTooLow);

                // 计算进化费用
                let base_fee = T::LevelUpBaseFee::get();
                let multiplier = T::EvolutionFeeMultiplier::get();
                let stage_multiplier = (pet.evolution_stage as u32 + 1) * multiplier;
                let fee = base_fee.saturating_mul(stage_multiplier.into());

                T::Currency::withdraw(
                    &who,
                    fee,
                    frame_support::traits::WithdrawReasons::FEE,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;

                // 进化
                pet.evolution_stage = pet.evolution_stage.saturating_add(1);

                // 可选择改变元素
                let new_element = target_element.unwrap_or(pet.element.clone());
                pet.element = new_element.clone();

                // 属性提升
                let boost = match pet.evolution_stage {
                    1 => 10,
                    2 => 15,
                    3 => 20,
                    4 => 30,
                    _ => 5,
                };
                pet.attributes.health = pet.attributes.health.saturating_mul(100 + boost) / 100;
                pet.attributes.attack = pet.attributes.attack.saturating_mul(100 + boost) / 100;
                pet.attributes.defense = pet.attributes.defense.saturating_mul(100 + boost) / 100;
                pet.attributes.speed = pet.attributes.speed.saturating_mul(100 + boost) / 100;

                Self::deposit_event(Event::PetEvolved {
                    pet_id,
                    new_stage: pet.evolution_stage,
                    new_element,
                });

                Ok(())
            })
        }

        /// 转移宠物
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::transfer())]
        pub fn transfer(
            origin: OriginFor<T>,
            pet_id: u64,
            to: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who != to, Error::<T>::CannotTransferToSelf);

            // 检查接收者宠物数量
            let mut to_pets = PetOwners::<T>::get(&to);
            ensure!(
                to_pets.len() < T::MaxPetsPerAccount::get() as usize,
                Error::<T>::MaxPetsReached
            );

            Pets::<T>::try_mutate(pet_id, |maybe_pet| -> DispatchResult {
                let pet = maybe_pet.as_mut().ok_or(Error::<T>::PetNotFound)?;
                ensure!(pet.owner == who, Error::<T>::NotOwner);

                // 更新所有者
                pet.owner = to.clone();

                // 更新发送者的宠物列表
                PetOwners::<T>::mutate(&who, |pets| {
                    pets.retain(|&id| id != pet_id);
                });

                // 更新接收者的宠物列表
                to_pets.try_push(pet_id).map_err(|_| Error::<T>::MaxPetsReached)?;
                PetOwners::<T>::insert(&to, to_pets);

                Self::deposit_event(Event::PetTransferred {
                    pet_id,
                    from: who,
                    to,
                });

                Ok(())
            })
        }

        /// 重命名宠物
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::rename())]
        pub fn rename(
            origin: OriginFor<T>,
            pet_id: u64,
            new_name: BoundedVec<u8, ConstU32<32>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Pets::<T>::try_mutate(pet_id, |maybe_pet| -> DispatchResult {
                let pet = maybe_pet.as_mut().ok_or(Error::<T>::PetNotFound)?;
                ensure!(pet.owner == who, Error::<T>::NotOwner);

                pet.name = new_name.clone();

                Self::deposit_event(Event::PetRenamed { pet_id, new_name });

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// 获取宠物信息
        pub fn get_pet(pet_id: u64) -> Option<Pet<T>> {
            Pets::<T>::get(pet_id)
        }

        /// 获取账户的所有宠物
        pub fn get_pets_by_owner(owner: &T::AccountId) -> Vec<Pet<T>> {
            PetOwners::<T>::get(owner)
                .iter()
                .filter_map(|&pet_id| Pets::<T>::get(pet_id))
                .collect()
        }

        /// 计算升级所需经验值
        pub fn exp_required_for_level(level: u32) -> u64 {
            // EXP = 100 * level^1.5 + 50 * level
            // 使用整数近似: level^1.5 ≈ level * sqrt(level)
            let level_u64 = level as u64;
            let sqrt_level = Self::integer_sqrt(level_u64);
            let exp = 100 * level_u64 * sqrt_level / 10 + 50 * level_u64;
            exp
        }

        /// 整数平方根
        fn integer_sqrt(n: u64) -> u64 {
            if n == 0 {
                return 0;
            }
            let mut x = n;
            let mut y = (x + 1) / 2;
            while y < x {
                x = y;
                y = (x + n / x) / 2;
            }
            x
        }

        /// 增加经验值
        pub fn add_experience(pet_id: u64, exp: u64) -> DispatchResult {
            Pets::<T>::try_mutate(pet_id, |maybe_pet| -> DispatchResult {
                let pet = maybe_pet.as_mut().ok_or(Error::<T>::PetNotFound)?;
                pet.experience = pet.experience.saturating_add(exp);
                Ok(())
            })
        }

        /// 内部转移函数 (供其他 pallet 调用)
        pub fn do_transfer(
            pet_id: u64,
            from: &T::AccountId,
            to: &T::AccountId,
        ) -> DispatchResult {
            ensure!(from != to, Error::<T>::CannotTransferToSelf);

            // 检查接收者宠物数量
            let mut to_pets = PetOwners::<T>::get(to);
            ensure!(
                to_pets.len() < T::MaxPetsPerAccount::get() as usize,
                Error::<T>::MaxPetsReached
            );

            Pets::<T>::try_mutate(pet_id, |maybe_pet| -> DispatchResult {
                let pet = maybe_pet.as_mut().ok_or(Error::<T>::PetNotFound)?;
                ensure!(pet.owner == *from, Error::<T>::NotOwner);

                // 更新所有者
                pet.owner = to.clone();

                // 更新发送者的宠物列表
                PetOwners::<T>::mutate(from, |pets| {
                    pets.retain(|&id| id != pet_id);
                });

                // 更新接收者的宠物列表
                to_pets.try_push(pet_id).map_err(|_| Error::<T>::MaxPetsReached)?;
                PetOwners::<T>::insert(to, to_pets);

                Self::deposit_event(Event::PetTransferred {
                    pet_id,
                    from: from.clone(),
                    to: to.clone(),
                });

                Ok(())
            })
        }
    }
}
