# Pallet Matchmaking Profile（婚恋模块 - 用户资料管理）

## 📋 模块概述

`pallet-matchmaking-profile` 是婚恋系统的用户资料管理模块，提供用户资料的创建、更新、查询和隐私控制功能。

### 核心特性

- ✅ **资料创建**：创建用户婚恋资料，需支付 50 USDT 等值保证金
- ✅ **资料更新**：更新个人信息、择偶条件、隐私设置
- ✅ **八字绑定**：绑定八字命盘用于合婚匹配
- ✅ **性格分析**：用户自填性格 + 八字解盘性格综合分析
- ✅ **照片管理**：IPFS 存储，自动 Pin 固定
- ✅ **会员系统**：月费会员，15层推荐链分成
- ✅ **保证金机制**：违规罚没，保护平台生态
- ✅ **隐私保护**：字段级隐私控制

---

## 🔑 核心功能

### 1. 资料创建与保证金

创建资料需支付 **50 USDT 等值的 COS** 作为保证金：

```rust
pub fn create_profile(
    origin: OriginFor<T>,
    nickname: BoundedVec<u8, T::MaxNicknameLen>,
    gender: Gender,
    age: Option<u8>,
    birth_date: Option<BirthDate>,
    current_location: Option<BoundedVec<u8, T::MaxLocationLen>>,
    bio: Option<BoundedVec<u8, T::MaxBioLen>>,
) -> DispatchResult;
```

**保证金计算**：
- 优先使用实时汇率（通过 `PricingProvider`）
- 汇率不可用时使用兆底金额

### 2. 月费会员系统

支付 **2 USDT/月** 等值的 COS 成为会员：

```rust
pub fn pay_monthly_fee(
    origin: OriginFor<T>,
    months: u32,  // 1-12个月
) -> DispatchResult;
```

**费用分配（15层推荐链）**：

| 分配项 | 比例 |
|--------|------|
| 销毁 | 5% |
| 国库 | 2% |
| 存储 | 3% |
| 推荐链分配 | 90% |

### 3. 性格分析系统

**用户自填性格**：

```rust
pub fn update_user_personality(
    origin: OriginFor<T>,
    traits: BoundedVec<PersonalityTrait, ConstU32<5>>,  // 最多5个
    self_description: Option<BoundedVec<u8, T::MaxDescLen>>,
) -> DispatchResult;
```

**八字性格同步**：

```rust
pub fn sync_bazi_personality(
    origin: OriginFor<T>,
    bazi_main_traits: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
    bazi_strengths: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
    bazi_weaknesses: BoundedVec<BaziPersonalityTrait, ConstU32<2>>,
) -> DispatchResult;
```

### 4. 照片管理（IPFS 集成）

**上传照片**：

```rust
pub fn upload_photo(
    origin: OriginFor<T>,
    cid: BoundedVec<u8, T::MaxCidLen>,
    is_avatar: bool,
) -> DispatchResult;
```

**批量上传**：

```rust
pub fn upload_photos_batch(
    origin: OriginFor<T>,
    cids: BoundedVec<BoundedVec<u8, T::MaxCidLen>, ConstU32<9>>,  // 最多9张
) -> DispatchResult;
```

- 使用 `Standard` 层级存储（3副本，24小时巡检）
- 删除资料时自动 Unpin

### 5. 违规处理与保证金罚没

**违规类型与罚没比例**：

| 违规类型 | 罚没比例 | 暂停天数 | 是否封禁 |
|----------|----------|----------|----------|
| Minor（轻微） | 5% | 0 | 否 |
| Moderate（一般） | 10% | 0 | 否 |
| Severe（严重） | 20% | 7天 | 否 |
| Critical（特别严重） | 50% | 30天 | 否 |
| PermanentBan（永久封禁） | 100% | - | 是 |

**治理调用**：

```rust
pub fn handle_violation(
    origin: OriginFor<T>,  // 需要治理权限
    user: T::AccountId,
    violation_type: ViolationType,
    reason: SlashReason,
) -> DispatchResult;
```

---

## 📊 数据结构

### 用户资料（UserProfile）

```rust
pub struct UserProfile<T: Config> {
    // ========== 基本信息 ==========
    pub nickname: BoundedVec<u8, T::MaxNicknameLen>,
    pub gender: Gender,
    pub age: Option<u8>,
    pub birth_date: Option<BirthDate>,
    pub birth_time: Option<BirthTime>,
    pub birth_location: Option<BirthLocation<T>>,
    pub current_location: Option<BoundedVec<u8, T::MaxLocationLen>>,
    pub avatar_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
    pub photo_cids: BoundedVec<BoundedVec<u8, T::MaxCidLen>, ConstU32<9>>,
    
    // ========== 个人条件 ==========
    pub height: Option<u16>,
    pub weight: Option<u16>,
    pub education: Option<EducationLevel>,
    pub occupation: Option<BoundedVec<u8, T::MaxOccupationLen>>,
    pub income_range: Option<(u32, u32)>,
    pub property_status: Option<PropertyStatus>,
    pub vehicle_status: Option<VehicleStatus>,
    pub marital_status: Option<MaritalStatus>,
    pub has_children: Option<bool>,
    pub wants_children: Option<bool>,
    
    // ========== 性格与兴趣 ==========
    pub personality_traits: BoundedVec<PersonalityTrait, T::MaxTraits>,
    pub hobbies: BoundedVec<BoundedVec<u8, T::MaxHobbyLen>, T::MaxHobbies>,
    pub lifestyle: Option<Lifestyle>,
    
    // ========== 玄学信息 ==========
    pub bazi_chart_id: Option<u64>,
    pub compatibility_preferences: Option<CompatibilityPreferences>,
    
    // ========== 择偶条件 ==========
    pub partner_preferences: Option<PartnerPreferences<T>>,
    
    // ========== 自我介绍 ==========
    pub bio: Option<BoundedVec<u8, T::MaxBioLen>>,
    pub ideal_partner_desc: Option<BoundedVec<u8, T::MaxDescLen>>,
    
    // ========== 隐私与权限 ==========
    pub privacy_mode: ProfilePrivacyMode,
    pub field_privacy: FieldPrivacySettings,
    
    // ========== 状态与元数据 ==========
    pub completeness: u8,
    pub status: ProfileStatus,
    pub verified: bool,
    pub created_at: BlockNumberFor<T>,
    pub updated_at: BlockNumberFor<T>,
    pub last_active_at: BlockNumberFor<T>,
}
```

### 性格分析数据

```rust
pub struct PersonalityAnalysisData<T: Config> {
    // 用户自填
    pub user_traits: BoundedVec<PersonalityTrait, ConstU32<5>>,
    pub self_description: Option<BoundedVec<u8, T::MaxDescLen>>,
    
    // 八字解盘
    pub bazi_main_traits: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
    pub bazi_strengths: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
    pub bazi_weaknesses: BoundedVec<BaziPersonalityTrait, ConstU32<2>>,
    
    // 元数据
    pub source: PersonalitySource,
    pub bazi_chart_id: Option<u64>,
    pub updated_at: BlockNumberFor<T>,
}
```

---

## 💾 存储项

```rust
/// 用户资料
pub type Profiles<T> = StorageMap<_, Blake2_128Concat, T::AccountId, UserProfile<T>>;

/// 用户总数
pub type ProfileCount<T> = StorageValue<_, u64, ValueQuery>;

/// 性别索引（用于推荐）
pub type GenderIndex<T> = StorageDoubleMap<_, Blake2_128Concat, Gender, Blake2_128Concat, T::AccountId, ()>;

/// 用户保证金记录
pub type Deposits<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

/// 用户会员到期时间
pub type MembershipExpiry<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>>;

/// 用户性格分析
pub type PersonalityAnalyses<T> = StorageMap<_, Blake2_128Concat, T::AccountId, PersonalityAnalysisData<T>>;

/// 封禁用户列表
pub type BannedUsers<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>>;

/// 暂停用户列表
pub type SuspendedUntil<T> = StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>>;
```

---

## 🎯 外部调用（Extrinsics）

| 调用 | 描述 | 权限 |
|------|------|------|
| `create_profile` | 创建资料（需保证金） | 用户 |
| `update_profile` | 更新基本信息 | 用户 |
| `update_preferences` | 更新择偶条件 | 用户 |
| `link_bazi` | 绑定八字命盘 | 用户 |
| `update_privacy_mode` | 更新隐私模式 | 用户 |
| `delete_profile` | 删除资料（释放保证金） | 用户 |
| `pay_monthly_fee` | 支付月费 | 用户 |
| `update_user_personality` | 更新自填性格 | 用户 |
| `sync_bazi_personality` | 同步八字性格 | 用户 |
| `upload_photo` | 上传照片 | 用户 |
| `upload_photos_batch` | 批量上传照片 | 用户 |
| `handle_violation` | 处理违规 | 治理 |
| `top_up_deposit` | 补充保证金 | 用户 |
| `lift_suspension` | 解除暂停 | 用户/治理 |

---

## 📡 事件定义

```rust
ProfileCreated { who, nickname, gender }
ProfileUpdated { who }
PreferencesUpdated { who }
BaziLinked { who, bazi_id }
PrivacyModeUpdated { who, mode }
ProfileDeleted { who }
DepositLocked { who, amount }
DepositReleased { who, amount }
DepositSlashed { who, amount, reason }
MonthlyFeePaid { who, amount, months, expiry_block }
MembershipExpired { who }
UserPersonalityUpdated { who, traits_count }
BaziPersonalitySynced { who, bazi_chart_id }
PhotoUploaded { who, cid, pin_tier }
AvatarUpdated { who, cid }
PhotoUnpinned { who, cid }
UserBanned { who, reason }
UserSuspended { who, until_block, reason }
DepositToppedUp { who, amount, new_total }
DepositInsufficient { who, current, required }
```

---

## ⚙️ 配置参数

```rust
impl pallet_matchmaking_profile::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    
    // 长度限制
    type MaxNicknameLen = ConstU32<32>;
    type MaxLocationLen = ConstU32<64>;
    type MaxCidLen = ConstU32<64>;
    type MaxBioLen = ConstU32<512>;
    type MaxDescLen = ConstU32<256>;
    type MaxOccupationLen = ConstU32<64>;
    type MaxTraits = ConstU32<10>;
    type MaxHobbies = ConstU32<10>;
    type MaxHobbyLen = ConstU32<32>;
    
    // 保证金配置
    type ProfileDeposit = ConstU128<50_000_000_000_000_000_000>;  // 50 COS 兆底
    type ProfileDepositUsd = ConstU64<50_000_000>;  // 50 USDT
    type MonthlyFee = ConstU128<2_000_000_000_000_000_000>;  // 2 COS 兆底
    type MonthlyFeeUsd = ConstU64<2_000_000>;  // 2 USDT
    
    // 其他配置
    type BlocksPerDay = ConstU32<14400>;
    type Fungible = Balances;
    type Pricing = PricingPallet;
    type IpfsPinner = StorageService;
    type AffiliateDistributor = AffiliatePallet;
    type TreasuryAccount = TreasuryAccountId;
    type BurnAccount = BurnAccountId;
    type StorageAccount = StorageAccountId;
    type GovernanceOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}
```

---

## 📚 相关文档

- [婚恋模块主文档](../README.md)
- [共享类型模块](../common/README.md)
- [匹配算法模块](../matching/README.md)
- [互动功能模块](../interaction/README.md)

---

## 📝 版本历史

### v0.1.0（当前版本）

- ✅ 用户资料 CRUD
- ✅ 保证金机制
- ✅ 月费会员系统
- ✅ 性格分析系统
- ✅ IPFS 照片管理
- ✅ 违规处理机制
