# 占卜模块会员系统 (Membership Pallet) 设计方案

## 一、需求分析与功能定位

### 1.1 核心问题

**现有系统痛点**：
- ✗ 缺乏用户留存机制（用完即走）
- ✗ 无长期激励体系（仅有一次性押金）
- ✗ 高频用户无优惠（每次都要支付AI解读费）
- ✗ 服务商无订阅收入（仅按单收费）
- ✗ 平台缺乏稳定现金流

**目标用户画像**：
1. **高频占卜用户**：每月5次以上占卜，愿意为折扣付费
2. **隐私需求用户**：长期存储数据，希望降低押金成本
3. **专业研究者**：批量占卜、数据分析，需要API配额
4. **服务提供者**：希望降低平台费率，获得流量倾斜

### 1.2 功能定位

**会员系统 = 订阅制 + 积分系统 + 权益池**

```
┌─────────────────────────────────────────────────┐
│          Membership Pallet Architecture         │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │订阅管理  │  │积分系统  │  │权益引擎  │      │
│  │Tiers     │  │Points    │  │Benefits  │      │
│  └─────┬────┘  └─────┬────┘  └─────┬────┘      │
│        │             │              │           │
│        └─────────────┴──────────────┘           │
│                      │                          │
│         ┌────────────▼────────────┐             │
│         │   MembershipProvider    │             │
│         │   (Trait Interface)     │             │
│         └────────────┬────────────┘             │
│                      │                          │
│    ┌─────────────────┼─────────────────┐        │
│    │                 │                 │        │
│    ▼                 ▼                 ▼        │
│  占卜模块         AI模块           Market模块   │
│  (折扣)           (免费额度)        (费率优惠)   │
└─────────────────────────────────────────────────┘
```

---

## 二、会员等级体系设计

### 2.1 等级定义

| 等级 | 名称 | 月费 (DUST) | 年费 (DUST) | 目标人群 | 核心价值 |
|------|------|------------|------------|---------|---------|
| **Free** | 普通用户 | 0 | 0 | 尝鲜用户 | 基础功能 |
| **Bronze** | 青铜会员 | 10 | 100 (8.3折) | 月均3次占卜 | 押金折扣 |
| **Silver** | 白银会员 | 30 | 300 (8.3折) | 月均10次占卜 | AI解读折扣 |
| **Gold** | 黄金会员 | 80 | 800 (8.3折) | 深度用户 | 免费AI额度 |
| **Platinum** | 铂金会员 | 200 | 2000 (8.3折) | 专业研究者 | 批量折扣+API |
| **Diamond** | 钻石会员 | 500 | 5000 (8.3折) | 服务提供者 | 费率优惠+流量 |

### 2.2 权益矩阵

| 权益类别 | Free | Bronze | Silver | Gold | Platinum | Diamond |
|---------|------|--------|--------|------|----------|---------|
| **存储押金折扣** | 0% | 20% | 30% | 40% | 50% | 60% |
| **AI解读折扣** | 0% | 10% | 20% | 50% | 70% | 80% |
| **免费AI次数/月** | 0 | 0 | 1 | 5 | 20 | 50 |
| **每日免费占卜** | 3 | 5 | 10 | 20 | 50 | 100 |
| **最大占卜数** | 100 | 150 | 300 | 500 | 1000 | 无限 |
| **数据永久存储** | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ |
| **批量操作** | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ |
| **API访问** | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ |
| **服务商费率** | - | - | - | - | - | 5% |
| **市场流量加权** | 1x | 1x | 1x | 1x | 1x | 2x |
| **专属客服** | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| **DUST奖励加成** | 1x | 1.2x | 1.5x | 2x | 3x | 5x |

### 2.3 成本收益分析（用户视角）

**场景1：轻度用户（月均3次占卜）**
```
Free方案：
- AI解读：5 DUST × 3 = 15 DUST
- 押金：0.02 × 3 = 0.06 DUST (假设删除全退)
- 总成本：15 DUST/月

Bronze方案（月费10）：
- AI解读：5 × 0.9 × 3 = 13.5 DUST
- 押金：0.02 × 0.8 × 3 = 0.048 DUST
- 总成本：10 + 13.5 = 23.5 DUST/月
✗ 不划算，建议保持Free
```

**场景2：中度用户（月均10次占卜）**
```
Free方案：
- AI解读：5 × 10 = 50 DUST
- 押金：0.02 × 10 = 0.2 DUST
- 总成本：50 DUST/月

Silver方案（月费30）：
- AI解读：5 × 0.8 × 9 + 0 = 36 DUST (1次免费)
- 押金：0.02 × 0.7 × 10 = 0.14 DUST
- 总成本：30 + 36 = 66 DUST/月
✗ 临界点，需要12次以上才划算

优化建议：Silver免费次数改为3次
- 总成本：30 + 5×0.8×7 = 58 DUST/月
✓ 节省：8 DUST/月 (16%折扣)
```

**场景3：重度用户（月均30次占卜）**
```
Free方案：
- AI解读：5 × 30 = 150 DUST
- 押金：0.02 × 30 = 0.6 DUST
- 总成本：150 DUST/月

Gold方案（月费80）：
- AI解读：5 × 0.5 × 25 + 0 = 62.5 DUST (5次免费)
- 押金：0.02 × 0.6 × 30 = 0.36 DUST
- 总成本：80 + 62.5 = 142.5 DUST/月
✓ 节省：7.5 DUST/月 (5%折扣)
✓ 数据永久存储（押金可累积退还）
```

**场景4：服务提供者（月均100单）**
```
无会员：
- 平台费率：10%
- 100单 × 50 DUST × 10% = 500 DUST手续费

Diamond会员（月费500）：
- 平台费率：5%
- 100单 × 50 DUST × 5% = 250 DUST手续费
✓ 节省：250 DUST/月 (50%折扣)
✓ ROI：(500-250-500)/500 = -50%

需要200单才能盈亏平衡
- 200单 × 50 × 5% = 500 DUST手续费
- 净收益：500 - 500 = 0
✓ 200单以上开始盈利
```

**结论**：
- Bronze等级**需要调整**（当前无吸引力）
- Silver等级需要增加**免费AI次数到3次**
- Gold/Platinum适合重度用户
- Diamond适合月均**200单以上**服务商

---

## 三、DUST 奖励系统设计

> **设计变更说明**：取消独立积分系统，改用 DUST 原生代币直接奖励，简化用户认知，提升激励效果。

### 3.1 DUST 奖励规则

| 行为 | 基础奖励 (DUST) | 会员加成 | 每日上限 (DUST) | 说明 |
|------|----------------|---------|----------------|------|
| **每日签到** | 0.001 | ✓ | 0.001 | 连续7天×1.5倍 |
| **创建占卜** | 0.005 | ✓ | 0.1 | Public模式 |
| **创建占卜** | 0.01 | ✓ | 0.1 | Partial/Private模式 |
| **请求AI解读** | 0.02 | ✓ | 无限 | 付费行为返现 |
| **删除数据** | 0.003 | ✗ | 0.05 | 鼓励清理 |
| **市场订单** | 单价×0.1% | ✓ | 无限 | 买家消费返现 |
| **提交解读** | 单价×0.2% | ✓ | 无限 | 卖家服务奖励 |
| **评价订单** | 0.005 | ✓ | 0.05 | 高质量评价×2 |
| **推荐好友** | 0.1 | ✗ | 0.5 | 好友首次付费（通过 referral pallet） |
| **铸造NFT** | 0.03 | ✓ | 无限 | 创作激励 |
| **NFT交易** | 成交价×0.05% | ✓ | 无限 | 买卖双方 |

### 3.2 会员奖励加成

| 会员等级 | 奖励加成倍数 |
|---------|-------------|
| Free | 1.0x |
| Bronze | 1.2x |
| Silver | 1.5x |
| Gold | 2.0x |
| Platinum | 3.0x |
| Diamond | 5.0x |

**示例**：Diamond会员每日签到可获得 0.001 × 5 × 1.5(连续) = 0.0075 DUST

### 3.3 防刷机制

由于直接发放 DUST，必须严格防止羊毛党：

| 防刷措施 | 说明 |
|---------|------|
| **新账户冷却期** | 注册后 7 天内无法领取任何奖励 |
| **最低余额要求** | 账户需持有 ≥1 DUST 才能领取签到奖励 |
| **链上交易验证** | 签到需要发起链上交易（消耗 Gas） |
| **每日上限** | 各类奖励设置每日上限，防止批量刷取 |
| **推荐门槛** | 被邀请人累计消费 ≥30 DUST 后才发放推荐奖励（由 referral pallet 管理） |
| **评价验证** | 评价奖励需订单实际完成且评价字数 ≥10 字 |

### 3.4 经济模型分析

**年度奖励预算估算**（假设 10,000 活跃用户）：

```
签到奖励：0.001 × 10,000 × 365 = 3,650 DUST/年
占卜奖励：0.005 × 10,000 × 100次/年 = 5,000 DUST/年
AI解读返现：0.02 × 10,000 × 50次/年 = 10,000 DUST/年
其他奖励：约 5,000 DUST/年
─────────────────────────────────────────────────
总计：约 23,650 DUST/年（约 2,000 DUST/月）
```

**对比积分系统**：
- 积分系统：无直接成本，但用户感知弱
- DUST系统：年成本约 23,650 DUST，但激励效果强，用户留存率预期提升 20%+

**通胀控制**：
- 奖励 DUST 来源于平台收入（会员费、平台手续费）的 10%
- 设置年度奖励池上限：50,000 DUST
- 奖励池耗尽时，奖励自动减半直至下一周期

---

## 四、会员资料与加密存储

### 4.1 设计目标

会员资料用于：
1. **占卜自动填充** - 出生信息只需填一次，后续八字/紫微等占卜自动带入
2. **服务商展示** - 市场模块需要服务商的昵称、认证状态
3. **隐私保护** - 敏感信息（姓名、出生信息、地址）加密存储，仅用户本人可解密

### 4.2 加密方案设计

**核心原则**：敏感数据使用用户私钥派生的对称密钥加密，链上只存密文

```
┌─────────────────────────────────────────────────────────────┐
│                    加密存储架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  用户私钥 ──► HKDF派生 ──► 对称密钥 (AES-256-GCM)           │
│                              │                               │
│                              ▼                               │
│  明文资料 ──────────────► 加密 ──────────────► 密文          │
│  {                           │                  │            │
│    name: "张三",             │                  │            │
│    birth_date: "1990-01-01", │                  ▼            │
│    birth_hour: 5,            │            链上存储           │
│    birth_place: "北京市"     │         (EncryptedProfile)    │
│  }                           │                               │
│                              │                               │
│  解密流程：                  │                               │
│  用户私钥 ──► 派生密钥 ──► 解密密文 ──► 明文资料            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**加密算法选择**：
- **密钥派生**：HKDF-SHA256（从签名私钥派生加密密钥）
- **对称加密**：AES-256-GCM（带认证的加密，防篡改）
- **Nonce**：每次更新资料生成新的随机 nonce

### 4.3 数据结构

```rust
/// 会员资料存储
#[pallet::storage]
pub type MemberProfiles<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    MemberProfile<T::BlockNumber>,
    OptionQuery,
>;

/// 会员资料（链上存储）
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MemberProfile<BlockNumber> {
    /// 昵称（明文，用于公开展示）
    pub display_name: BoundedVec<u8, ConstU32<32>>,
    
    /// 加密的敏感资料（姓名、出生信息、地址）
    pub encrypted_data: Option<EncryptedProfileData>,
    
    /// 是否为服务商
    pub is_provider: bool,
    
    /// 服务商认证状态
    pub provider_verified: bool,
    
    /// 资料创建时间
    pub created_at: BlockNumber,
    
    /// 资料更新时间
    pub updated_at: BlockNumber,
}

/// 加密的敏感资料
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EncryptedProfileData {
    /// 加密后的数据（AES-256-GCM 密文）
    /// 明文结构：SensitiveProfile
    /// 最大长度：512 bytes（足够存储加密后的资料）
    pub ciphertext: BoundedVec<u8, ConstU32<512>>,
    
    /// 加密 nonce（12 bytes for AES-GCM）
    pub nonce: [u8; 12],
    
    /// 加密版本（用于未来升级加密算法）
    pub version: u8,
}

/// 敏感资料明文结构（加密前/解密后）
/// 注意：此结构不直接存储在链上，仅用于序列化/反序列化
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SensitiveProfile {
    /// 真实姓名
    pub name: BoundedVec<u8, ConstU32<64>>,
    
    /// 性别
    pub gender: Option<Gender>,
    
    /// 出生日期（公历）
    pub birth_date: BirthDate,
    
    /// 出生时辰（0-23，None 表示未知）
    pub birth_hour: Option<u8>,
    
    /// 出生地址
    pub birth_place: BoundedVec<u8, ConstU32<128>>,
    
    /// 出生地经度（可选，用于精确计算）
    pub longitude: Option<i32>, // 精度：0.0001度，如 1164532 = 116.4532°E
    
    /// 出生地纬度（可选）
    pub latitude: Option<i32>,  // 精度：0.0001度，如 399042 = 39.9042°N
}

/// 性别枚举
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Gender {
    /// 男性
    Male,
    /// 女性
    Female,
    /// 其他/不愿透露
    Other,
}

/// 出生日期
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BirthDate {
    pub year: u16,   // 1900-2100
    pub month: u8,   // 1-12
    pub day: u8,     // 1-31
}
```

### 4.4 加密/解密流程

**客户端加密流程**（前端/钱包实现）：

```typescript
// 1. 从用户私钥派生加密密钥
const encryptionKey = hkdf(
    userPrivateKey,
    salt: "stardust-profile-encryption-v1",
    info: userAccountId,
    length: 32  // AES-256
);

// 2. 构造明文资料
const sensitiveProfile = {
    name: "张三",
    gender: "Male",  // Male / Female / Other / null
    birth_date: { year: 1990, month: 1, day: 15 },
    birth_hour: 5,  // 寅时
    birth_place: "北京市朝阳区",
    longitude: 1164532,  // 116.4532°E
    latitude: 399042,    // 39.9042°N
};

// 3. 序列化（使用 SCALE 编码，与链上一致）
const plaintext = scaleEncode(sensitiveProfile);

// 4. 生成随机 nonce
const nonce = crypto.getRandomValues(new Uint8Array(12));

// 5. AES-256-GCM 加密
const ciphertext = aesGcmEncrypt(encryptionKey, nonce, plaintext);

// 6. 提交到链上
api.tx.membership.updateProfile(
    displayName: "张三",
    encryptedData: { ciphertext, nonce, version: 1 }
);
```

**客户端解密流程**：

```typescript
// 1. 从链上获取加密资料
const profile = await api.query.membership.memberProfiles(accountId);

// 2. 派生解密密钥（与加密时相同）
const decryptionKey = hkdf(userPrivateKey, ...);

// 3. AES-256-GCM 解密
const plaintext = aesGcmDecrypt(
    decryptionKey,
    profile.encryptedData.nonce,
    profile.encryptedData.ciphertext
);

// 4. 反序列化
const sensitiveProfile = scaleDecode(plaintext);
```

### 4.5 Extrinsics

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 创建/更新会员资料
    #[pallet::weight(15_000)]
    pub fn update_profile(
        origin: OriginFor<T>,
        display_name: BoundedVec<u8, ConstU32<32>>,
        encrypted_data: Option<EncryptedProfileData>,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        // 验证加密数据格式
        if let Some(ref data) = encrypted_data {
            ensure!(data.version == 1, Error::<T>::UnsupportedEncryptionVersion);
            ensure!(data.ciphertext.len() >= 16, Error::<T>::InvalidCiphertext); // 至少有 GCM tag
        }
        
        let now = frame_system::Pallet::<T>::block_number();
        
        MemberProfiles::<T>::mutate(&who, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.display_name = display_name;
                profile.encrypted_data = encrypted_data;
                profile.updated_at = now;
            } else {
                *maybe_profile = Some(MemberProfile {
                    display_name,
                    encrypted_data,
                    is_provider: false,
                    provider_verified: false,
                    created_at: now,
                    updated_at: now,
                });
            }
        });
        
        Self::deposit_event(Event::ProfileUpdated { who });
        Ok(())
    }
    
    /// 删除敏感资料（保留昵称）
    #[pallet::weight(10_000)]
    pub fn clear_sensitive_data(origin: OriginFor<T>) -> DispatchResult {
        let who = ensure_signed(origin)?;
        
        MemberProfiles::<T>::mutate(&who, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.encrypted_data = None;
                profile.updated_at = frame_system::Pallet::<T>::block_number();
            }
        });
        
        Self::deposit_event(Event::SensitiveDataCleared { who });
        Ok(())
    }
    
    /// 申请成为服务商
    #[pallet::weight(10_000)]
    pub fn apply_provider(origin: OriginFor<T>) -> DispatchResult;
    
    /// 管理员认证服务商
    #[pallet::weight(10_000)]
    pub fn verify_provider(
        origin: OriginFor<T>,
        provider: T::AccountId,
        verified: bool,
    ) -> DispatchResult;
}
```

### 4.6 与占卜模块集成

**自动填充出生信息**（前端实现）：

```typescript
// 用户创建八字占卜时
async function createBaziChart() {
    // 1. 获取并解密用户资料
    const profile = await getDecryptedProfile();
    
    // 2. 如果有出生信息，自动填充
    if (profile?.birth_date && profile?.birth_hour !== null) {
        const input = {
            year: profile.birth_date.year,
            month: profile.birth_date.month,
            day: profile.birth_date.day,
            hour: profile.birth_hour,
            longitude: profile.longitude,
            latitude: profile.latitude,
        };
        
        // 3. 调用链上创建占卜
        await api.tx.bazi.createBaziChart(name, input);
    } else {
        // 提示用户填写出生信息
        showBirthInfoForm();
    }
}
```

### 4.7 存储成本估算

| 字段 | 大小 | 说明 |
|------|------|------|
| display_name | 36 bytes | 32 + 4 (length prefix) |
| encrypted_data.ciphertext | 516 bytes | 512 + 4 (length prefix) |
| encrypted_data.nonce | 12 bytes | 固定 |
| encrypted_data.version | 1 byte | 固定 |
| is_provider | 1 byte | bool |
| provider_verified | 1 byte | bool |
| created_at | 8 bytes | BlockNumber |
| updated_at | 8 bytes | BlockNumber |
| **总计** | **~583 bytes** | 每用户 |

**10万用户存储成本**：583 × 100,000 = **58.3 MB**

### 4.8 安全性分析

| 安全特性 | 实现方式 |
|---------|---------|
| **数据保密性** | AES-256-GCM 加密，只有持有私钥的用户可解密 |
| **数据完整性** | GCM 模式自带认证标签，防篡改 |
| **前向安全** | 每次更新使用新 nonce，旧密文无法重放 |
| **密钥隔离** | 使用 HKDF 派生专用加密密钥，与签名密钥分离 |
| **版本升级** | version 字段支持未来更换加密算法 |

**风险与缓解**：

| 风险 | 严重性 | 缓解措施 |
|------|-------|---------|
| 私钥泄露导致资料泄露 | 高 | 用户责任，建议使用硬件钱包 |
| 量子计算威胁 | 低（长期） | version 字段支持升级到后量子算法 |
| 链上密文永久存储 | 中 | 提供 clear_sensitive_data 清除功能 |
| 加密实现漏洞 | 中 | 使用成熟的加密库（如 ring/aes-gcm） |

---

## 五、数据存储结构

### 4.1 核心存储项

```rust
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 原生代币（用于支付会员费）
        type Currency: ReservableCurrency<Self::AccountId>;

        /// 会员费接收账户（国库）
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// 积分小数位（1 Point = 10^Decimals）
        #[pallet::constant]
        type PointDecimals: Get<u8>;
    }

    // ============ 存储项 ============

    /// 会员信息
    #[pallet::storage]
    pub type Members<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        MemberInfo<T::BlockNumber>,
        OptionQuery,
    >;

    /// DUST 奖励余额（用于追踪奖励统计）
    #[pallet::storage]
    pub type RewardBalances<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        RewardBalance<T::BlockNumber>,
        ValueQuery,
    >;

    /// DUST 奖励历史（环形缓冲，最近100条）
    #[pallet::storage]
    pub type RewardHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u32, // 索引 (0-99循环)
        RewardTransaction<T::BlockNumber>,
        OptionQuery,
    >;

    /// 每日签到记录
    #[pallet::storage]
    pub type CheckInRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        CheckInRecord<T::BlockNumber>,
        ValueQuery,
    >;

    /// 推荐关系（由独立的 referral pallet 管理，此处仅保留兼容接口）
    /// 实际推荐码存储位于：pallets/referral
    #[pallet::storage]
    #[deprecated(note = "推荐系统已迁移至独立的 referral pallet")]
    pub type ReferralCodes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<16>>, // 邀请码（如 "ABC123"）
        T::AccountId,
        OptionQuery,
    >;

    /// 被邀请人记录（由独立的 referral pallet 管理）
    /// 实际存储位于：pallets/referral
    #[pallet::storage]
    #[deprecated(note = "推荐系统已迁移至独立的 referral pallet")]
    pub type ReferredBy<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        T::AccountId,
        OptionQuery,
    >;

    /// 全局统计
    #[pallet::storage]
    pub type GlobalStats<T: Config> = StorageValue<
        _,
        MembershipStats,
        ValueQuery,
    >;
}

// ============ 数据结构 ============

/// 会员信息
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct MemberInfo<BlockNumber> {
    /// 会员等级
    pub tier: MemberTier,
    /// 到期时间（区块号）
    pub expires_at: BlockNumber,
    /// 订阅开始时间
    pub subscribed_at: BlockNumber,
    /// 累计充值金额（生命周期价值 LTV）
    pub total_paid: u128,
    /// 自动续费
    pub auto_renew: bool,
}

/// 会员等级
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum MemberTier {
    Free = 0,
    Bronze = 1,
    Silver = 2,
    Gold = 3,
    Platinum = 4,
    Diamond = 5,
}

/// DUST 奖励余额（统计用，实际 DUST 在账户余额中）
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardBalance<BlockNumber> {
    /// 累计获得奖励 (DUST)
    pub total_earned: u128,
    /// 今日已获得奖励 (DUST)
    pub today_earned: u128,
    /// 今日日期（天数，自创世区块）
    pub today_date: u32,
    /// 最后更新时间
    pub last_updated: BlockNumber,
}

/// DUST 奖励交易记录
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct RewardTransaction<BlockNumber> {
    /// 交易类型
    pub tx_type: RewardTxType,
    /// 金额 (DUST，精度为链上最小单位)
    pub amount: u128,
    /// 时间戳
    pub timestamp: BlockNumber,
    /// 备注（如 "签到" / "AI解读返现"）
    pub memo: BoundedVec<u8, ConstU32<32>>,
}

#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RewardTxType {
    CheckIn,          // 签到
    Divination,       // 占卜
    AiCashback,       // AI解读返现
    Delete,           // 删除数据
    MarketCashback,   // 市场订单返现
    Review,           // 评价
    Referral,         // 推荐奖励
    NftMint,          // NFT铸造
    NftTrade,         // NFT交易
}

/// 签到记录
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CheckInRecord<BlockNumber> {
    /// 连续签到天数
    pub streak: u32,
    /// 最后签到日期（天数，自创世区块）
    pub last_check_in_day: u32,
    /// 累计签到天数
    pub total_days: u32,
    /// 本周已签到（位图，0-6表示周一到周日）
    pub this_week: u8,
}

/// 全局统计
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[codec(mel_bound())]
pub struct MembershipStats {
    /// 各等级会员数
    pub tier_counts: [u32; 6], // [Free, Bronze, Silver, Gold, Platinum, Diamond]
    /// 累计收入（DUST）
    pub total_revenue: u128,
    /// 累计发放积分
    pub total_points_issued: u128,
    /// 累计消费积分
    pub total_points_redeemed: u128,
}
```

### 4.2 存储成本估算

| 存储项 | 每用户大小 | 1万用户 | 10万用户 |
|-------|-----------|---------|----------|
| Members | 128 bytes | 1.28 MB | 12.8 MB |
| RewardBalances | 48 bytes | 480 KB | 4.8 MB |
| RewardHistory (100条) | 4800 bytes | 48 MB | 480 MB |
| CheckInRecords | 48 bytes | 480 KB | 4.8 MB |
| ReferralCodes | ~~64 bytes~~ | ~~640 KB~~ | ~~6.4 MB~~ | 已迁移至 referral pallet |
| **总计** | ~5.0 KB | **50 MB** | **502 MB** | 不含 referral pallet |

**优化方案**：
- RewardHistory 使用链下索引（Subquery）存储完整历史
- 链上仅保留最近100条用于争议验证
- CheckInRecords 可按需加载（懒加载）
- DUST 奖励直接进入用户账户余额，无需额外存储

---

## 五、与现有模块的集成方案

### 5.1 MembershipProvider Trait

```rust
/// 会员系统接口（供其他模块调用）
pub trait MembershipProvider<AccountId, Balance> {
    /// 获取会员等级
    fn get_tier(who: &AccountId) -> MemberTier;

    /// 检查会员是否有效
    fn is_active_member(who: &AccountId, min_tier: MemberTier) -> bool;

    /// 获取折扣率（返回万分比，如 2000 = 20%折扣）
    fn get_storage_discount(who: &AccountId) -> u32;
    fn get_ai_discount(who: &AccountId) -> u32;

    /// 获取每日免费次数
    fn get_daily_free_quota(who: &AccountId) -> u32;

    /// 获取 DUST 奖励加成倍数（返回万分比，如 12000 = 1.2x）
    fn get_reward_multiplier(who: &AccountId) -> u32;

    /// 发放 DUST 奖励（自动应用会员加成，检查每日上限）
    /// 返回实际发放的 DUST 金额
    fn grant_reward(
        who: &AccountId,
        base_amount: Balance,
        tx_type: RewardTxType,
        memo: &[u8],
    ) -> Result<Balance, DispatchError>;

    /// 检查用户是否满足领取奖励的条件（冷却期、最低余额等）
    fn can_receive_reward(who: &AccountId) -> bool;
}
```

### 5.2 占卜模块集成（以bazi为例）

**修改前**：
```rust
// pallets/divination/bazi/src/lib.rs
#[pallet::weight(T::WeightInfo::create_bazi_chart())]
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, T::MaxNameLen>>,
    input: BaziInputType,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 计算存储押金
    let deposit = Self::calculate_deposit(&chart, &PrivacyMode::Public)?;
    T::Currency::reserve(&who, deposit)?;

    // 存储数据...
    Ok(())
}
```

**修改后**：
```rust
#[pallet::weight(T::WeightInfo::create_bazi_chart())]
pub fn create_bazi_chart(
    origin: OriginFor<T>,
    name: Option<BoundedVec<u8, T::MaxNameLen>>,
    input: BaziInputType,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 计算存储押金（应用会员折扣）
    let base_deposit = Self::calculate_deposit(&chart, &PrivacyMode::Public)?;
    let discount = T::MembershipProvider::get_storage_discount(&who); // 如 2000 = 20%
    let final_deposit = base_deposit.saturating_sub(
        base_deposit.saturating_mul(discount.into()) / 10000
    );
    T::Currency::reserve(&who, final_deposit)?;

    // 发放 DUST 奖励（自动应用会员加成）
    T::MembershipProvider::grant_reward(
        &who,
        5_000_000_000_000, // 0.005 DUST (假设12位精度)
        RewardTxType::Divination,
        b"bazi_chart",
    ).ok();

    // 存储数据...
    Ok(())
}
```

### 5.3 AI模块集成

**修改后**：
```rust
// pallets/divination/ai/src/lib.rs
#[pallet::weight(T::WeightInfo::request_interpretation())]
pub fn request_interpretation(
    origin: OriginFor<T>,
    divination_result_id: u64,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 检查免费额度
    let tier = T::MembershipProvider::get_tier(&who);
    let free_quota = Self::get_monthly_free_quota(&who, tier);
    let used = MonthlyFreeUsage::<T>::get(&who, Self::current_month());

    let fee = if used < free_quota {
        // 使用免费额度
        MonthlyFreeUsage::<T>::insert(&who, Self::current_month(), used + 1);
        0
    } else {
        // 收费（应用会员折扣）
        let base_fee = T::BaseInterpretationFee::get();
        let discount = T::MembershipProvider::get_ai_discount(&who);
        base_fee.saturating_sub(base_fee.saturating_mul(discount.into()) / 10000)
    };

    if fee > 0 {
        T::Currency::transfer(&who, &T::TreasuryAccount::get(), fee, KeepAlive)?;
    }

    // 发放 DUST 返现奖励（20分 → 0.02 DUST）
    T::MembershipProvider::grant_reward(
        &who,
        20_000_000_000_000, // 0.02 DUST
        RewardTxType::AiCashback,
        b"ai_interpretation",
    ).ok();

    // 创建AI请求...
    Ok(())
}

fn get_monthly_free_quota(who: &T::AccountId, tier: MemberTier) -> u32 {
    match tier {
        MemberTier::Gold => 5,
        MemberTier::Platinum => 20,
        MemberTier::Diamond => 50,
        _ => 0,
    }
}
```

### 5.4 Market模块集成

**修改后**：
```rust
// pallets/divination/market/src/lib.rs
fn process_payment(
    provider: &T::AccountId,
    buyer: &T::AccountId,
    amount: BalanceOf<T>,
) -> DispatchResult {
    // 计算平台费率（应用会员折扣）
    let provider_tier = T::MembershipProvider::get_tier(provider);
    let fee_rate = match provider_tier {
        MemberTier::Diamond => 500,  // 5%
        MemberTier::Platinum => 800, // 8%
        MemberTier::Gold => 1000,    // 10%
        _ => 1500,                   // 15%
    };

    let platform_fee = amount.saturating_mul(fee_rate.into()) / 10000;
    let provider_amount = amount.saturating_sub(platform_fee);

    // 转账
    T::Currency::transfer(buyer, &T::PlatformAccount::get(), platform_fee, KeepAlive)?;
    T::Currency::transfer(buyer, provider, provider_amount, KeepAlive)?;

    // 发放 DUST 奖励（买卖双方）
    let reward_amount = amount.saturating_mul(10u128.into()) / 10000; // 0.1%
    T::MembershipProvider::grant_reward(buyer, reward_amount, RewardTxType::MarketCashback, b"buyer").ok();
    T::MembershipProvider::grant_reward(provider, reward_amount * 2, RewardTxType::MarketCashback, b"seller").ok();

    Ok(())
}
```

---

## 六、关键外部接口（Extrinsics）

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 订阅会员
    #[pallet::weight(10_000)]
    pub fn subscribe(
        origin: OriginFor<T>,
        tier: MemberTier,
        duration: SubscriptionDuration, // Monthly / Yearly
        auto_renew: bool,
    ) -> DispatchResult;

    /// 取消订阅（到期后不续费）
    #[pallet::weight(5_000)]
    pub fn cancel_subscription(origin: OriginFor<T>) -> DispatchResult;

    /// 升级会员等级
    #[pallet::weight(8_000)]
    pub fn upgrade_tier(origin: OriginFor<T>, new_tier: MemberTier) -> DispatchResult;

    /// 每日签到
    #[pallet::weight(5_000)]
    pub fn check_in(origin: OriginFor<T>) -> DispatchResult;

    /// 兑换 DUST 奖励物品（保留少量兑换场景）
    #[pallet::weight(10_000)]
    pub fn redeem_reward(
        origin: OriginFor<T>,
        item: RedeemItem, // FreeAI / DepositCoupon 等
        quantity: u32,
    ) -> DispatchResult;

    /// 生成推荐码（已迁移至 referral pallet）
    #[deprecated(note = "请使用 referral pallet 的 create_referral_code")]
    #[pallet::weight(5_000)]
    pub fn create_referral_code(
        origin: OriginFor<T>,
        code: BoundedVec<u8, ConstU32<16>>,
    ) -> DispatchResult;

    /// 绑定推荐人（已迁移至 referral pallet）
    #[deprecated(note = "请使用 referral pallet 的 bind_referrer")]
    #[pallet::weight(5_000)]
    pub fn bind_referrer(
        origin: OriginFor<T>,
        code: BoundedVec<u8, ConstU32<16>>,
    ) -> DispatchResult;

    /// 管理员发放积分
    #[pallet::weight(5_000)]
    pub fn admin_grant_points(
        origin: OriginFor<T>,
        who: T::AccountId,
        amount: u128,
        memo: BoundedVec<u8, ConstU32<32>>,
    ) -> DispatchResult;
}
```

---

## 七、技术可行性评估

### 7.1 技术风险

| 风险项 | 严重性 | 缓解方案 |
|-------|-------|---------|
| **存储膨胀** | 中 | 奖励历史使用链下索引 |
| **签到刷分** | 高 | 需要链上交易 + Gas成本 + 最低余额要求 + 7天冷却期 |
| **推荐滥用** | 中 | 由独立 referral pallet 管理，被邀请人需累计消费≥30 DUST |
| **DUST通胀** | 中 | 奖励来源于平台收入10%，设置年度上限50,000 DUST |
| **会员到期管理** | 低 | 使用区块号而非时间戳 |
| **跨模块依赖** | 中 | 使用Trait抽象接口 |
| **羊毛党攻击** | 高 | 新账户冷却期 + 最低余额 + 每日上限 |

### 7.2 性能评估

**Gas成本估算**：
| 操作 | Weight | Gas (DUST) | 频率 |
|------|--------|-----------|------|
| subscribe | 200,000 | 0.002 | 月/年 |
| check_in | 80,000 | 0.0008 | 每日 |
| grant_reward (内部) | 50,000 | - | 高频 |
| redeem_reward | 100,000 | 0.001 | 周 |

**并发能力**：
- 每区块可处理 **500+** 次签到
- 每区块可处理 **1000+** 次 DUST 奖励发放（内部调用）

### 7.3 向后兼容性

**迁移策略**：
1. **Phase 1（部署）**：
   - 部署membership pallet
   - 所有现有用户自动为Free等级

2. **Phase 2（集成）**：
   - 各占卜模块逐步集成MembershipProvider
   - 保持原有功能不变（折扣=0）

3. **Phase 3（激活）**：
   - Runtime升级，启用折扣机制
   - 开放会员订阅

---

## 八、经济模型分析

### 8.1 收入预测

**假设**：
- 用户基数：10,000人
- 付费转化率：5%（500付费用户）
- 会员分布：Bronze 30% / Silver 40% / Gold 20% / Platinum 8% / Diamond 2%

**月度收入**：
```
Bronze:   150人 × 10 DUST  = 1,500 DUST
Silver:   200人 × 30 DUST  = 6,000 DUST
Gold:     100人 × 80 DUST  = 8,000 DUST
Platinum:  40人 × 200 DUST = 8,000 DUST
Diamond:   10人 × 500 DUST = 5,000 DUST
─────────────────────────────────────
总计：28,500 DUST/月 = 342,000 DUST/年
```

**用户LTV（生命周期价值）**：
- 平均留存周期：12个月
- 平均ARPU：28,500 / 500 = 57 DUST/月
- LTV = 57 × 12 = **684 DUST**

### 8.2 成本分析

**运营成本**：
- 存储成本：670 MB（10万用户）× 0.01 DUST/MB = **6.7 DUST**（一次性）
- 链下索引：云服务器 **50 DUST/月**
- 客服人力：Diamond用户10人 × 5 DUST/人 = **50 DUST/月**
- 营销费用：**100 DUST/月**

**毛利率**：
```
收入：28,500 DUST/月
成本：200 DUST/月
毛利：28,300 DUST/月
毛利率：99.3%
```

### 8.3 用户留存策略

**关键指标**：
- D7留存：目标 **40%**（签到奖励）
- M1留存：目标 **25%**（免费AI额度锁定）
- M3留存：目标 **15%**（会员权益体验）
- 付费续费率：目标 **70%**（年费折扣）

**增长飞轮**：
```
签到奖励 → 积分累积 → 兑换免费AI → 体验价值 → 订阅会员
    ↑                                              ↓
    └────────── 推荐好友奖励 ←─────────────────────┘
```

---

## 九、实施路线图

### Phase 1：核心功能（2周）
- [ ] 实现membership pallet基础框架
- [ ] 实现MembershipProvider Trait
- [ ] 实现订阅、升级、取消功能
- [ ] 单元测试覆盖率 >80%

### Phase 2：DUST 奖励系统（1周）
- [ ] 实现 DUST 奖励发放逻辑
- [ ] 实现签到系统（含防刷机制）
- [ ] 实现奖励历史记录
- [ ] 集成事件监听（自动发放奖励）

### Phase 3：模块集成（2周）
- [ ] bazi/qimen/liuyao等占卜模块集成
- [ ] ai模块集成（免费额度+折扣）
- [ ] market模块集成（费率优惠）
- [ ] 集成测试

### Phase 4：推荐系统集成（1周）
- [ ] 集成独立的 referral pallet（位于 `/pallets/referral`）
- [ ] 实现 membership 与 referral 的跨模块调用
- [ ] 会员 DUST 奖励与推荐奖励联动

### Phase 5：链下服务（1周）
- [ ] Subquery索引服务（积分历史）
- [ ] 会员数据统计API
- [ ] 前端SDK

### Phase 6：上线准备（1周）
- [ ] 审计与安全测试
- [ ] 文档编写
- [ ] Runtime升级方案
- [ ] 灰度发布

**总工期：8周**

---

## 十、总结与建议

### 10.1 方案优势

✅ **用户价值明确**：重度用户可节省30%-50%成本
✅ **收入稳定可预测**：订阅制提供稳定现金流
✅ **技术实现简单**：无复杂依赖，8周可上线
✅ **可扩展性强**：通过Trait接口，新模块零改动接入
✅ **防滥用机制完善**：签到需交易、推荐有门槛、新账户冷却期
✅ **经济模型健康**：99%毛利率，DUST奖励来源于平台收入
✅ **激励效果强**：直接发放 DUST，用户感知明确

### 10.2 核心建议

1. **优先级调整**：
   - **P0（必须）**：订阅系统 + 积分系统 + bazi/ai模块集成
   - **P1（重要）**：签到系统 + 兑换商城 + market模块集成
   - **P2（可选）**：referral pallet集成 + 链下索引

2. **Bronze等级重新设计**：
   ```
   修改前：月费10 DUST，押金折扣20%，AI折扣10%
   修改后：月费10 DUST，押金折扣30%，AI折扣15%，免费AI 1次
   ```

3. **Silver等级免费次数增加**：
   ```
   修改前：1次免费AI/月
   修改后：3次免费AI/月
   ```

4. **~~积分有效期优化~~**（已取消积分系统）：
   ```
   改用 DUST 直接奖励，无需管理积分有效期
   ```

5. **增加"体验会员"机制**：
   ```
   新用户注册即赠送 7天Bronze会员
   → 体验押金折扣和免费AI
   → 提升付费转化率
   ```

### 10.3 风险提示

⚠️ **市场风险**：用户付费意愿需要实际验证，建议先做MVP测试
⚠️ **竞争风险**：其他占卜平台可能跟进会员制，需要差异化运营
⚠️ **监管风险**：会员订阅可能涉及预付费监管，需法律合规审查
⚠️ **DUST通胀风险**：奖励发放需严格控制，建议设置年度上限和动态调整机制
⚠️ **羊毛党风险**：直接发放 DUST 会吸引刷号，需要持续监控和调整防刷策略

### 10.4 下一步行动

1. 用户调研：问卷调查愿付价格区间
2. MVP开发：先实现最小功能集（订阅+积分+1个模块集成）
3. A/B测试：对比有/无会员系统的转化率和留存率
4. 迭代优化：根据真实数据调整等级定价和权益

---

**文档版本**：v1.3
**最后更新**：2026-01-01
**作者**：Claude Code (Anthropic)
**状态**：设计方案待评审
**变更记录**：
- v1.0：初始设计（积分系统）
- v1.1：推荐系统迁移至独立 referral pallet
- v1.2：取消积分系统，改用 DUST 直接奖励（方案A）
- v1.3：SensitiveProfile 添加 gender（性别）字段 v1.2：取消积分系统，改用 DUST 直接奖励（方案A）

---

## 附录：Referral Pallet 集成说明

### A.1 架构关系

推荐系统已独立为单独的 pallet，位于 `pallets/referral`，与 membership pallet 通过 trait 接口交互：

```
┌─────────────────────────────────────────────────────────┐
│                    Runtime                               │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────┐      ┌──────────────────┐         │
│  │  Membership      │      │  Referral        │         │
│  │  Pallet          │◄────►│  Pallet          │         │
│  │                  │      │                  │         │
│  │  - 订阅管理      │      │  - 推荐码管理    │         │
│  │  - 积分系统      │      │  - 邀请关系      │         │
│  │  - 权益引擎      │      │  - 奖励发放      │         │
│  └────────┬─────────┘      └────────┬─────────┘         │
│           │                         │                    │
│           └─────────┬───────────────┘                    │
│                     │                                    │
│           ┌─────────▼─────────┐                          │
│           │  ReferralProvider │                          │
│           │  (Trait Interface)│                          │
│           └───────────────────┘                          │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### A.2 ReferralProvider Trait

```rust
/// 推荐系统接口（供 membership pallet 调用）
pub trait ReferralProvider<AccountId> {
    /// 获取用户的推荐人
    fn get_referrer(who: &AccountId) -> Option<AccountId>;

    /// 检查用户是否有有效推荐码
    fn has_referral_code(who: &AccountId) -> bool;

    /// 通知推荐系统：被邀请人完成了付费行为
    /// 返回是否触发了推荐奖励
    fn notify_referral_payment(
        referee: &AccountId,
        amount: u128,
    ) -> Result<bool, DispatchError>;

    /// 获取用户的推荐统计
    fn get_referral_stats(who: &AccountId) -> ReferralStats;
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ReferralStats {
    /// 直接邀请人数
    pub direct_referrals: u32,
    /// 累计获得的推荐奖励
    pub total_rewards: u128,
    /// 待发放奖励（被邀请人未达消费门槛）
    pub pending_rewards: u128,
}
```

### A.3 Membership 与 Referral 的交互流程

**场景：用户A邀请用户B，用户B订阅会员**

```
1. 用户B 调用 referral::bind_referrer(code_of_A)
   → referral pallet 记录 B 的推荐人为 A

2. 用户B 调用 membership::subscribe(Silver, Monthly)
   → membership pallet 收取会员费 30 DUST
   → membership pallet 调用 referral::notify_referral_payment(B, 30)
   → referral pallet 检查 B 的累计消费是否 ≥ 30 DUST
   → 如果达标，referral pallet 发放奖励给 A
   → membership pallet 调用 add_points(A, 100, Referral, "invite_B")
```

### A.4 配置示例

```rust
// runtime/src/lib.rs
impl pallet_membership::Config for Runtime {
    // ...
    type ReferralProvider = pallet_referral::Pallet<Runtime>;
}

impl pallet_referral::Config for Runtime {
    // ...
    type MembershipProvider = pallet_membership::Pallet<Runtime>;
}
```

### A.5 迁移注意事项

- membership pallet 中的 `ReferralCodes` 和 `ReferredBy` 存储项已标记为 `#[deprecated]`
- 新部署应直接使用 referral pallet
- 现有数据迁移需要编写 migration 脚本，将数据从 membership 迁移到 referral
