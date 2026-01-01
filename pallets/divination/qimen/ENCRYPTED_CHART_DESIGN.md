# 奇门遁甲加密命盘接口设计

## 一、设计目标

### 核心需求
1. **隐私保护**：敏感信息（姓名、完整出生时间、问题明文）加密存储
2. **功能完整**：加密后仍可进行解盘分析
3. **授权管理**：支持多方授权访问（如大师解盘）
4. **用户控制**：用户完全掌控密钥和授权

### 参考方案
- 八字加密方案：`EncryptedBaziChart` 和 `MultiKeyEncryptedBaziChart`
- 梅花易数：使用问题哈希 + 前端本地存储

---

## 二、数据分类与隐私分析

### 2.1 核心问题：数据关联性

**重要认识**：四柱干支与公历日期存在直接对应关系，加密公历日期但保留四柱明文是无效的隐私保护。

```
公历日期时间 ←→ 四柱干支 ←→ 节气 ←→ 局数
     ↓              ↓          ↓       ↓
  可反推         可反推      可反推   可反推
```

**隐私泄露路径**：
- 从四柱干支可反推出生日期（精确到2小时内）
- 从节气+三元可反推月份和旬
- 从局数+遁型可反推节气范围

### 2.2 三种加密模式设计

根据"加密即不可计算"的原则，设计三种模式：

#### 模式 A：完全公开（Public）

**适用场景**：教学演示、公开案例分享

| 数据类别 | 存储方式 | 说明 |
|----------|----------|------|
| 所有排盘数据 | 明文 | 支持链上解盘 |
| 命主信息 | 明文 | 任何人可见 |
| 问题内容 | 明文 | 任何人可见 |

**特点**：
- ✅ 支持 Runtime API 链上解盘
- ✅ 支持公开查询和分析
- ❌ 无隐私保护

#### 模式 B：仅加密敏感文本（PartialEncrypted）⭐推荐

**适用场景**：需要链上解盘，同时保护姓名和问题内容（最佳平衡方案）

| 数据类别 | 存储方式 | 说明 |
|----------|----------|------|
| 四柱干支、节气、局数 | 明文 | 支持链上解盘 |
| 九宫排盘结果 | 明文 | 支持链上解盘 |
| 姓名、问题文本 | 加密 | 授权后可见 |
| 公历日期时间 | **不存储** | 避免冗余暴露 |

**特点**：
- ✅ 支持 Runtime API 链上解盘
- ✅ 支持格局检测、用神分析、应期推算
- ⚠️ 四柱干支明文，**出生时间可被反推**（约2小时精度）
- ✅ 姓名和问题内容受保护
- ✅ 功能与隐私的最佳平衡

**隐私声明**：此模式下，有经验的命理师可以从四柱干支反推出大致的出生时间。

#### 模式 C：完全加密（FullyEncrypted）

**适用场景**：最高隐私保护需求（牺牲链上计算能力）

| 数据类别 | 存储方式 | 说明 |
|----------|----------|------|
| chart_id, owner | 明文 | 基础索引 |
| encryption_level | 明文 | 加密级别标识 |
| **所有排盘数据** | **加密** | 四柱、九宫、局数全部加密 |
| 命主信息 | 加密 | 姓名、问题等 |
| question_hash | 明文 | 仅用于验证 |

**特点**：
- ❌ 不支持链上解盘（需前端解密后计算）
- ✅ 完整隐私保护，无法反推任何时间信息
- ✅ 授权查看者可完整解密

### 2.3 数据存储对照表

| 字段 | Public | Partial | Full | 说明 |
|------|--------|---------|------|------|
| `chart_id` | 明文 | 明文 | 明文 | 必需索引 |
| `owner` | 明文 | 明文 | 明文 | 所有者 |
| `encryption_level` | 明文 | 明文 | 明文 | 加密标识 |
| `question_hash` | 明文 | 明文 | 明文 | 验证用 |
| `year_ganzhi` | 明文 | 明文 | **加密** | |
| `month_ganzhi` | 明文 | 明文 | **加密** | |
| `day_ganzhi` | 明文 | 明文 | **加密** | |
| `hour_ganzhi` | 明文 | 明文 | **加密** | |
| `jie_qi` | 明文 | 明文 | **加密** | |
| `dun_type` | 明文 | 明文 | **加密** | |
| `san_yuan` | 明文 | 明文 | **加密** | |
| `ju_number` | 明文 | 明文 | **加密** | |
| `palaces[9]` | 明文 | 明文 | **加密** | |
| `zhi_fu_xing` | 明文 | 明文 | **加密** | |
| `zhi_shi_men` | 明文 | 明文 | **加密** | |
| `name` | 明文 | **加密** | **加密** | |
| `question` | 明文 | **加密** | **加密** | |
| `gender` | 明文 | 明文 | **加密** | |
| `birth_year` | 明文 | 明文 | **加密** | |
| `question_type` | 明文 | 明文 | **加密** | |

### 2.4 解盘能力对比

| 功能 | Public | Partial ⭐ | Full |
|------|--------|---------|------|
| 链上 Runtime API 解盘 | ✅ | ✅ | ❌ |
| 前端解盘 | ✅ | ✅ | ✅（解密后） |
| 公开浏览排盘结果 | ✅ | ✅ | ❌ |
| 格局检测（九遁、六仪击刑等） | ✅ | ✅ | ❌ |
| 用神分析 | ✅ | ✅ | ❌ |
| 应期推算 | ✅ | ✅ | ❌ |
| AI 链上解读 | ✅ | ✅ | ❌ |
| 隐私保护级别 | 无 | 中（推荐） | **高** |
| 姓名/问题保护 | ❌ | ✅ | ✅ |
| 出生时间保护 | ❌ | ❌（可反推） | ✅ |

### 2.5 推荐使用场景

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        加密模式选择指南                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  需要公开分享？──────Yes────→ 【Public】完全公开                          │
│        │                                                                 │
│        No                                                                │
│        ↓                                                                 │
│  需要最高隐私保护？──Yes────→ 【Full】完全加密                            │
│        │                      ⚠️ 注意：不支持链上解盘API                   │
│        No                                                                │
│        ↓                                                                 │
│  【Partial】部分加密 ⭐推荐                                              │
│  • 支持完整链上解盘API                                                   │
│  • 格局检测、用神分析、应期推算                                           │
│  • 姓名和问题内容加密保护                                                 │
│  • 功能与隐私的最佳平衡                                                   │
│  • ⚠️ 出生时间可从四柱反推（约2小时精度）                                 │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.6 哈希存储（所有模式通用）

| 字段 | 类型 | 用途 |
|------|------|------|
| `question_hash` | `[u8; 32]` | 问题验证（不泄露内容） |
| `data_hash` | `[u8; 32]` | 加密数据完整性验证 |

---

## 三、技术架构

### 3.1 分层混合加密设计

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      奇门遁甲加密命盘架构                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Layer 1: 公开元数据（明文）                       │ │
│  │  chart_id, diviner, dun_type, ju_number, palaces, timestamp        │ │
│  │  → 链上存储，任何人可查询                                            │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                  │                                       │
│                                  ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Layer 2: 加密敏感数据                             │ │
│  │  name, question, solar_date, solar_time, notes                     │ │
│  │  → AES-256-GCM 加密，链上存储密文                                    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                  │                                       │
│                                  ▼                                       │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Layer 3: 密钥管理                                 │ │
│  │  • 主密钥：用户本地生成，永不上链                                     │ │
│  │  • 授权密钥：X25519 密钥交换生成包装密钥                              │ │
│  │  • 链上存储：包装后的会话密钥 + 授权列表                              │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 加密算法选择

| 用途 | 算法 | 说明 |
|------|------|------|
| 数据加密 | AES-256-GCM | 对称加密，认证加密模式 |
| 密钥交换 | X25519 | 与 Substrate sr25519 兼容 |
| 哈希 | Blake2b-256 | Substrate 原生支持 |
| 密钥派生 | HKDF-SHA256 | 从主密钥派生会话密钥 |

---

## 四、数据结构设计

### 4.1 加密级别枚举

```rust
/// 加密级别
///
/// 定义命盘数据的加密程度
/// 默认使用 PartialEncrypted（部分加密），兼顾链上解盘能力和隐私保护
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum EncryptionLevel {
    /// 完全公开（不加密）
    /// 所有数据明文存储，任何人可查看
    Public = 0,

    /// 部分加密（推荐）⭐
    /// 排盘数据明文（支持链上解盘），敏感信息加密
    /// 优点：支持完整的链上解盘API、格局检测、用神分析
    /// 缺点：出生时间可从四柱干支反推（约2小时精度）
    PartialEncrypted = 1,

    /// 完全加密
    /// 除基本元数据外全部加密，需授权才能查看
    /// 优点：最高隐私保护
    /// 缺点：不支持链上解盘API，需前端解密后计算
    FullyEncrypted = 2,
}

impl Default for EncryptionLevel {
    fn default() -> Self {
        Self::PartialEncrypted  // 默认使用部分加密（推荐）
    }
}
```

### 4.2 公开元数据结构

```rust
/// 公开元数据（不加密）
///
/// 包含排盘计算所需的所有数据，以及基本的查询信息
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PublicMetadata<AccountId, BlockNumber> {
    /// 排盘 ID
    pub chart_id: u64,

    /// 创建者账户
    pub owner: AccountId,

    /// 加密级别
    pub encryption_level: EncryptionLevel,

    /// 起局方式
    pub method: DivinationMethod,

    /// 排盘方法（转盘/飞盘）
    pub pan_method: PanMethod,

    // ========== 排盘计算必需数据 ==========

    /// 四柱干支
    pub year_ganzhi: GanZhi,
    pub month_ganzhi: GanZhi,
    pub day_ganzhi: GanZhi,
    pub hour_ganzhi: GanZhi,

    /// 节气
    pub jie_qi: JieQi,

    /// 阴阳遁
    pub dun_type: DunType,

    /// 三元
    pub san_yuan: SanYuan,

    /// 局数（1-9）
    pub ju_number: u8,

    /// 值符星
    pub zhi_fu_xing: JiuXing,

    /// 值使门
    pub zhi_shi_men: BaMen,

    /// 九宫排盘结果
    pub palaces: [Palace; 9],

    // ========== 解卦辅助数据（可选明文） ==========

    /// 性别（用于年命分析）
    pub gender: Option<Gender>,

    /// 出生年份（用于年命分析）
    pub birth_year: Option<u16>,

    /// 问事类型（用于用神确定）
    pub question_type: Option<QuestionType>,

    // ========== 元数据 ==========

    /// 创建时间戳
    pub timestamp: u64,

    /// 创建区块号
    pub block_number: BlockNumber,

    /// 是否公开可见
    pub is_public: bool,

    /// 问题哈希（用于验证）
    pub question_hash: [u8; 32],
}
```

### 4.3 加密数据结构

```rust
/// 加密的敏感数据
///
/// 使用 AES-256-GCM 加密，链上仅存储密文
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxEncryptedLen))]
pub struct EncryptedData<MaxEncryptedLen: Get<u32>> {
    /// 加密后的数据（包含 nonce + ciphertext + tag）
    ///
    /// 格式：[nonce: 12 bytes][ciphertext: variable][tag: 16 bytes]
    pub ciphertext: BoundedVec<u8, MaxEncryptedLen>,

    /// 数据完整性哈希（Blake2b-256）
    ///
    /// 用于验证解密后数据的完整性
    pub data_hash: [u8; 32],

    /// 加密时间戳
    pub encrypted_at: u64,

    /// 算法版本（用于未来升级）
    pub algorithm_version: u8,
}

/// 敏感数据明文结构（用于前端序列化）
///
/// 此结构用于前端加密前的数据组织
#[derive(Clone, Debug, Encode, Decode)]
pub struct SensitiveDataPlaintext {
    /// 命主姓名
    pub name: Option<Vec<u8>>,

    /// 占问事宜
    pub question: Option<Vec<u8>>,

    /// 公历日期（年月日）
    pub solar_date: Option<(u16, u8, u8)>,

    /// 公历时间（时分秒）
    pub solar_time: Option<(u8, u8, u8)>,

    /// 备注信息
    pub notes: Option<Vec<u8>>,
}
```

### 4.4 授权密钥结构

```rust
/// 查看权限级别
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[repr(u8)]
pub enum ViewPermission {
    /// 仅查看排盘数据（公开元数据）
    ChartOnly = 0,

    /// 查看排盘和解卦
    ChartAndInterpretation = 1,

    /// 完整访问（包含所有敏感数据）
    FullAccess = 2,
}

/// 加密的授权密钥
///
/// 使用查看者的公钥加密会话密钥
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct EncryptedKey<AccountId> {
    /// 被授权者账户
    pub viewer: AccountId,

    /// 包装后的会话密钥（X25519 加密）
    ///
    /// 格式：[ephemeral_pubkey: 32 bytes][encrypted_key: 32 bytes + tag]
    pub wrapped_key: [u8; 80],

    /// 授权时间
    pub authorized_at: u64,

    /// 过期时间（0 表示永不过期）
    pub expires_at: u64,

    /// 权限级别
    pub permission: ViewPermission,
}

/// 授权列表
pub type AuthorizationList<AccountId, MaxViewers> =
    BoundedVec<EncryptedKey<AccountId>, MaxViewers>;
```

### 4.5 完整加密命盘结构

```rust
/// 加密奇门命盘
///
/// 完整的加密命盘存储结构
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxEncryptedLen, MaxViewers, MaxCidLen))]
pub struct EncryptedQimenChart<
    AccountId,
    BlockNumber,
    MaxEncryptedLen: Get<u32>,
    MaxViewers: Get<u32>,
    MaxCidLen: Get<u32>,
> {
    /// 公开元数据
    pub metadata: PublicMetadata<AccountId, BlockNumber>,

    /// 加密的敏感数据（可选）
    ///
    /// 当 encryption_level 为 Public 时为 None
    pub encrypted_data: Option<EncryptedData<MaxEncryptedLen>>,

    /// 授权列表
    ///
    /// 存储所有被授权查看者的包装密钥
    pub authorizations: AuthorizationList<AccountId, MaxViewers>,

    /// AI 解读 CID（可选）
    pub interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 所有者的加密密钥备份
    ///
    /// 使用所有者公钥加密的主密钥备份
    pub owner_key_backup: Option<[u8; 80]>,
}
```

---

## 五、Pallet 接口设计

### 5.1 Extrinsics（可调用函数）

#### 5.1.1 统一创建接口

**设计原则**：Public 和 Partial 模式使用同一个接口，通过参数区分。

| 参数 | Public | Partial ⭐ | Full |
|------|--------|---------|------|
| `encryption_level` | `0` | `1` | `2` |
| `encrypted_data` | `None` | `Some(加密的姓名+问题)` | `Some(加密的全部数据)` |
| `owner_key_backup` | `None` | `Some(密钥备份)` | `Some(密钥备份)` |
| 公开元数据参数 | ✅ 必传 | ✅ 必传 | ⚠️ 可忽略（被加密覆盖） |

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 创建命盘（支持三种加密模式）
    ///
    /// 统一接口，通过 encryption_level 参数区分：
    /// - Public (0): 完全公开，encrypted_data = None
    /// - Partial (1): 部分加密（推荐），encrypted_data = 加密的姓名+问题
    /// - Full (2): 完全加密，encrypted_data = 加密的全部数据
    ///
    /// # 参数
    /// - `origin`: 调用者
    /// - 公开元数据: 四柱干支、节气等（Public/Partial 必需，Full 可选）
    /// - `encryption_level`: 加密级别 (0/1/2)
    /// - `encrypted_data`: 加密的敏感数据（Partial/Full 必需）
    /// - `data_hash`: 数据哈希（用于完整性验证）
    /// - `owner_key_backup`: 所有者密钥备份（Partial/Full 必需）
    ///
    /// # 事件
    /// - `ChartCreated`: 命盘创建成功
    #[pallet::call_index(10)]
    #[pallet::weight(Weight::from_parts(100_000_000, 0))]
    pub fn create_chart(
        origin: OriginFor<T>,
        // 公开元数据（Public/Partial 必传，Full 可选）
        year_ganzhi: (u8, u8),
        month_ganzhi: (u8, u8),
        day_ganzhi: (u8, u8),
        hour_ganzhi: (u8, u8),
        jie_qi: u8,
        day_in_jieqi: u8,
        // 加密相关
        encryption_level: u8,                                    // 0=Public, 1=Partial, 2=Full
        encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>, // Partial/Full 必需
        data_hash: Option<[u8; 32]>,                             // Partial/Full 必需
        owner_key_backup: Option<[u8; 80]>,                      // Partial/Full 必需
        // 其他参数
        question_hash: [u8; 32],
        is_public: bool,
        gender: Option<u8>,
        birth_year: Option<u16>,
        question_type: Option<u8>,
        pan_method: u8,
    ) -> DispatchResult;

    /// 公历时间起局（支持三种加密模式）
    ///
    /// 结合公历时间起局和加密功能，自动转换为四柱干支。
    /// 通过 encryption_level 参数区分加密模式。
    ///
    /// # 参数
    /// - 公历时间参数: year, month, day, hour
    /// - 加密参数: encryption_level, encrypted_data, data_hash, owner_key_backup
    /// - 命主信息参数
    #[pallet::call_index(11)]
    #[pallet::weight(Weight::from_parts(150_000_000, 0))]
    pub fn divine_by_solar_time(
        origin: OriginFor<T>,
        solar_year: u16,
        solar_month: u8,
        solar_day: u8,
        hour: u8,
        // 加密相关（同 create_chart）
        encryption_level: u8,
        encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>,
        data_hash: Option<[u8; 32]>,
        owner_key_backup: Option<[u8; 80]>,
        // 其他参数
        question_hash: [u8; 32],
        is_public: bool,
        gender: Option<u8>,
        birth_year: Option<u16>,
        question_type: Option<u8>,
        pan_method: u8,
    ) -> DispatchResult;
```

#### 5.1.2 授权管理接口

```rust
    /// 授权查看者（仅 Partial/Full 模式需要）
    /// 将加密的会话密钥授权给指定用户。
    /// 密钥包装在前端完成。
    ///
    /// # 参数
    /// - `origin`: 调用者（必须是命盘所有者）
    /// - `chart_id`: 命盘 ID
    /// - `viewer`: 被授权者账户
    /// - `wrapped_key`: 包装后的密钥
    /// - `permission`: 权限级别
    /// - `expires_at`: 过期时间（0=永不过期）
    ///
    /// # 事件
    /// - `ViewerAuthorized`: 查看者授权成功
    #[pallet::call_index(12)]
    #[pallet::weight(Weight::from_parts(50_000_000, 0))]
    pub fn authorize_viewer(
        origin: OriginFor<T>,
        chart_id: u64,
        viewer: T::AccountId,
        wrapped_key: [u8; 80],
        permission: u8,
        expires_at: u64,
    ) -> DispatchResult;

    /// 撤销授权
    ///
    /// 撤销指定用户的查看权限。
    ///
    /// # 参数
    /// - `origin`: 调用者（必须是命盘所有者）
    /// - `chart_id`: 命盘 ID
    /// - `viewer`: 被撤销者账户
    ///
    /// # 事件
    /// - `AuthorizationRevoked`: 授权已撤销
    #[pallet::call_index(13)]
    #[pallet::weight(Weight::from_parts(30_000_000, 0))]
    pub fn revoke_authorization(
        origin: OriginFor<T>,
        chart_id: u64,
        viewer: T::AccountId,
    ) -> DispatchResult;

    /// 更新加密数据
    ///
    /// 更新命盘的加密敏感数据。
    ///
    /// # 参数
    /// - `origin`: 调用者（必须是命盘所有者）
    /// - `chart_id`: 命盘 ID
    /// - `new_encrypted_data`: 新的加密数据
    /// - `new_data_hash`: 新数据哈希
    ///
    /// # 事件
    /// - `EncryptedDataUpdated`: 加密数据已更新
    #[pallet::call_index(14)]
    #[pallet::weight(Weight::from_parts(60_000_000, 0))]
    pub fn update_encrypted_data(
        origin: OriginFor<T>,
        chart_id: u64,
        new_encrypted_data: BoundedVec<u8, T::MaxEncryptedLen>,
        new_data_hash: [u8; 32],
    ) -> DispatchResult;

    /// 转移所有权
    ///
    /// 将命盘所有权转移给新用户。
    /// 需要同时提供新所有者的密钥备份。
    ///
    /// # 参数
    /// - `origin`: 调用者（必须是当前所有者）
    /// - `chart_id`: 命盘 ID
    /// - `new_owner`: 新所有者账户
    /// - `new_owner_key_backup`: 新所有者的密钥备份
    ///
    /// # 事件
    /// - `OwnershipTransferred`: 所有权已转移
    #[pallet::call_index(15)]
    #[pallet::weight(Weight::from_parts(40_000_000, 0))]
    pub fn transfer_ownership(
        origin: OriginFor<T>,
        chart_id: u64,
        new_owner: T::AccountId,
        new_owner_key_backup: [u8; 80],
    ) -> DispatchResult;

    /// 删除加密命盘
    ///
    /// 永久删除命盘及其所有关联数据。
    ///
    /// # 参数
    /// - `origin`: 调用者（必须是命盘所有者）
    /// - `chart_id`: 命盘 ID
    ///
    /// # 事件
    /// - `ChartDeleted`: 命盘已删除
    #[pallet::call_index(16)]
    #[pallet::weight(Weight::from_parts(40_000_000, 0))]
    pub fn delete_encrypted_chart(
        origin: OriginFor<T>,
        chart_id: u64,
    ) -> DispatchResult;

    /// 批量授权
    ///
    /// 一次性授权多个查看者。
    ///
    /// # 参数
    /// - `origin`: 调用者（必须是命盘所有者）
    /// - `chart_id`: 命盘 ID
    /// - `authorizations`: 授权列表
    ///
    /// # 事件
    /// - `BatchAuthorizationCompleted`: 批量授权完成
    #[pallet::call_index(17)]
    #[pallet::weight(Weight::from_parts(100_000_000, 0))]
    pub fn batch_authorize(
        origin: OriginFor<T>,
        chart_id: u64,
        authorizations: BoundedVec<(T::AccountId, [u8; 80], u8, u64), ConstU32<10>>,
    ) -> DispatchResult;
}
```

### 5.2 事件定义

```rust
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// 命盘创建成功（适用于所有加密模式）
    ChartCreated {
        chart_id: u64,
        owner: T::AccountId,
        encryption_level: EncryptionLevel,
        dun_type: DunType,
        ju_number: u8,
    },

    /// 查看者授权成功
    ViewerAuthorized {
        chart_id: u64,
        owner: T::AccountId,
        viewer: T::AccountId,
        permission: ViewPermission,
        expires_at: u64,
    },

    /// 授权已撤销
    AuthorizationRevoked {
        chart_id: u64,
        owner: T::AccountId,
        viewer: T::AccountId,
    },

    /// 加密数据已更新
    EncryptedDataUpdated {
        chart_id: u64,
        owner: T::AccountId,
        updated_at: u64,
    },

    /// 所有权已转移
    OwnershipTransferred {
        chart_id: u64,
        from: T::AccountId,
        to: T::AccountId,
    },

    /// 加密命盘已删除
    ChartDeleted {
        chart_id: u64,
        owner: T::AccountId,
    },

    /// 批量授权完成
    BatchAuthorizationCompleted {
        chart_id: u64,
        owner: T::AccountId,
        count: u32,
    },
}
```

### 5.3 错误定义

```rust
#[pallet::error]
pub enum Error<T> {
    /// 命盘不存在
    ChartNotFound,

    /// 非命盘所有者
    NotOwner,

    /// 无查看权限
    NotAuthorized,

    /// 授权已过期
    AuthorizationExpired,

    /// 授权列表已满
    AuthorizationListFull,

    /// 无效的加密级别
    InvalidEncryptionLevel,

    /// 无效的权限级别
    InvalidPermission,

    /// 加密数据过大
    EncryptedDataTooLarge,

    /// 数据哈希不匹配
    DataHashMismatch,

    /// 该用户已被授权
    AlreadyAuthorized,

    /// 该用户未被授权
    NotInAuthorizationList,

    /// 无效的密钥备份
    InvalidKeyBackup,

    /// 加密数据缺失
    EncryptedDataMissing,

    /// 批量授权数量超限
    BatchAuthorizationTooMany,
}
```

---

## 六、Runtime API 扩展

### 6.1 API 定义

```rust
sp_api::decl_runtime_apis! {
    /// 奇门遁甲加密命盘 Runtime API
    pub trait QimenEncryptedApi<AccountId, BlockNumber>
    where
        AccountId: codec::Codec,
        BlockNumber: codec::Codec,
    {
        /// 获取加密命盘公开元数据
        ///
        /// 任何人都可以查询公开元数据
        fn get_public_metadata(chart_id: u64) -> Option<PublicMetadata<AccountId, BlockNumber>>;

        /// 获取加密数据（需要链下验证权限）
        ///
        /// 返回加密的密文，由前端解密
        fn get_encrypted_data(chart_id: u64) -> Option<EncryptedData>;

        /// 获取用户的包装密钥
        ///
        /// 返回指定用户的包装密钥（如果已授权）
        fn get_wrapped_key(chart_id: u64, viewer: AccountId) -> Option<WrappedKeyInfo>;

        /// 检查用户是否有权限
        ///
        /// 检查用户是否被授权查看指定命盘
        fn check_authorization(chart_id: u64, viewer: AccountId) -> Option<ViewPermission>;

        /// 获取用户的所有加密命盘
        ///
        /// 返回用户拥有的所有加密命盘 ID 列表
        fn get_user_encrypted_charts(owner: AccountId) -> Vec<u64>;

        /// 获取用户被授权查看的命盘
        ///
        /// 返回用户被授权查看的所有命盘 ID 列表
        fn get_authorized_charts(viewer: AccountId) -> Vec<u64>;

        /// 获取加密命盘的核心解卦
        ///
        /// 基于公开元数据计算核心解卦（不需要解密）
        fn get_encrypted_chart_interpretation(
            chart_id: u64,
            question_type: QuestionType,
        ) -> Option<QimenCoreInterpretation>;
    }
}
```

### 6.2 包装密钥信息结构

```rust
/// 包装密钥信息（用于 API 返回）
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct WrappedKeyInfo {
    /// 包装后的密钥
    pub wrapped_key: [u8; 80],

    /// 授权时间
    pub authorized_at: u64,

    /// 过期时间
    pub expires_at: u64,

    /// 权限级别
    pub permission: ViewPermission,

    /// 是否已过期
    pub is_expired: bool,
}
```

---

## 七、存储设计

### 7.1 存储项

```rust
/// 加密命盘存储
#[pallet::storage]
#[pallet::getter(fn encrypted_charts)]
pub type EncryptedCharts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // chart_id
    EncryptedQimenChart<T::AccountId, BlockNumberFor<T>, T::MaxEncryptedLen, T::MaxViewers, T::MaxCidLen>,
>;

/// 用户加密命盘索引
#[pallet::storage]
#[pallet::getter(fn user_encrypted_charts)]
pub type UserEncryptedCharts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, T::MaxUserCharts>,
    ValueQuery,
>;

/// 被授权查看的命盘索引
///
/// 记录每个用户被授权查看的命盘列表
#[pallet::storage]
#[pallet::getter(fn authorized_charts)]
pub type AuthorizedCharts<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, T::MaxAuthorizedCharts>,
    ValueQuery,
>;
```

### 7.2 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    // ... 现有配置 ...

    /// 加密数据最大长度（默认: 512 bytes）
    #[pallet::constant]
    type MaxEncryptedLen: Get<u32>;

    /// 每个命盘最大授权人数（默认: 20）
    #[pallet::constant]
    type MaxViewers: Get<u32>;

    /// 用户被授权查看的最大命盘数（默认: 100）
    #[pallet::constant]
    type MaxAuthorizedCharts: Get<u32>;
}
```

---

## 八、前端加密流程

### 8.1 加密流程图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         前端加密工作流                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  1. 用户输入敏感数据                                                      │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  姓名: "张三"                                                    │ │
│     │  问题: "今年事业运势如何？"                                       │ │
│     │  出生时间: 1990-05-15 10:30                                      │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                  ↓                                       │
│  2. 生成随机主密钥（本地）                                                │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  const masterKey = crypto.getRandomValues(new Uint8Array(32));  │ │
│     │  // 保存到 LocalStorage 或安全存储                                │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                  ↓                                       │
│  3. 序列化敏感数据                                                        │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  const plaintext = encodeSensitiveData({                        │ │
│     │    name: "张三",                                                 │ │
│     │    question: "今年事业运势如何？",                                │ │
│     │    solarDate: [1990, 5, 15],                                     │ │
│     │    solarTime: [10, 30, 0],                                       │ │
│     │  });                                                             │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                  ↓                                       │
│  4. AES-256-GCM 加密                                                     │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  const nonce = crypto.getRandomValues(new Uint8Array(12));      │ │
│     │  const ciphertext = aesGcmEncrypt(plaintext, masterKey, nonce); │ │
│     │  const encryptedData = concat(nonce, ciphertext);               │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                  ↓                                       │
│  5. 计算数据哈希                                                          │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  const dataHash = blake2b256(plaintext);                        │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                  ↓                                       │
│  6. 用所有者公钥包装主密钥                                                │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  const ownerKeyBackup = x25519Wrap(masterKey, ownerPublicKey);  │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                  ↓                                       │
│  7. 提交到链上                                                            │
│     ┌─────────────────────────────────────────────────────────────────┐ │
│     │  api.tx.qimen.createChart(                                      │ │
│     │    ...publicMetadata,                                           │ │
│     │    encryptionLevel: 1, // PartialEncrypted                      │ │
│     │    encryptedData,                                               │ │
│     │    dataHash,                                                    │ │
│     │    ownerKeyBackup,                                              │ │
│     │    ...                                                          │ │
│     │  );                                                             │ │
│     └─────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.2 TypeScript 实现

```typescript
import { gcm } from '@noble/ciphers/aes';
import { x25519 } from '@noble/curves/ed25519';
import { blake2b } from '@noble/hashes/blake2b';
import { randomBytes } from '@noble/ciphers/webcrypto';

/**
 * 敏感数据接口
 */
interface SensitiveData {
  name?: string;
  question?: string;
  solarDate?: [number, number, number]; // [year, month, day]
  solarTime?: [number, number, number]; // [hour, minute, second]
  notes?: string;
}

/**
 * 加密结果接口
 */
interface EncryptionResult {
  encryptedData: Uint8Array;
  dataHash: Uint8Array;
  ownerKeyBackup: Uint8Array;
  masterKey: Uint8Array; // 需要安全存储
}

/**
 * 奇门加密服务类
 *
 * 提供前端加密/解密功能
 */
export class QimenEncryptionService {
  private static NONCE_LENGTH = 12;
  private static KEY_LENGTH = 32;

  /**
   * 加密敏感数据
   *
   * @param data - 敏感数据明文
   * @param ownerPublicKey - 所有者公钥（用于密钥备份）
   * @returns 加密结果
   */
  static async encrypt(
    data: SensitiveData,
    ownerPublicKey: Uint8Array
  ): Promise<EncryptionResult> {
    // 1. 生成随机主密钥
    const masterKey = randomBytes(this.KEY_LENGTH);

    // 2. 序列化敏感数据
    const plaintext = this.serializeSensitiveData(data);

    // 3. 计算数据哈希（用于完整性验证）
    const dataHash = blake2b(plaintext, { dkLen: 32 });

    // 4. 生成随机 nonce
    const nonce = randomBytes(this.NONCE_LENGTH);

    // 5. AES-256-GCM 加密
    const cipher = gcm(masterKey, nonce);
    const ciphertext = cipher.encrypt(plaintext);

    // 6. 组装加密数据（nonce + ciphertext）
    const encryptedData = new Uint8Array(nonce.length + ciphertext.length);
    encryptedData.set(nonce);
    encryptedData.set(ciphertext, nonce.length);

    // 7. 用所有者公钥包装主密钥
    const ownerKeyBackup = await this.wrapKey(masterKey, ownerPublicKey);

    return {
      encryptedData,
      dataHash,
      ownerKeyBackup,
      masterKey, // 调用者需要安全存储此密钥
    };
  }

  /**
   * 解密敏感数据
   *
   * @param encryptedData - 加密数据
   * @param masterKey - 主密钥
   * @param expectedHash - 期望的数据哈希（用于验证）
   * @returns 解密后的敏感数据
   */
  static decrypt(
    encryptedData: Uint8Array,
    masterKey: Uint8Array,
    expectedHash?: Uint8Array
  ): SensitiveData {
    // 1. 分离 nonce 和密文
    const nonce = encryptedData.slice(0, this.NONCE_LENGTH);
    const ciphertext = encryptedData.slice(this.NONCE_LENGTH);

    // 2. AES-256-GCM 解密
    const cipher = gcm(masterKey, nonce);
    const plaintext = cipher.decrypt(ciphertext);

    // 3. 验证数据哈希
    if (expectedHash) {
      const actualHash = blake2b(plaintext, { dkLen: 32 });
      if (!this.compareBytes(actualHash, expectedHash)) {
        throw new Error('Data hash mismatch - data may be corrupted');
      }
    }

    // 4. 反序列化
    return this.deserializeSensitiveData(plaintext);
  }

  /**
   * 包装密钥（用于授权）
   *
   * 使用 X25519 密钥交换生成共享密钥，然后加密主密钥
   *
   * @param masterKey - 主密钥
   * @param recipientPublicKey - 接收者公钥
   * @returns 包装后的密钥（80 bytes: 32 ephemeral + 32 encrypted + 16 tag）
   */
  static async wrapKey(
    masterKey: Uint8Array,
    recipientPublicKey: Uint8Array
  ): Promise<Uint8Array> {
    // 1. 生成临时密钥对
    const ephemeralPrivate = randomBytes(32);
    const ephemeralPublic = x25519.getPublicKey(ephemeralPrivate);

    // 2. X25519 密钥交换
    const sharedSecret = x25519.getSharedSecret(ephemeralPrivate, recipientPublicKey);

    // 3. 派生包装密钥
    const wrapKey = blake2b(sharedSecret, { dkLen: 32 });

    // 4. 加密主密钥
    const nonce = new Uint8Array(12); // 固定 nonce，因为 wrapKey 是一次性的
    const cipher = gcm(wrapKey, nonce);
    const encryptedKey = cipher.encrypt(masterKey);

    // 5. 组装输出（ephemeral_public + encrypted_key）
    const result = new Uint8Array(80);
    result.set(ephemeralPublic);
    result.set(encryptedKey, 32);

    return result;
  }

  /**
   * 解包密钥
   *
   * @param wrappedKey - 包装后的密钥
   * @param recipientPrivateKey - 接收者私钥
   * @returns 主密钥
   */
  static unwrapKey(
    wrappedKey: Uint8Array,
    recipientPrivateKey: Uint8Array
  ): Uint8Array {
    // 1. 分离临时公钥和加密的密钥
    const ephemeralPublic = wrappedKey.slice(0, 32);
    const encryptedKey = wrappedKey.slice(32);

    // 2. X25519 密钥交换
    const sharedSecret = x25519.getSharedSecret(recipientPrivateKey, ephemeralPublic);

    // 3. 派生包装密钥
    const wrapKey = blake2b(sharedSecret, { dkLen: 32 });

    // 4. 解密主密钥
    const nonce = new Uint8Array(12);
    const cipher = gcm(wrapKey, nonce);

    return cipher.decrypt(encryptedKey);
  }

  /**
   * 序列化敏感数据
   */
  private static serializeSensitiveData(data: SensitiveData): Uint8Array {
    const encoder = new TextEncoder();
    const parts: Uint8Array[] = [];

    // 简单的 TLV (Type-Length-Value) 编码
    if (data.name) {
      const nameBytes = encoder.encode(data.name);
      parts.push(new Uint8Array([0x01, nameBytes.length])); // Type: name
      parts.push(nameBytes);
    }

    if (data.question) {
      const questionBytes = encoder.encode(data.question);
      parts.push(new Uint8Array([0x02, questionBytes.length])); // Type: question
      parts.push(questionBytes);
    }

    if (data.solarDate) {
      parts.push(new Uint8Array([0x03, 4])); // Type: solarDate, Length: 4
      const dateBytes = new Uint8Array(4);
      new DataView(dateBytes.buffer).setUint16(0, data.solarDate[0], true);
      dateBytes[2] = data.solarDate[1];
      dateBytes[3] = data.solarDate[2];
      parts.push(dateBytes);
    }

    if (data.solarTime) {
      parts.push(new Uint8Array([0x04, 3])); // Type: solarTime, Length: 3
      parts.push(new Uint8Array(data.solarTime));
    }

    if (data.notes) {
      const notesBytes = encoder.encode(data.notes);
      parts.push(new Uint8Array([0x05, notesBytes.length])); // Type: notes
      parts.push(notesBytes);
    }

    // 合并所有部分
    const totalLength = parts.reduce((sum, part) => sum + part.length, 0);
    const result = new Uint8Array(totalLength);
    let offset = 0;
    for (const part of parts) {
      result.set(part, offset);
      offset += part.length;
    }

    return result;
  }

  /**
   * 反序列化敏感数据
   */
  private static deserializeSensitiveData(data: Uint8Array): SensitiveData {
    const decoder = new TextDecoder();
    const result: SensitiveData = {};

    let offset = 0;
    while (offset < data.length) {
      const type = data[offset];
      const length = data[offset + 1];
      const value = data.slice(offset + 2, offset + 2 + length);

      switch (type) {
        case 0x01:
          result.name = decoder.decode(value);
          break;
        case 0x02:
          result.question = decoder.decode(value);
          break;
        case 0x03:
          result.solarDate = [
            new DataView(value.buffer, value.byteOffset).getUint16(0, true),
            value[2],
            value[3],
          ];
          break;
        case 0x04:
          result.solarTime = [value[0], value[1], value[2]];
          break;
        case 0x05:
          result.notes = decoder.decode(value);
          break;
      }

      offset += 2 + length;
    }

    return result;
  }

  /**
   * 比较两个字节数组
   */
  private static compareBytes(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) return false;
    }
    return true;
  }
}
```

### 8.3 使用示例

#### 8.3.1 三种模式的调用对比

```typescript
import { ApiPromise } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { QimenEncryptionService } from './qimenEncryption';
import { blake2b } from '@noble/hashes/blake2b';

// ========== Public 模式 ==========
// 完全公开，无加密
async function createPublicChart(api: ApiPromise, account: KeyringPair) {
  const questionHash = blake2b(new TextEncoder().encode('今年运势如何？'), { dkLen: 32 });

  const tx = api.tx.qimen.createChart(
    [6, 5], [3, 4], [0, 2], [4, 5],  // 四柱干支
    9, 5,                             // 节气、天数
    0,                                // encryption_level: Public
    null,                             // encrypted_data: None
    null,                             // data_hash: None
    null,                             // owner_key_backup: None
    Array.from(questionHash),
    true,                             // is_public
    0, 1990, 1, 0
  );
  await tx.signAndSend(account);
}

// ========== Partial 模式（推荐）==========
// 排盘数据明文，敏感信息（姓名、问题）加密
async function createPartialChart(api: ApiPromise, account: KeyringPair) {
  // 1. 准备敏感数据（仅姓名和问题）
  const sensitiveData = {
    name: '张三',
    question: '今年事业运势如何？',
  };

  // 2. 加密敏感数据
  const encryptionResult = await QimenEncryptionService.encrypt(
    sensitiveData,
    account.publicKey
  );

  // 3. 保存主密钥
  localStorage.setItem(`qimen_key_${account.address}`,
    Buffer.from(encryptionResult.masterKey).toString('hex'));

  // 4. 提交（排盘数据明文传入）
  const questionHash = blake2b(new TextEncoder().encode(sensitiveData.question), { dkLen: 32 });
  const tx = api.tx.qimen.createChart(
    [6, 5], [3, 4], [0, 2], [4, 5],  // 四柱干支 - 明文
    9, 5,                             // 节气、天数 - 明文
    1,                                // encryption_level: Partial ⭐
    Array.from(encryptionResult.encryptedData),    // 加密的姓名+问题
    Array.from(encryptionResult.dataHash),
    Array.from(encryptionResult.ownerKeyBackup),
    Array.from(questionHash),
    false,                            // is_public
    0, 1990, 1, 0
  );
  await tx.signAndSend(account);
}

// ========== Full 模式 ==========
// 所有数据加密（不支持链上解盘API）
async function createFullChart(api: ApiPromise, account: KeyringPair) {
  // 1. 准备所有敏感数据
  const sensitiveData = {
    name: '张三',
    question: '今年事业运势如何？',
    solarDate: [1990, 5, 15] as [number, number, number],
    solarTime: [10, 30, 0] as [number, number, number],
  };

  // 2. 加密全部数据
  const encryptionResult = await QimenEncryptionService.encrypt(
    sensitiveData,
    account.publicKey
  );

  // 3. 提交（公开元数据可填占位符）
  const questionHash = blake2b(new TextEncoder().encode(sensitiveData.question || ''), { dkLen: 32 });
  const tx = api.tx.qimen.createChart(
    [0, 0], [0, 0], [0, 0], [0, 0],  // 四柱干支 - 占位（被加密数据覆盖）
    0, 1,                             // 节气、天数 - 占位
    2,                                // encryption_level: Full
    Array.from(encryptionResult.encryptedData),    // 加密的全部数据
    Array.from(encryptionResult.dataHash),
    Array.from(encryptionResult.ownerKeyBackup),
    Array.from(questionHash),
    false,
    null, null, null, 0
  );
  await tx.signAndSend(account);
}
```

#### 8.3.2 公历时间起局（推荐方式）

```typescript
/**
 * 使用公历时间起局（Partial 模式）
 *
 * 无需手动计算四柱干支，链上自动转换
 */
async function divineByDateTime(api: ApiPromise, account: KeyringPair) {
  // 1. 准备敏感数据
  const sensitiveData = {
    name: '张三',
    question: '今年事业运势如何？',
  };

  // 2. 加密
  const encryptionResult = await QimenEncryptionService.encrypt(
    sensitiveData,
    account.publicKey
  );

  // 3. 保存主密钥
  localStorage.setItem(`qimen_key_${account.address}`,
    Buffer.from(encryptionResult.masterKey).toString('hex'));

  // 4. 计算问题哈希
  const questionHash = blake2b(
    new TextEncoder().encode(sensitiveData.question || ''),
    { dkLen: 32 }
  );

  // 5. 提交（使用公历时间，链上自动计算四柱）
  const tx = api.tx.qimen.divineBySolarTime(
    2025, 12, 25, 10,                 // 公历：2025年12月25日 10时
    1,                                // encryption_level: Partial
    Array.from(encryptionResult.encryptedData),
    Array.from(encryptionResult.dataHash),
    Array.from(encryptionResult.ownerKeyBackup),
    Array.from(questionHash),
    false,                            // is_public
    0,                                // gender: Male
    1990,                             // birth_year
    1,                                // question_type: Career
    0                                 // pan_method: ZhuanPan
  );

  // 6. 签名并发送
  const unsub = await tx.signAndSend(account, ({ status, events }) => {
    if (status.isInBlock) {
      console.log('Transaction included in block');

      events.forEach(({ event }) => {
        if (api.events.qimen.ChartCreated.is(event)) {
          const chartId = event.data[0].toString();
          console.log('Created chart:', chartId);
        }
      });

      unsub();
    }
  });
}

/**
 * 授权查看者示例
 */
async function authorizeViewer(
  api: ApiPromise,
  ownerAccount: KeyringPair,
  chartId: number,
  viewerPublicKey: Uint8Array,
) {
  // 1. 从本地存储获取主密钥
  const masterKeyHex = localStorage.getItem(
    `qimen_master_key_${ownerAccount.address}`
  );
  if (!masterKeyHex) {
    throw new Error('Master key not found');
  }
  const masterKey = Buffer.from(masterKeyHex, 'hex');

  // 2. 为查看者包装密钥
  const wrappedKey = await QimenEncryptionService.wrapKey(
    masterKey,
    viewerPublicKey
  );

  // 3. 提交授权交易
  const tx = api.tx.qimen.authorizeViewer(
    chartId,
    viewerPublicKey,
    Array.from(wrappedKey),
    2, // permission: FullAccess
    0, // expires_at: never
  );

  await tx.signAndSend(ownerAccount);
}

/**
 * 查看者解密数据示例
 */
async function viewEncryptedChart(
  api: ApiPromise,
  viewerAccount: KeyringPair,
  chartId: number,
) {
  // 1. 获取包装的密钥
  const wrappedKeyInfo = await api.call.qimenEncryptedApi.getWrappedKey(
    chartId,
    viewerAccount.address
  );

  if (!wrappedKeyInfo || wrappedKeyInfo.isExpired) {
    throw new Error('Not authorized or authorization expired');
  }

  // 2. 解包密钥
  const masterKey = QimenEncryptionService.unwrapKey(
    wrappedKeyInfo.wrappedKey,
    viewerAccount.secretKey
  );

  // 3. 获取加密数据
  const encryptedData = await api.call.qimenEncryptedApi.getEncryptedData(chartId);
  if (!encryptedData) {
    throw new Error('Encrypted data not found');
  }

  // 4. 解密
  const sensitiveData = QimenEncryptionService.decrypt(
    encryptedData.ciphertext,
    masterKey,
    encryptedData.dataHash
  );

  console.log('Decrypted data:', sensitiveData);
  return sensitiveData;
}
```

---

## 九、悬赏问答授权流程

### 9.1 业务场景

**核心问题**：创建者发布加密命盘悬赏，大师接单后如何获取加密内容？

**采用方案**：手动授权（简单可靠）

### 9.2 授权流程设计

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      悬赏问答手动授权流程                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  创建者                              大师                                │
│    │                                  │                                  │
│    │ 1. 发布悬赏（加密命盘）            │                                  │
│    │    - 加密数据上链                 │                                  │
│    │    - owner_key_backup 上链       │                                  │
│    │                                  │                                  │
│    │                                  │ 2. 浏览悬赏列表                   │
│    │                                  │    （看到问题类型、悬赏金额）       │
│    │                                  │                                  │
│    │                                  │ 3. 接单（质押保证金）              │
│    │                                  │    → 链上记录待授权状态            │
│    │ ←────────────────────────────────│                                  │
│    │                                  │                                  │
│    │ 4. 收到通知 / 再次上线看到待授权    │                                  │
│    │                                  │                                  │
│    │ 5. 手动授权                       │                                  │
│    │    - 从 owner_key_backup 恢复密钥  │                                  │
│    │    - 为大师包装新密钥              │                                  │
│    │    - 提交授权交易                 │                                  │
│    │────────────────────────────────→ │                                  │
│    │                                  │                                  │
│    │                                  │ 6. 获得授权，解密内容              │
│    │                                  │                                  │
│    │                                  │ 7. 进行解读，提交结果              │
│    │ ←────────────────────────────────│                                  │
│    │                                  │                                  │
│    │ 8. 确认满意，释放悬赏              │                                  │
│    │────────────────────────────────→ │ 收到悬赏 ✅                       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 9.3 数据结构

```rust
/// 悬赏状态
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum BountyStatus {
    /// 等待接单
    Open,
    /// 已接单，待创建者授权
    PendingAuthorization,
    /// 已授权，大师解读中
    InProgress,
    /// 大师已提交解读，待确认
    PendingConfirmation,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 争议中
    Disputed,
}

/// 悬赏记录
#[derive(Clone, Debug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Bounty<AccountId, Balance, BlockNumber> {
    /// 悬赏 ID
    pub id: u64,
    /// 创建者
    pub creator: AccountId,
    /// 关联的加密命盘 ID
    pub chart_id: u64,
    /// 悬赏金额
    pub amount: Balance,
    /// 接单大师（接单后填充）
    pub master: Option<AccountId>,
    /// 大师质押金额
    pub master_stake: Balance,
    /// 状态
    pub status: BountyStatus,
    /// 创建时间
    pub created_at: BlockNumber,
    /// 截止时间
    pub deadline: BlockNumber,
    /// 授权时间（授权后填充）
    pub authorized_at: Option<BlockNumber>,
}

/// 待授权记录（用于创建者查询）
#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct PendingAuthorization<AccountId> {
    /// 悬赏 ID
    pub bounty_id: u64,
    /// 加密命盘 ID
    pub chart_id: u64,
    /// 待授权的大师
    pub master: AccountId,
    /// 接单时间
    pub accepted_at: u64,
}
```

### 9.4 存储设计

```rust
/// 悬赏存储
#[pallet::storage]
pub type Bounties<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // bounty_id
    Bounty<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
>;

/// 创建者的待授权列表
///
/// 当大师接单后，自动添加到此列表
/// 创建者授权后，从此列表移除
#[pallet::storage]
pub type PendingAuthorizations<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId, // creator
    BoundedVec<PendingAuthorization<T::AccountId>, T::MaxPendingAuthorizations>,
    ValueQuery,
>;

/// 大师接单的悬赏列表
#[pallet::storage]
pub type MasterBounties<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId, // master
    BoundedVec<u64, T::MaxMasterBounties>, // bounty_ids
    ValueQuery,
>;
```

### 9.5 Pallet 接口

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 创建悬赏（带加密命盘）
    ///
    /// 创建加密命盘并发布悬赏
    #[pallet::call_index(20)]
    #[pallet::weight(Weight::from_parts(120_000_000, 0))]
    pub fn create_bounty(
        origin: OriginFor<T>,
        // 加密命盘参数
        chart_params: EncryptedChartParams,
        // 悬赏金额
        amount: BalanceOf<T>,
        // 截止时间
        deadline: BlockNumberFor<T>,
    ) -> DispatchResult {
        let creator = ensure_signed(origin)?;

        // 1. 创建加密命盘
        let chart_id = Self::create_encrypted_chart_internal(
            creator.clone(),
            chart_params,
        )?;

        // 2. 锁定悬赏金额到托管账户
        T::Currency::transfer(
            &creator,
            &Self::escrow_account(),
            amount,
            ExistenceRequirement::KeepAlive,
        )?;

        // 3. 创建悬赏记录
        let bounty_id = Self::next_bounty_id();
        let bounty = Bounty {
            id: bounty_id,
            creator: creator.clone(),
            chart_id,
            amount,
            master: None,
            master_stake: Zero::zero(),
            status: BountyStatus::Open,
            created_at: <frame_system::Pallet<T>>::block_number(),
            deadline,
            authorized_at: None,
        };

        Bounties::<T>::insert(bounty_id, bounty);

        Self::deposit_event(Event::BountyCreated {
            bounty_id,
            creator,
            chart_id,
            amount,
        });

        Ok(())
    }

    /// 大师接单
    ///
    /// 大师接受悬赏，需要质押保证金
    #[pallet::call_index(21)]
    #[pallet::weight(Weight::from_parts(80_000_000, 0))]
    pub fn accept_bounty(
        origin: OriginFor<T>,
        bounty_id: u64,
        stake_amount: BalanceOf<T>,
    ) -> DispatchResult {
        let master = ensure_signed(origin)?;

        Bounties::<T>::try_mutate(bounty_id, |maybe_bounty| {
            let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.status == BountyStatus::Open, Error::<T>::InvalidBountyStatus);
            ensure!(bounty.creator != master, Error::<T>::CannotAcceptOwnBounty);

            // 锁定大师质押金
            T::Currency::transfer(
                &master,
                &Self::escrow_account(),
                stake_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 更新悬赏状态
            bounty.master = Some(master.clone());
            bounty.master_stake = stake_amount;
            bounty.status = BountyStatus::PendingAuthorization;

            // 添加到创建者的待授权列表
            PendingAuthorizations::<T>::try_mutate(&bounty.creator, |list| {
                list.try_push(PendingAuthorization {
                    bounty_id,
                    chart_id: bounty.chart_id,
                    master: master.clone(),
                    accepted_at: Self::current_timestamp(),
                }).map_err(|_| Error::<T>::TooManyPendingAuthorizations)
            })?;

            Self::deposit_event(Event::BountyAccepted {
                bounty_id,
                master: master.clone(),
                creator: bounty.creator.clone(),
            });

            Ok(())
        })
    }

    /// 创建者授权大师查看
    ///
    /// 创建者为接单大师授权查看加密内容
    #[pallet::call_index(22)]
    #[pallet::weight(Weight::from_parts(60_000_000, 0))]
    pub fn authorize_bounty_master(
        origin: OriginFor<T>,
        bounty_id: u64,
        wrapped_key: [u8; 80],
    ) -> DispatchResult {
        let creator = ensure_signed(origin)?;

        Bounties::<T>::try_mutate(bounty_id, |maybe_bounty| {
            let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.creator == creator, Error::<T>::NotBountyCreator);
            ensure!(bounty.status == BountyStatus::PendingAuthorization,
                    Error::<T>::InvalidBountyStatus);

            let master = bounty.master.clone().ok_or(Error::<T>::NoMasterAssigned)?;

            // 添加授权到加密命盘
            Self::add_authorization(
                bounty.chart_id,
                master.clone(),
                wrapped_key,
                ViewPermission::FullAccess,
                0, // 永不过期
            )?;

            // 更新状态
            bounty.status = BountyStatus::InProgress;
            bounty.authorized_at = Some(<frame_system::Pallet<T>>::block_number());

            // 从待授权列表移除
            PendingAuthorizations::<T>::mutate(&creator, |list| {
                list.retain(|p| p.bounty_id != bounty_id);
            });

            Self::deposit_event(Event::BountyAuthorized {
                bounty_id,
                creator,
                master,
            });

            Ok(())
        })
    }

    /// 大师提交解读结果
    #[pallet::call_index(23)]
    #[pallet::weight(Weight::from_parts(50_000_000, 0))]
    pub fn submit_interpretation(
        origin: OriginFor<T>,
        bounty_id: u64,
        interpretation_cid: BoundedVec<u8, T::MaxCidLen>,
    ) -> DispatchResult {
        let master = ensure_signed(origin)?;

        Bounties::<T>::try_mutate(bounty_id, |maybe_bounty| {
            let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.master == Some(master.clone()), Error::<T>::NotBountyMaster);
            ensure!(bounty.status == BountyStatus::InProgress,
                    Error::<T>::InvalidBountyStatus);

            // 更新命盘的解读 CID
            EncryptedCharts::<T>::try_mutate(bounty.chart_id, |maybe_chart| {
                let chart = maybe_chart.as_mut().ok_or(Error::<T>::ChartNotFound)?;
                chart.interpretation_cid = Some(interpretation_cid.clone());
                Ok::<_, DispatchError>(())
            })?;

            bounty.status = BountyStatus::PendingConfirmation;

            Self::deposit_event(Event::InterpretationSubmitted {
                bounty_id,
                master,
            });

            Ok(())
        })
    }

    /// 创建者确认完成
    #[pallet::call_index(24)]
    #[pallet::weight(Weight::from_parts(60_000_000, 0))]
    pub fn confirm_bounty(
        origin: OriginFor<T>,
        bounty_id: u64,
    ) -> DispatchResult {
        let creator = ensure_signed(origin)?;

        Bounties::<T>::try_mutate(bounty_id, |maybe_bounty| {
            let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.creator == creator, Error::<T>::NotBountyCreator);
            ensure!(bounty.status == BountyStatus::PendingConfirmation,
                    Error::<T>::InvalidBountyStatus);

            let master = bounty.master.clone().ok_or(Error::<T>::NoMasterAssigned)?;

            // 释放悬赏给大师
            T::Currency::transfer(
                &Self::escrow_account(),
                &master,
                bounty.amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // 退还大师质押金
            T::Currency::transfer(
                &Self::escrow_account(),
                &master,
                bounty.master_stake,
                ExistenceRequirement::AllowDeath,
            )?;

            bounty.status = BountyStatus::Completed;

            Self::deposit_event(Event::BountyCompleted {
                bounty_id,
                creator,
                master,
                amount: bounty.amount,
            });

            Ok(())
        })
    }
}
```

### 9.6 事件定义

```rust
#[pallet::event]
pub enum Event<T: Config> {
    /// 悬赏创建
    BountyCreated {
        bounty_id: u64,
        creator: T::AccountId,
        chart_id: u64,
        amount: BalanceOf<T>,
    },

    /// 大师接单（触发待授权通知）
    BountyAccepted {
        bounty_id: u64,
        master: T::AccountId,
        creator: T::AccountId,  // 前端可监听此事件通知创建者
    },

    /// 创建者已授权
    BountyAuthorized {
        bounty_id: u64,
        creator: T::AccountId,
        master: T::AccountId,
    },

    /// 大师提交解读
    InterpretationSubmitted {
        bounty_id: u64,
        master: T::AccountId,
    },

    /// 悬赏完成
    BountyCompleted {
        bounty_id: u64,
        creator: T::AccountId,
        master: T::AccountId,
        amount: BalanceOf<T>,
    },
}
```

### 9.7 前端实现

```typescript
/**
 * 创建者：发布悬赏
 */
async function createBounty(
  api: ApiPromise,
  creator: KeyringPair,
  sensitiveData: SensitiveData,
  chartParams: PublicChartParams,
  bountyAmount: bigint,
  deadlineBlocks: number,
) {
  // 1. 加密敏感数据
  const encryptionResult = await QimenEncryptionService.encrypt(
    sensitiveData,
    creator.publicKey
  );

  // 2. 保存主密钥到本地
  saveLocalMasterKey(creator.address, 'pending', encryptionResult.masterKey);

  // 3. 提交悬赏
  const tx = api.tx.qimenBounty.createBounty(
    {
      ...chartParams,
      encryptedData: Array.from(encryptionResult.encryptedData),
      dataHash: Array.from(encryptionResult.dataHash),
      ownerKeyBackup: Array.from(encryptionResult.ownerKeyBackup),
    },
    bountyAmount,
    deadlineBlocks,
  );

  const result = await tx.signAndSend(creator);

  // 4. 获取 chart_id 并更新本地存储
  // ... 从事件中获取 chart_id，更新 masterKey 的存储键
}

/**
 * 创建者：查看待授权列表并授权
 */
async function handlePendingAuthorizations(
  api: ApiPromise,
  creator: KeyringPair,
) {
  // 1. 查询待授权列表
  const pendingList = await api.query.qimenBounty.pendingAuthorizations(
    creator.address
  );

  if (pendingList.length === 0) {
    console.log('没有待授权的请求');
    return;
  }

  console.log(`有 ${pendingList.length} 个待授权请求`);

  for (const pending of pendingList) {
    const { bountyId, chartId, master } = pending;

    // 2. 获取主密钥（先尝试本地，再从链上恢复）
    let masterKey = getLocalMasterKey(creator.address, chartId);

    if (!masterKey) {
      console.log(`从链上恢复密钥: chartId=${chartId}`);
      masterKey = await recoverMasterKeyFromChain(api, creator, chartId);
    }

    // 3. 获取大师公钥
    const masterPublicKey = await getAccountPublicKey(api, master);

    // 4. 为大师包装密钥
    const wrappedKey = await QimenEncryptionService.wrapKey(
      masterKey,
      masterPublicKey
    );

    // 5. 提交授权
    const tx = api.tx.qimenBounty.authorizeBountyMaster(
      bountyId,
      Array.from(wrappedKey)
    );

    await tx.signAndSend(creator);
    console.log(`已授权大师 ${master} 查看悬赏 ${bountyId}`);
  }
}

/**
 * 从链上恢复主密钥
 */
async function recoverMasterKeyFromChain(
  api: ApiPromise,
  owner: KeyringPair,
  chartId: number,
): Promise<Uint8Array> {
  // 1. 获取链上存储的 owner_key_backup
  const chart = await api.query.qimen.encryptedCharts(chartId);

  if (!chart.ownerKeyBackup) {
    throw new Error('No key backup found on chain');
  }

  // 2. 用所有者私钥解密
  const masterKey = QimenEncryptionService.unwrapKey(
    new Uint8Array(chart.ownerKeyBackup),
    owner.secretKey
  );

  // 3. 保存到本地
  saveLocalMasterKey(owner.address, chartId, masterKey);

  return masterKey;
}

/**
 * 大师：接单后等待授权并解密
 */
async function masterWorkflow(
  api: ApiPromise,
  master: KeyringPair,
  bountyId: number,
) {
  // 1. 接单
  await api.tx.qimenBounty.acceptBounty(bountyId, stakeAmount)
    .signAndSend(master);

  console.log('已接单，等待创建者授权...');

  // 2. 轮询或监听授权事件
  const authorized = await waitForAuthorization(api, bountyId, master.address);

  if (!authorized) {
    console.log('授权超时');
    return;
  }

  // 3. 获取加密内容并解密
  const bounty = await api.query.qimenBounty.bounties(bountyId);
  const chartId = bounty.chartId;

  // 获取为我包装的密钥
  const wrappedKeyInfo = await api.call.qimenEncryptedApi.getWrappedKey(
    chartId,
    master.address
  );

  // 解包密钥
  const dataKey = QimenEncryptionService.unwrapKey(
    wrappedKeyInfo.wrappedKey,
    master.secretKey
  );

  // 获取加密数据并解密
  const encryptedData = await api.call.qimenEncryptedApi.getEncryptedData(chartId);
  const decryptedData = QimenEncryptionService.decrypt(
    encryptedData.ciphertext,
    dataKey,
    encryptedData.dataHash
  );

  console.log('解密成功:', decryptedData);

  // 4. 进行解读...
  // 5. 提交解读结果
}

/**
 * 等待授权（轮询方式）
 */
async function waitForAuthorization(
  api: ApiPromise,
  bountyId: number,
  masterAddress: string,
  timeoutMs: number = 24 * 60 * 60 * 1000, // 默认24小时
): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const bounty = await api.query.qimenBounty.bounties(bountyId);

    if (bounty.status.toString() === 'InProgress') {
      return true; // 已授权
    }

    if (bounty.status.toString() === 'Cancelled') {
      return false; // 已取消
    }

    // 等待一段时间后重试
    await sleep(10000); // 10秒
  }

  return false; // 超时
}
```

### 9.8 用户体验优化

#### 创建者通知机制

```typescript
// 方式1：监听链上事件（实时）
function subscribeToAcceptEvents(api: ApiPromise, creator: string) {
  api.query.system.events((events) => {
    events.forEach(({ event }) => {
      if (api.events.qimenBounty.BountyAccepted.is(event)) {
        const [bountyId, master, eventCreator] = event.data;
        if (eventCreator.toString() === creator) {
          showNotification({
            title: '有大师接单了！',
            body: `大师 ${master} 接受了您的悬赏，请尽快授权`,
            action: () => navigateTo(`/bounty/${bountyId}/authorize`)
          });
        }
      }
    });
  });
}

// 方式2：登录时检查（离线后）
async function checkPendingOnLogin(api: ApiPromise, creator: string) {
  const pending = await api.query.qimenBounty.pendingAuthorizations(creator);

  if (pending.length > 0) {
    showBanner({
      type: 'warning',
      message: `您有 ${pending.length} 个悬赏等待授权`,
      action: { text: '去授权', link: '/pending-authorizations' }
    });
  }
}
```

#### 超时保护

```rust
/// 授权超时检查（可通过链下工作机或定时任务触发）
#[pallet::call_index(25)]
pub fn check_authorization_timeout(
    origin: OriginFor<T>,
    bounty_id: u64,
) -> DispatchResult {
    let bounty = Bounties::<T>::get(bounty_id)
        .ok_or(Error::<T>::BountyNotFound)?;

    // 如果超过48小时未授权，大师可以取消接单并取回质押
    if bounty.status == BountyStatus::PendingAuthorization {
        let accepted_block = /* 从 pending 记录获取 */;
        let current_block = <frame_system::Pallet<T>>::block_number();
        let timeout_blocks = T::AuthorizationTimeout::get(); // 如 28800 blocks ≈ 48小时

        if current_block > accepted_block + timeout_blocks {
            // 退还大师质押，恢复悬赏状态为 Open
            // ...
        }
    }

    Ok(())
}
```

### 9.9 流程状态图

```
                                    ┌──────────────┐
                                    │              │
            创建悬赏                 │     Open     │
           ─────────────────────→   │   (等待接单)  │
                                    │              │
                                    └──────┬───────┘
                                           │
                                           │ 大师接单
                                           ▼
                                    ┌──────────────┐
                                    │   Pending    │
            ┌───────────────────────│Authorization │───────────────┐
            │                       │  (待授权)     │               │
            │                       └──────┬───────┘               │
            │                              │                       │
            │ 超时(48h)                    │ 创建者授权              │ 大师取消
            │                              ▼                       │
            │                       ┌──────────────┐               │
            │                       │              │               │
            │                       │  InProgress  │               │
            │                       │  (解读中)     │               │
            │                       │              │               │
            │                       └──────┬───────┘               │
            │                              │                       │
            │                              │ 大师提交解读            │
            │                              ▼                       │
            │                       ┌──────────────┐               │
            │                       │   Pending    │               │
            │                       │ Confirmation │               │
            │                       │  (待确认)     │               │
            │                       └──────┬───────┘               │
            │                              │                       │
            │           ┌──────────────────┼──────────────────┐    │
            │           │                  │                  │    │
            │           ▼                  ▼                  ▼    │
            │    ┌────────────┐     ┌────────────┐     ┌──────────┐│
            │    │ Completed  │     │  Disputed  │     │Cancelled ││
            └───→│  (已完成)   │     │  (争议中)   │     │ (已取消) │←┘
                 └────────────┘     └────────────┘     └──────────┘
```

---

## 十、安全考虑

### 10.1 密钥管理

| 风险 | 缓解措施 |
|------|----------|
| 主密钥泄露 | 本地安全存储（如 IndexedDB + 加密） |
| 主密钥丢失 | 链上存储所有者密钥备份 |
| 私钥泄露 | 使用硬件钱包签名 |

### 10.2 数据安全

| 风险 | 缓解措施 |
|------|----------|
| 密文被篡改 | AES-GCM 认证标签验证 |
| 重放攻击 | 随机 nonce + 时间戳 |
| 中间人攻击 | X25519 密钥交换 + 链上验证 |

### 10.3 权限控制

| 风险 | 缓解措施 |
|------|----------|
| 越权访问 | 链上授权列表 + 时间戳验证 |
| 授权滥用 | 权限分级 + 过期机制 |
| 所有权争议 | 转移需双方签名 |

---

## 十一、实现路线图

### Phase 1: 基础加密功能（2-3 周）

- [ ] 实现 `EncryptedQimenChart` 数据结构
- [ ] 实现 `create_encrypted_chart` extrinsic
- [ ] 实现前端 `QimenEncryptionService` 类
- [ ] 单元测试覆盖

### Phase 2: 授权管理（1-2 周）

- [ ] 实现授权/撤销 extrinsics
- [ ] 实现 Runtime API
- [ ] 前端授权 UI

### Phase 3: 高级功能（1-2 周）

- [ ] 批量授权
- [ ] 所有权转移
- [ ] 密钥轮换支持

### Phase 4: 优化与安全审计（1 周）

- [ ] 性能优化
- [ ] 安全审计
- [ ] 文档完善

---

## 十二、兼容性说明

### 12.1 与现有 QimenChart 的兼容性

- `EncryptedQimenChart` 是独立的存储结构
- 现有 `QimenChart` 保持不变
- 可通过迁移脚本将旧数据转换为加密格式

### 12.2 与八字加密方案的复用

- 共享 `EncryptionLevel`、`ViewPermission` 等枚举
- 共享前端加密工具类
- 共享授权管理逻辑

---

## 附录 A: 存储大小估算

| 组件 | 大小 | 说明 |
|------|------|------|
| PublicMetadata | ~350 bytes | 公开元数据 |
| EncryptedData | ~300 bytes | 加密敏感数据 |
| EncryptedKey | ~100 bytes | 单个授权 |
| 总计（单盘） | ~650 bytes + 100 × 授权数 | 完整加密命盘 |

## 附录 B: 常量配置建议

```rust
// 加密数据最大长度
pub const MAX_ENCRYPTED_LEN: u32 = 512;

// 每个命盘最大授权人数
pub const MAX_VIEWERS: u32 = 20;

// 用户被授权查看的最大命盘数
pub const MAX_AUTHORIZED_CHARTS: u32 = 100;

// 算法版本
pub const ALGORITHM_VERSION: u8 = 1;
```

---

*文档版本: 1.0.0*
*最后更新: 2025-12-25*
