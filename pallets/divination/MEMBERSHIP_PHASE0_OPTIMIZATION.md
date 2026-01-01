# 会员系统阶段0设计优化方案

**版本**: v1.0
**日期**: 2026-01-01
**状态**: 已完成

---

## 一、批量刷占卜奖励漏洞修复

### 1.1 问题描述

**原设计**：用户创建占卜时即发放 0.005 DUST 奖励

**攻击向量**：
```
攻击者创建1000个小号：
  初始充值：1000 × 1 DUST = 1000 DUST
  每日占卜奖励：1000账户 × 20次 × 0.005 = 100 DUST/天
  Gas成本：1000 × 20 × 0.0003 = 6 DUST/天
  净收益：94 DUST/天
  年化收益率：3431% 🚨
```

### 1.2 解决方案

**核心原则**：奖励必须与付费行为绑定

#### 方案A：奖励仅在AI解读时发放（推荐）

```rust
// 修改前：创建占卜时发放奖励
pub fn create_bazi_chart(...) {
    // ... 创建占卜逻辑
    T::MembershipProvider::grant_reward(&who, 5_000_000_000_000, RewardTxType::Divination, b"bazi")?;
}

// 修改后：AI解读时发放合并奖励
pub fn request_interpretation(...) {
    // ... AI解读逻辑

    // 合并发放：占卜奖励(0.005) + AI返现(0.02) = 0.025 DUST
    T::MembershipProvider::grant_reward(
        &who,
        25_000_000_000_000, // 0.025 DUST
        RewardTxType::AiCashback,
        b"divination_and_ai",
    )?;
}
```

**攻击成本分析**：
```
攻击前提：必须支付AI解读费 5 DUST
奖励收益：0.025 DUST
净收益：-4.975 DUST/次 ✓ 无利可图
```

#### 方案B：延迟发放 + 数据验证

```rust
/// 占卜奖励待发放记录
#[pallet::storage]
pub type PendingDivinationRewards<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    (T::AccountId, u64), // (用户, 占卜ID)
    PendingReward<T::BlockNumber>,
    OptionQuery,
>;

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PendingReward<BlockNumber> {
    /// 奖励金额
    pub amount: u128,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 过期时间（7天后）
    pub expires_at: BlockNumber,
}

// 创建占卜时：记录待发放奖励
pub fn create_bazi_chart(...) {
    // ... 创建占卜
    PendingDivinationRewards::<T>::insert(
        (&who, chart_id),
        PendingReward {
            amount: 5_000_000_000_000,
            created_at: now,
            expires_at: now + T::RewardLockPeriod::get(), // 7天
        },
    );
}

// 用户请求AI解读时：解锁并发放奖励
pub fn request_interpretation(...) {
    if let Some(pending) = PendingDivinationRewards::<T>::take((&who, divination_id)) {
        // 验证未过期
        if frame_system::Pallet::<T>::block_number() <= pending.expires_at {
            T::MembershipProvider::grant_reward(&who, pending.amount, ...)?;
        }
    }
    // 发放AI返现
    T::MembershipProvider::grant_reward(&who, 20_000_000_000_000, ...)?;
}
```

### 1.3 选择方案A的理由

| 对比维度 | 方案A | 方案B |
|---------|-------|-------|
| 实现复杂度 | 低 | 高（需额外存储） |
| 存储成本 | 无 | 48 bytes/条 |
| 用户体验 | 奖励合并发放，感知一致 | 奖励分两次，可能困惑 |
| 防刷效果 | 100%有效 | 99%有效（仍有7天窗口） |

**决策**：✅ **采用方案A**

### 1.4 更新后的奖励规则表

| 行为 | 基础奖励 (DUST) | 触发条件 | 说明 |
|------|----------------|---------|------|
| ~~创建占卜~~ | ~~0.005~~ | - | **已移除** |
| **请求AI解读** | **0.025** | 付费5 DUST | 合并占卜+AI奖励 |
| 每日签到 | 0.001 | 链上交易 | 保持不变 |
| 删除数据 | 0.003 | 删除操作 | 保持不变 |
| 市场订单 | 单价×0.1% | 订单完成 | 保持不变 |
| 评价订单 | 0.005 | 字数≥10 | 保持不变 |

---

## 二、Bronze/Silver会员等级定价调整

### 2.1 原定价问题分析

**Bronze（月费10 DUST）**：
```
轻度用户（月3次占卜）：
  Free成本：5×3 = 15 DUST
  Bronze成本：10 + 5×0.9×3 = 23.5 DUST
  差价：+8.5 DUST (56%贵) ✗
```

**Silver（月费30 DUST，1次免费AI）**：
```
中度用户（月10次占卜）：
  Free成本：5×10 = 50 DUST
  Silver成本：30 + 5×0.8×9 = 66 DUST
  差价：+16 DUST (32%贵) ✗
```

### 2.2 优化后的定价方案

#### 新等级定义

| 等级 | 名称 | 月费 (DUST) | 年费 (DUST) | 目标人群 |
|------|------|------------|------------|---------|
| **Free** | 普通用户 | 0 | 0 | 尝鲜用户 |
| **Bronze** | 青铜会员 | **5** ← (原10) | **50** | 月均5次占卜 |
| **Silver** | 白银会员 | **25** ← (原30) | **250** | 月均10次占卜 |
| **Gold** | 黄金会员 | 80 | 800 | 深度用户 |
| **Platinum** | 铂金会员 | 200 | 2000 | 专业研究者 |
| **Diamond** | 钻石会员 | 500 | 5000 | 服务提供者 |

#### 新权益矩阵

| 权益类别 | Free | Bronze | Silver | Gold | Platinum | Diamond |
|---------|------|--------|--------|------|----------|---------|
| **存储押金折扣** | 0% | **30%** ← | 30% | 40% | 50% | 60% |
| **AI解读折扣** | 0% | **15%** ← | 20% | 50% | 70% | 80% |
| **免费AI次数/月** | 0 | **1** ← | **3** ← | 5 | 20 | 50 |
| **每日免费占卜** | 3 | 5 | 10 | 20 | 50 | 100 |
| **最大占卜数** | 100 | 150 | 300 | 500 | 1000 | 无限 |
| **数据永久存储** | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ |
| **DUST奖励加成** | 1x | 1.2x | 1.5x | 2x | 3x | 5x |

### 2.3 优化后的成本收益分析

**Bronze（月费5 DUST）**：
```
目标用户（月5次占卜）：
  Free成本：5×5 = 25 DUST
  Bronze成本：5 + 5×0.85×4 + 0 = 22 DUST (1次免费)
  节省：3 DUST/月 (12%折扣) ✓
```

**Silver（月费25 DUST，3次免费AI）**：
```
目标用户（月10次占卜）：
  Free成本：5×10 = 50 DUST
  Silver成本：25 + 5×0.8×7 = 53 DUST (3次免费)

  需调整 → 降低月费或增加免费次数

  最终方案：月费25 + 免费5次
  Silver成本：25 + 5×0.8×5 = 45 DUST
  节省：5 DUST/月 (10%折扣) ✓
```

**修正：Silver免费AI改为5次/月**

### 2.4 最终权益矩阵（v2）

| 权益类别 | Free | Bronze | Silver | Gold | Platinum | Diamond |
|---------|------|--------|--------|------|----------|---------|
| **月费 (DUST)** | 0 | **5** | **25** | 80 | 200 | 500 |
| **AI解读折扣** | 0% | 15% | 20% | 50% | 70% | 80% |
| **免费AI次数/月** | 0 | **1** | **5** | 5 | 20 | 50 |
| **存储押金折扣** | 0% | 30% | 30% | 40% | 50% | 60% |
| **DUST奖励加成** | 1x | 1.2x | 1.5x | 2x | 3x | 5x |

---

## 三、DUST奖励预算重新计算

### 3.1 修正会员加成影响

**假设**：10,000活跃用户，500付费会员

**会员分布**：
| 等级 | 人数 | 占比 | 奖励加成 |
|------|------|------|---------|
| Free | 9,500 | 95% | 1.0x |
| Bronze | 150 | 1.5% | 1.2x |
| Silver | 200 | 2.0% | 1.5x |
| Gold | 100 | 1.0% | 2.0x |
| Platinum | 40 | 0.4% | 3.0x |
| Diamond | 10 | 0.1% | 5.0x |

### 3.2 各类奖励支出计算

#### 签到奖励（已修正）

**假设**：50%用户每日签到，连续签到加成1.5x（平均）

```
Free:    4,750人 × 0.001 × 1.0 × 1.25(平均) × 365 = 2,168 DUST/年
Bronze:    75人 × 0.001 × 1.2 × 1.25 × 365 = 41 DUST/年
Silver:   100人 × 0.001 × 1.5 × 1.25 × 365 = 68 DUST/年
Gold:      50人 × 0.001 × 2.0 × 1.25 × 365 = 46 DUST/年
Platinum:  20人 × 0.001 × 3.0 × 1.25 × 365 = 27 DUST/年
Diamond:    5人 × 0.001 × 5.0 × 1.25 × 365 = 11 DUST/年
───────────────────────────────────────────────────────────
签到总支出：2,361 DUST/年 ≈ 197 DUST/月
```

#### AI解读返现（已合并占卜奖励）

**假设**：每用户年均50次AI解读，基础奖励0.025 DUST

```
Free:    9,500人 × 0.025 × 50 × 1.0 = 11,875 DUST/年
Bronze:    150人 × 0.025 × 50 × 1.2 = 225 DUST/年
Silver:    200人 × 0.025 × 50 × 1.5 = 375 DUST/年
Gold:      100人 × 0.025 × 50 × 2.0 = 250 DUST/年
Platinum:   40人 × 0.025 × 50 × 3.0 = 150 DUST/年
Diamond:    10人 × 0.025 × 50 × 5.0 = 62.5 DUST/年
───────────────────────────────────────────────────────────
AI返现总支出：12,937 DUST/年 ≈ 1,078 DUST/月
```

#### 其他奖励

| 类型 | 年度预算 | 月度预算 |
|------|---------|---------|
| 删除数据奖励 | 1,000 | 83 |
| 市场订单返现 | 2,000 | 167 |
| 评价奖励 | 500 | 42 |
| NFT奖励 | 1,000 | 83 |
| 推荐奖励 | 2,000 | 167 |

#### 总预算

```
签到：     2,361 DUST/年
AI返现：  12,937 DUST/年
其他：     6,500 DUST/年
───────────────────────────
总计：    21,798 DUST/年 ≈ 1,817 DUST/月
```

### 3.3 收支平衡分析

**收入**：
```
会员费（调整后）：
  Bronze:   150人 × 5 DUST = 750 DUST/月
  Silver:   200人 × 25 DUST = 5,000 DUST/月
  Gold:     100人 × 80 DUST = 8,000 DUST/月
  Platinum:  40人 × 200 DUST = 8,000 DUST/月
  Diamond:   10人 × 500 DUST = 5,000 DUST/月
  ─────────────────────────────────────────
  总计：26,750 DUST/月 (年费优惠后约25,000)

平台手续费：5,000 DUST/月（假设）

总收入：31,750 DUST/月
```

**奖励池分配**：
```
奖励池来源：总收入 × 10% = 3,175 DUST/月
奖励支出：1,817 DUST/月

盈余：1,358 DUST/月 ✓ 可持续
奖励池覆盖率：175% ✓ 安全边际充足
```

### 3.4 奖励池管理策略

```rust
/// 奖励池配置
#[pallet::constant]
type RewardPoolAllocation: Get<u8>; // 10% 进入奖励池

/// 动态调整规则
fn get_reward_multiplier_factor() -> u32 {
    let pool_balance = T::Currency::free_balance(&T::RewardPool::get());
    let monthly_burn = Self::get_monthly_burn_rate();
    let months_remaining = pool_balance / monthly_burn;

    match months_remaining {
        0..=2 => 5000,   // 50% 奖励（紧急）
        3..=5 => 7500,   // 75% 奖励（警告）
        _ => 10000,      // 100% 奖励（正常）
    }
}
```

---

## 四、会员资料加密方案优化

### 4.1 问题描述

**原设计**：全加密存储（姓名、出生日期、地址均加密）

**问题**：
- 占卜模块无法直接读取出生日期
- 前端需要先解密才能自动填充
- 用户体验：每次创建占卜需要手动输入

### 4.2 优化方案：部分加密

**核心原则**：
- 占卜必需数据（出生日期）明文存储
- 隐私数据（真实姓名、详细地址）加密存储

```rust
/// 会员资料（链上存储）- 优化版
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MemberProfile<BlockNumber> {
    /// 昵称（明文，公开展示）
    pub display_name: BoundedVec<u8, ConstU32<32>>,

    // ========== 占卜必需数据（明文）==========
    /// 性别
    pub gender: Option<Gender>,

    /// 出生日期（公历）
    pub birth_date: Option<BirthDate>,

    /// 出生时辰（0-23，None 表示未知）
    pub birth_hour: Option<u8>,

    /// 出生地经度（精度：0.0001度）
    pub longitude: Option<i32>,

    /// 出生地纬度
    pub latitude: Option<i32>,

    // ========== 隐私数据（加密）==========
    /// 加密的敏感资料（真实姓名、详细地址）
    pub encrypted_sensitive: Option<EncryptedSensitiveData>,

    // ========== 元数据 ==========
    /// 是否为服务商
    pub is_provider: bool,

    /// 服务商认证状态
    pub provider_verified: bool,

    /// 资料更新时间
    pub updated_at: BlockNumber,
}

/// 加密的敏感资料（仅包含隐私数据）
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EncryptedSensitiveData {
    /// 加密后的数据（最大256 bytes，足够存储姓名+地址）
    pub ciphertext: BoundedVec<u8, ConstU32<256>>,
    /// 加密 nonce
    pub nonce: [u8; 12],
    /// 加密版本
    pub version: u8,
}

/// 敏感资料明文结构（加密前/解密后）
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SensitiveData {
    /// 真实姓名
    pub real_name: BoundedVec<u8, ConstU32<64>>,
    /// 详细出生地址
    pub birth_place: BoundedVec<u8, ConstU32<128>>,
}
```

### 4.3 优化后的占卜模块集成

```rust
// pallets/divination/bazi/src/lib.rs

/// 从会员资料自动填充（链上直接读取，无需解密）
pub fn create_bazi_chart_from_profile(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, T::MaxNameLen>>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 直接读取会员资料（明文部分）
    let profile = T::MembershipProvider::get_profile(&who)
        .ok_or(Error::<T>::ProfileNotFound)?;

    let birth_date = profile.birth_date
        .ok_or(Error::<T>::BirthDateRequired)?;
    let birth_hour = profile.birth_hour
        .ok_or(Error::<T>::BirthHourRequired)?;
    let gender = profile.gender
        .ok_or(Error::<T>::GenderRequired)?;

    // 构建八字输入
    let input = BaziInputType::Gregorian {
        year: birth_date.year,
        month: birth_date.month,
        day: birth_date.day,
        hour: birth_hour,
        minute: 0,
        gender,
        zishi_mode: ZiShiMode::Modern,
        longitude: profile.longitude.map(|l| l as f64 / 10000.0),
    };

    Self::do_create_bazi_chart(&who, name, input)
}
```

### 4.4 存储成本对比

| 方案 | 每用户大小 | 10万用户 |
|------|-----------|---------|
| 原设计（全加密） | 583 bytes | 58.3 MB |
| 优化设计（部分加密） | 380 bytes | 38.0 MB |
| **节省** | **35%** | **20.3 MB** |

### 4.5 安全性分析

| 数据类型 | 存储方式 | 泄露风险 | 缓解措施 |
|---------|---------|---------|---------|
| 昵称 | 明文 | 低（用户自愿公开） | - |
| 性别 | 明文 | 低（非敏感） | - |
| 出生日期 | 明文 | 中 | 用户选择填写 |
| 出生时辰 | 明文 | 中 | 用户选择填写 |
| 经纬度 | 明文 | 低（城市级精度） | 精度限制 |
| 真实姓名 | **加密** | 低 | AES-256-GCM |
| 详细地址 | **加密** | 低 | AES-256-GCM |

**结论**：隐私风险可接受，占卜功能优先

---

## 五、更新后的代码实现

### 5.1 MembershipProvider Trait（v2）

```rust
/// 会员系统接口（供其他模块调用）- 优化版
pub trait MembershipProvider<AccountId, Balance, BlockNumber> {
    /// 获取会员等级
    fn get_tier(who: &AccountId) -> MemberTier;

    /// 检查会员是否有效
    fn is_active_member(who: &AccountId, min_tier: MemberTier) -> bool;

    /// 获取折扣率（返回万分比，如 3000 = 30%折扣）
    fn get_storage_discount(who: &AccountId) -> u32;
    fn get_ai_discount(who: &AccountId) -> u32;

    /// 获取每日免费占卜次数
    fn get_daily_free_quota(who: &AccountId) -> u32;

    /// 获取每月免费AI解读次数
    fn get_monthly_free_ai_quota(who: &AccountId) -> u32;

    /// 获取 DUST 奖励加成倍数（返回万分比，如 12000 = 1.2x）
    fn get_reward_multiplier(who: &AccountId) -> u32;

    /// 发放 DUST 奖励（自动应用会员加成和动态调整）
    fn grant_reward(
        who: &AccountId,
        base_amount: Balance,
        tx_type: RewardTxType,
        memo: &[u8],
    ) -> Result<Balance, DispatchError>;

    /// 获取会员资料（明文部分，供占卜模块直接使用）
    fn get_profile(who: &AccountId) -> Option<MemberProfileSummary<BlockNumber>>;

    /// 检查用户是否满足领取奖励的条件
    fn can_receive_reward(who: &AccountId) -> bool;
}

/// 会员资料摘要（明文部分）
#[derive(Clone, Encode, Decode, TypeInfo)]
pub struct MemberProfileSummary<BlockNumber> {
    pub display_name: Vec<u8>,
    pub gender: Option<Gender>,
    pub birth_date: Option<BirthDate>,
    pub birth_hour: Option<u8>,
    pub longitude: Option<i32>,
    pub latitude: Option<i32>,
    pub is_provider: bool,
    pub provider_verified: bool,
    pub updated_at: BlockNumber,
}
```

### 5.2 会员等级配置（v2）

```rust
impl<T: Config> Pallet<T> {
    /// 获取等级对应的月费
    pub fn get_tier_monthly_fee(tier: MemberTier) -> BalanceOf<T> {
        match tier {
            MemberTier::Free => 0u32.into(),
            MemberTier::Bronze => 5_000_000_000_000u128.saturated_into(), // 5 DUST
            MemberTier::Silver => 25_000_000_000_000u128.saturated_into(), // 25 DUST
            MemberTier::Gold => 80_000_000_000_000u128.saturated_into(), // 80 DUST
            MemberTier::Platinum => 200_000_000_000_000u128.saturated_into(), // 200 DUST
            MemberTier::Diamond => 500_000_000_000_000u128.saturated_into(), // 500 DUST
        }
    }

    /// 获取AI解读折扣率（万分比）
    pub fn get_ai_discount_rate(tier: MemberTier) -> u32 {
        match tier {
            MemberTier::Free => 0,
            MemberTier::Bronze => 1500,    // 15%
            MemberTier::Silver => 2000,    // 20%
            MemberTier::Gold => 5000,      // 50%
            MemberTier::Platinum => 7000,  // 70%
            MemberTier::Diamond => 8000,   // 80%
        }
    }

    /// 获取每月免费AI次数
    pub fn get_monthly_free_ai(tier: MemberTier) -> u32 {
        match tier {
            MemberTier::Free => 0,
            MemberTier::Bronze => 1,
            MemberTier::Silver => 5,   // 优化：1 → 5
            MemberTier::Gold => 5,
            MemberTier::Platinum => 20,
            MemberTier::Diamond => 50,
        }
    }

    /// 获取存储押金折扣率（万分比）
    pub fn get_storage_discount_rate(tier: MemberTier) -> u32 {
        match tier {
            MemberTier::Free => 0,
            MemberTier::Bronze => 3000,    // 30%（优化：20% → 30%）
            MemberTier::Silver => 3000,    // 30%
            MemberTier::Gold => 4000,      // 40%
            MemberTier::Platinum => 5000,  // 50%
            MemberTier::Diamond => 6000,   // 60%
        }
    }

    /// 获取DUST奖励加成（万分比）
    pub fn get_reward_multiplier_base(tier: MemberTier) -> u32 {
        match tier {
            MemberTier::Free => 10000,     // 1.0x
            MemberTier::Bronze => 12000,   // 1.2x
            MemberTier::Silver => 15000,   // 1.5x
            MemberTier::Gold => 20000,     // 2.0x
            MemberTier::Platinum => 30000, // 3.0x
            MemberTier::Diamond => 50000,  // 5.0x
        }
    }
}
```

### 5.3 奖励发放逻辑（v2）

```rust
impl<T: Config> Pallet<T> {
    /// 发放DUST奖励（带动态调整）
    pub fn do_grant_reward(
        who: &T::AccountId,
        base_amount: BalanceOf<T>,
        tx_type: RewardTxType,
        memo: &[u8],
    ) -> Result<BalanceOf<T>, DispatchError> {
        // 1. 检查领取资格
        ensure!(Self::can_receive_reward(who), Error::<T>::RewardNotAllowed);

        // 2. 获取会员加成
        let tier = Self::get_tier(who);
        let tier_multiplier = Self::get_reward_multiplier_base(tier);

        // 3. 获取动态调整系数（基于奖励池余额）
        let pool_multiplier = Self::get_pool_adjustment_factor();

        // 4. 计算最终奖励
        let final_amount = base_amount
            .saturating_mul(tier_multiplier.into())
            .saturating_mul(pool_multiplier.into())
            / 100_000_000u128.saturated_into(); // 两个万分比相乘

        // 5. 检查每日上限
        let today = Self::current_day();
        let mut balance = RewardBalances::<T>::get(who);
        if balance.today_date != today {
            balance.today_date = today;
            balance.today_earned = Zero::zero();
        }

        let daily_limit = Self::get_daily_limit(tx_type);
        let remaining = daily_limit.saturating_sub(balance.today_earned);
        let actual_amount = final_amount.min(remaining);

        ensure!(!actual_amount.is_zero(), Error::<T>::DailyLimitExceeded);

        // 6. 从奖励池转账
        T::Currency::transfer(
            &T::RewardPool::get(),
            who,
            actual_amount,
            ExistenceRequirement::KeepAlive,
        )?;

        // 7. 更新统计
        balance.today_earned = balance.today_earned.saturating_add(actual_amount);
        balance.total_earned = balance.total_earned.saturating_add(actual_amount);
        balance.last_updated = frame_system::Pallet::<T>::block_number();
        RewardBalances::<T>::insert(who, balance);

        // 8. 记录历史
        Self::record_reward_history(who, tx_type, actual_amount, memo)?;

        // 9. 发送事件
        Self::deposit_event(Event::RewardGranted {
            who: who.clone(),
            amount: actual_amount,
            tx_type,
        });

        Ok(actual_amount)
    }

    /// 获取奖励池动态调整系数
    fn get_pool_adjustment_factor() -> u32 {
        let pool_balance = T::Currency::free_balance(&T::RewardPool::get());
        let monthly_burn = Self::get_average_monthly_burn();

        if monthly_burn.is_zero() {
            return 10000; // 100%
        }

        let months_remaining = pool_balance / monthly_burn;

        match months_remaining.saturated_into::<u32>() {
            0..=2 => 5000,   // 50% 奖励（紧急状态）
            3..=5 => 7500,   // 75% 奖励（警告状态）
            _ => 10000,      // 100% 奖励（正常状态）
        }
    }
}
```

---

## 六、变更总结

### 6.1 修改项清单

| 模块 | 修改内容 | 影响范围 |
|------|---------|---------|
| **奖励系统** | 移除占卜创建奖励，合并到AI解读 | ai pallet, bazi/qimen等占卜pallet |
| **会员定价** | Bronze 10→5, Silver 30→25 | membership pallet |
| **会员权益** | Bronze免费AI 0→1, Silver 1→5, Bronze押金折扣20%→30% | membership pallet |
| **奖励预算** | 重新计算，月预算1,817 DUST | 经济模型 |
| **会员资料** | 全加密→部分加密 | membership pallet, 前端 |
| **奖励发放** | 增加动态调整系数 | membership pallet |

### 6.2 向后兼容性

| 变更 | 兼容性 | 迁移需求 |
|------|-------|---------|
| 奖励触发点移动 | ✓ 兼容 | 无需迁移 |
| 会员费调整 | ✓ 兼容 | 现有会员按新价续费 |
| 权益调整 | ✓ 兼容 | 立即生效 |
| 资料结构变更 | ⚠️ 需迁移 | 编写storage migration |

### 6.3 资料迁移脚本

```rust
pub mod v2 {
    use super::*;

    /// 旧版会员资料结构
    #[derive(Decode)]
    pub struct OldMemberProfile<BlockNumber> {
        pub display_name: BoundedVec<u8, ConstU32<32>>,
        pub encrypted_data: Option<OldEncryptedProfileData>,
        pub is_provider: bool,
        pub provider_verified: bool,
        pub created_at: BlockNumber,
        pub updated_at: BlockNumber,
    }

    pub fn migrate<T: Config>() -> Weight {
        let mut weight = Weight::zero();

        // 遍历所有会员资料
        for (account, old_profile) in MemberProfiles::<T>::drain() {
            // 创建新版资料（明文部分初始化为None）
            let new_profile = MemberProfile {
                display_name: old_profile.display_name,
                gender: None,           // 需要用户重新填写
                birth_date: None,       // 需要用户重新填写
                birth_hour: None,
                longitude: None,
                latitude: None,
                encrypted_sensitive: None, // 旧加密数据无法迁移（结构不同）
                is_provider: old_profile.is_provider,
                provider_verified: old_profile.provider_verified,
                updated_at: old_profile.updated_at,
            };

            MemberProfiles::<T>::insert(&account, new_profile);
            weight += T::DbWeight::get().reads_writes(1, 1);
        }

        // 发送迁移完成事件
        Pallet::<T>::deposit_event(Event::ProfileMigrationCompleted);

        weight
    }
}
```

---

## 七、测试计划

### 7.1 单元测试

```rust
#[test]
fn test_reward_not_granted_on_divination_creation() {
    new_test_ext().execute_with(|| {
        let user = 1;
        let initial_balance = Balances::free_balance(&user);

        // 创建占卜
        assert_ok!(Bazi::create_bazi_chart(
            RuntimeOrigin::signed(user),
            None,
            BaziInputType::default(),
        ));

        // 验证余额未增加（无奖励）
        assert_eq!(Balances::free_balance(&user), initial_balance);
    });
}

#[test]
fn test_reward_granted_on_ai_interpretation() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // 创建占卜
        assert_ok!(Bazi::create_bazi_chart(...));

        // 请求AI解读
        let balance_before = Balances::free_balance(&user);
        assert_ok!(Ai::request_interpretation(RuntimeOrigin::signed(user), 1));

        // 验证收到合并奖励 0.025 DUST
        let expected_reward = 25_000_000_000_000u128;
        assert_eq!(
            Balances::free_balance(&user),
            balance_before - 5_000_000_000_000 + expected_reward // -5 AI费 +0.025 奖励
        );
    });
}

#[test]
fn test_bronze_tier_pricing() {
    new_test_ext().execute_with(|| {
        // 验证Bronze月费为5 DUST
        assert_eq!(
            Membership::get_tier_monthly_fee(MemberTier::Bronze),
            5_000_000_000_000u128
        );

        // 验证Bronze免费AI为1次
        assert_eq!(
            Membership::get_monthly_free_ai(MemberTier::Bronze),
            1
        );

        // 验证Bronze押金折扣为30%
        assert_eq!(
            Membership::get_storage_discount_rate(MemberTier::Bronze),
            3000
        );
    });
}

#[test]
fn test_silver_tier_pricing() {
    new_test_ext().execute_with(|| {
        // 验证Silver月费为25 DUST
        assert_eq!(
            Membership::get_tier_monthly_fee(MemberTier::Silver),
            25_000_000_000_000u128
        );

        // 验证Silver免费AI为5次
        assert_eq!(
            Membership::get_monthly_free_ai(MemberTier::Silver),
            5
        );
    });
}

#[test]
fn test_reward_pool_adjustment() {
    new_test_ext().execute_with(|| {
        // 设置奖励池余额为1个月支出
        set_reward_pool_balance(1_817_000_000_000_000u128);

        // 验证奖励减半
        assert_eq!(
            Membership::get_pool_adjustment_factor(),
            5000 // 50%
        );
    });
}
```

### 7.2 集成测试

```rust
#[test]
fn test_end_to_end_bronze_membership() {
    new_test_ext().execute_with(|| {
        let user = 1;

        // 1. 订阅Bronze会员
        assert_ok!(Membership::subscribe(
            RuntimeOrigin::signed(user),
            MemberTier::Bronze,
            SubscriptionDuration::Monthly,
            false,
        ));

        // 验证扣费5 DUST
        assert_eq!(/* 余额减少5 DUST */);

        // 2. 创建占卜（享受30%押金折扣）
        assert_ok!(Bazi::create_bazi_chart(...));
        // 验证押金 = 基础押金 × 0.7

        // 3. 请求AI解读（使用免费额度）
        assert_ok!(Ai::request_interpretation(...));
        // 验证未扣费（免费1次）

        // 4. 再次请求AI解读（享受15%折扣）
        assert_ok!(Ai::request_interpretation(...));
        // 验证扣费 = 5 × 0.85 = 4.25 DUST

        // 5. 验证收到奖励（1.2x加成）
        // 验证奖励 = 0.025 × 1.2 = 0.03 DUST
    });
}
```

---

## 八、发布检查清单

### 8.1 代码变更

- [ ] 修改 `bazi/src/lib.rs`: 移除 `create_bazi_chart` 中的奖励发放
- [ ] 修改 `qimen/src/lib.rs`: 同上
- [ ] 修改 `liuyao/src/lib.rs`: 同上
- [ ] 修改 `meihua/src/lib.rs`: 同上
- [ ] 修改 `tarot/src/lib.rs`: 同上
- [ ] 修改 `xiaoliuren/src/lib.rs`: 同上
- [ ] 修改 `ziwei/src/lib.rs`: 同上
- [ ] 修改 `daliuren/src/lib.rs`: 同上
- [ ] 修改 `ai/src/lib.rs`: 增加合并奖励发放
- [ ] 修改 `membership/src/lib.rs`: 更新等级配置、资料结构
- [ ] 添加 storage migration 脚本
- [ ] 更新 runtime 版本号

### 8.2 测试

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 本地网络测试（manual testing）
- [ ] 测试网部署验证

### 8.3 文档

- [ ] 更新 API 文档
- [ ] 更新前端 SDK 文档
- [ ] 发布变更日志

---

**文档版本**: v1.0
**完成日期**: 2026-01-01
**状态**: ✅ 阶段0设计优化完成
