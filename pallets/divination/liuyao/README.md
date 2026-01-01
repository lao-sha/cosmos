# pallet-liuyao

## 六爻排盘系统 - 区块链纳甲六爻占卜模块

本模块实现完整的六爻排盘算法，支持链上卦象生成与存储。六爻是中国传统周易占卜术的核心技法之一，通过摇卦得到六个爻，组成一个重卦（六十四卦之一），再通过纳甲装卦、安世应、配六亲六神等步骤，形成完整的卦象信息用于预测吉凶。

## 核心功能

### 🎲 多种起卦方式

| 方式 | 说明 | Extrinsic |
|------|------|-----------|
| 铜钱起卦 | 模拟三枚铜钱法，六次摇卦 | `divine_by_coins` |
| 数字起卦 | 报数法，上卦数+下卦数+动爻 | `divine_by_numbers` |
| 随机起卦 | 使用链上随机数生成 | `divine_random` |
| 手动指定 | 直接输入六爻类型 | `divine_manual` |

### 🔮 完整纳甲算法

纳甲是将天干地支配属于八卦各爻的方法：

```
纳甲口诀：
乾纳甲壬，坤纳乙癸，震纳庚，巽纳辛，
坎纳戊，离纳己，艮纳丙，兑纳丁。

纳支口诀：
乾金甲子外壬午，坎水戊寅外戊申，
艮土丙辰外丙戌，震木庚子外庚午，
巽木辛丑外辛未，离火己卯外己酉，
坤土乙未外癸丑，兑金丁巳外丁亥。
```

### 📈 世应计算

根据内外卦关系确定世爻和应爻位置：

```
寻世诀：
天同二世天变五，地同四世地变初。
本宫六世三世异，人同游魂人变归。
```

### 🏛️ 卦宫归属

确定卦象所属的八宫：

```
认宫诀：
一二三六外卦宫，四五游魂内变更。
若问归魂何所取，归魂内卦是本宫。
```

### 👨‍👩‍👧‍👦 六亲配置

根据卦宫五行与爻五行的生克关系配置六亲：

| 六亲 | 关系 | 含义 |
|------|------|------|
| 兄弟 | 同我者 | 竞争、破财 |
| 父母 | 生我者 | 文书、长辈 |
| 官鬼 | 克我者 | 功名、疾病 |
| 妻财 | 我克者 | 财富、妻子 |
| 子孙 | 我生者 | 后代、福德 |

### 🐉 六神排布

根据日干配置六神：

```
六神配日干口诀：
甲乙日起青龙，丙丁日起朱雀，
戊日起勾陈，己日起螣蛇，
庚辛日起白虎，壬癸日起玄武。
```

| 六神 | 五行 | 主事 |
|------|------|------|
| 青龙 | 木 | 吉庆、贵人 |
| 朱雀 | 火 | 口舌、文书 |
| 勾陈 | 土 | 田土、牵连 |
| 螣蛇 | 土 | 虚惊、怪异 |
| 白虎 | 金 | 凶事、血光 |
| 玄武 | 水 | 暗昧、盗贼 |

### ⭕ 旬空计算

六十甲子分六旬，每旬十天，缺两个地支为空亡：

| 旬首 | 空亡 |
|------|------|
| 甲子 | 戌亥 |
| 甲戌 | 申酉 |
| 甲申 | 午未 |
| 甲午 | 辰巳 |
| 甲辰 | 寅卯 |
| 甲寅 | 子丑 |

### 👤 伏神查找

当本卦六亲不全时，缺失的六亲从本宫纯卦中寻找伏神。

### 🔄 变卦生成

动爻（老阴、老阳）变化后形成变卦：
- 老阴（⚏）→ 变阳
- 老阳（⚌）→ 变阴

## 技术架构

```
┌─────────────────────────────────────────────────────────────┐
│                      pallet-liuyao                          │
├─────────────────────────────────────────────────────────────┤
│  Extrinsics:                                                │
│  - divine_by_coins: 铜钱起卦                                 │
│  - divine_by_numbers: 数字起卦                               │
│  - divine_random: 随机起卦                                   │
│  - divine_manual: 手动指定                                   │
│  - request_ai_interpretation: 请求AI解读                     │
│  - submit_ai_interpretation: 提交AI解读（预言机）             │
│  - set_gua_visibility: 设置卦象可见性                        │
├─────────────────────────────────────────────────────────────┤
│  Algorithm:                                                 │
│  - 纳甲算法（八卦配天干地支）                                  │
│  - 世应计算（寻世诀）                                         │
│  - 卦宫归属（认宫诀）                                         │
│  - 六亲配置                                                  │
│  - 六神排布                                                  │
│  - 旬空计算                                                  │
│  - 伏神查找                                                  │
│  - 变卦生成                                                  │
└─────────────────────────────────────────────────────────────┘
```

## 类型定义

### 基础类型

```rust
// 十天干
pub enum TianGan {
    Jia, Yi, Bing, Ding, Wu, Ji, Geng, Xin, Ren, Gui
}

// 十二地支
pub enum DiZhi {
    Zi, Chou, Yin, Mao, Chen, Si, Wu, Wei, Shen, You, Xu, Hai
}

// 五行
pub enum WuXing {
    Wood, Fire, Earth, Metal, Water
}

// 八卦（经卦）
pub enum Trigram {
    Qian, Dui, Li, Zhen, Xun, Kan, Gen, Kun
}

// 爻类型
pub enum Yao {
    ShaoYin,   // 少阴（静爻，阴）
    ShaoYang,  // 少阳（静爻，阳）
    LaoYin,    // 老阴（动爻，阴变阳）
    LaoYang,   // 老阳（动爻，阳变阴）
}
```

### 卦象结构

```rust
pub struct LiuYaoGua<AccountId, BlockNumber, MaxCidLen> {
    pub id: u64,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub method: DivinationMethod,

    // 时间信息
    pub year_gz: (TianGan, DiZhi),
    pub month_gz: (TianGan, DiZhi),
    pub day_gz: (TianGan, DiZhi),
    pub hour_gz: (TianGan, DiZhi),

    // 本卦信息
    pub original_yaos: [YaoInfo; 6],
    pub original_inner: Trigram,
    pub original_outer: Trigram,
    pub original_name_idx: u8,
    pub gong: Trigram,
    pub gua_xu: GuaXu,

    // 变卦信息
    pub has_bian_gua: bool,
    pub changed_yaos: [YaoInfo; 6],
    pub changed_inner: Trigram,
    pub changed_outer: Trigram,

    // 其他
    pub moving_yaos: u8,           // 动爻位图
    pub xun_kong: (DiZhi, DiZhi),  // 旬空
    pub fu_shen: [Option<FuShenInfo>; 6],  // 伏神
    pub is_public: bool,
    pub ai_interpretation_cid: Option<BoundedVec<u8, MaxCidLen>>,
}
```

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config + pallet_timestamp::Config {
    /// 货币类型
    type Currency: Currency<Self::AccountId>;

    /// 随机数生成器
    type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

    /// 每用户最大卦象数量
    type MaxUserGuas: Get<u32>;

    /// 公开列表最大长度
    type MaxPublicGuas: Get<u32>;

    /// 每日免费起卦次数
    type DailyFreeGuas: Get<u32>;

    /// 每日最大起卦次数
    type MaxDailyGuas: Get<u32>;

    /// AI 解读费用
    type AiInterpretationFee: Get<BalanceOf<Self>>;

    /// 国库账户
    type TreasuryAccount: Get<Self::AccountId>;

    /// AI 预言机权限
    type AiOracleOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// IPFS CID 最大长度
    type MaxCidLen: Get<u32>;
}
```

## 存储项

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `NextGuaId` | `u64` | 下一个卦象 ID |
| `Guas` | `Map<u64, LiuYaoGua>` | 所有卦象数据 |
| `UserGuas` | `Map<AccountId, Vec<u64>>` | 用户的卦象列表 |
| `PublicGuas` | `Vec<u64>` | 公开的卦象列表 |
| `DailyGuaCount` | `Map<(AccountId, u32), u32>` | 用户每日起卦次数 |
| `AiInterpretationRequests` | `Map<u64, bool>` | AI 解读请求状态 |
| `UserStatsStorage` | `Map<AccountId, UserStats>` | 用户统计数据 |

## 事件

```rust
pub enum Event<T: Config> {
    /// 卦象创建成功
    GuaCreated {
        gua_id: u64,
        creator: T::AccountId,
        method: DivinationMethod,
        original_name_idx: u8,
    },
    /// 请求 AI 解读
    AiInterpretationRequested {
        gua_id: u64,
        requester: T::AccountId,
    },
    /// AI 解读完成
    AiInterpretationSubmitted {
        gua_id: u64,
        cid: BoundedVec<u8, T::MaxCidLen>,
    },
    /// 可见性变更
    VisibilityChanged {
        gua_id: u64,
        is_public: bool,
    },
}
```

## 错误类型

```rust
pub enum Error<T> {
    GuaNotFound,                    // 卦象不存在
    NotGuaOwner,                    // 无权操作
    InvalidCoinCount,               // 无效的铜钱数（应为0-3）
    InvalidNumber,                  // 无效的数字（应大于0）
    InvalidDongYao,                 // 无效的动爻位置（应为1-6）
    DailyLimitExceeded,            // 超过每日起卦上限
    UserGuaLimitExceeded,          // 超过用户存储上限
    PublicGuaLimitExceeded,        // 超过公开列表上限
    AiInterpretationAlreadyRequested, // AI 解读已请求
    AiInterpretationNotRequested,  // AI 解读未请求
    InsufficientBalance,           // 余额不足
}
```

## 使用示例

### 铜钱起卦

```rust
// coins: 六次摇卦结果，每个值为阳面个数(0-3)
// 0=老阴, 1=少阳, 2=少阴, 3=老阳
let coins = [2, 1, 2, 1, 2, 1]; // 少阴少阳交替

Liuyao::divine_by_coins(
    origin,
    coins,
    (0, 0),  // 年干支：甲子
    (2, 2),  // 月干支：丙寅
    (4, 4),  // 日干支：戊辰
    (6, 6),  // 时干支：庚午
)?;
```

### 数字起卦

```rust
// num1: 上卦数, num2: 下卦数, dong: 动爻位置(1-6)
Liuyao::divine_by_numbers(
    origin,
    5,  // 上卦数
    3,  // 下卦数
    2,  // 动爻位置
    (0, 0), (2, 2), (4, 4), (6, 6),  // 年月日时干支
)?;
```

### 随机起卦

```rust
Liuyao::divine_random(origin)?;
```

### 请求 AI 解读

```rust
Liuyao::request_ai_interpretation(origin, gua_id)?;
```

## 六十四卦名表

```
坤为地、地雷复、地泽临、地天泰、雷天大壮、泽天夬、水天需、水地比、
艮为山、山火贲、山天大畜、山泽损、火泽睽、天泽履、风泽中孚、风山渐、
坎为水、水泽节、水雷屯、水火既济、泽火革、雷火丰、地火明夷、地水师、
巽为风、风天小畜、风火家人、风雷益、天雷无妄、火雷噬嗑、山雷颐、山风蛊、
震为雷、雷地豫、雷水解、雷风恒、地风升、水风井、泽风大过、泽雷随、
离为火、火山旅、火风鼎、火水未济、山水蒙、风水涣、天水讼、天火同人、
兑为泽、泽水困、泽地萃、泽山咸、水山蹇、地山谦、雷山小过、雷泽归妹、
乾为天、天风姤、天山遁、天地否、风地观、山地剥、火地晋、火天大有
```

## 测试

运行测试：

```bash
SKIP_WASM_BUILD=1 cargo test -p pallet-liuyao
```

测试覆盖：
- ✅ 基础功能测试（4种起卦方式）
- ✅ 参数校验测试
- ✅ 每日限制测试
- ✅ AI 解读测试
- ✅ 可见性测试
- ✅ 算法测试（纳甲、世应、六亲、六神、旬空、变卦等）
- ✅ 类型测试

## 依赖

- `frame-support`
- `frame-system`
- `pallet-balances`
- `pallet-timestamp`
- `sp-runtime`
- `sp-core`
- `sp-io`

## 许可证

MIT-0

## 参考资料

- 《增删卜易》- 野鹤老人
- 《卜筮正宗》- 王洪绪
- 《周易》
- najia - Python 六爻排盘库
- divicast - Python 占卜库
