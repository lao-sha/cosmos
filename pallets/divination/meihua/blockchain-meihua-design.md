# 区块链梅花易数排盘系统设计文档

## 项目概述

基于 Stardust 区块链（Substrate/Polkadot SDK）构建一个去中心化的梅花易数排盘系统，结合 AI 解卦能力，提供不可篡改的占卜记录和智能化的卦象解读服务。

### 核心价值

1. **不可篡改性**：所有占卜记录上链，确保数据真实可信
2. **时间戳证明**：利用区块链时间戳作为起卦时间的权威来源
3. **隐私保护**：支持加密存储敏感占卜内容
4. **AI 增强**：集成 AI 模型提供智能化卦象解读
5. **代币激励**：通过 DUST 代币实现占卜师和用户的价值交换

---

## 技术架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        前端 DApp (React)                         │
│   ├── 起卦界面 (数字/时间/随机)                                   │
│   ├── 卦象展示 (本卦/变卦/互卦)                                   │
│   ├── AI 解卦对话                                                │
│   └── 历史记录查询                                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ JSON-RPC / WebSocket
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Substrate Node (RPC :9944)                    │
│   ├── pallet-meihua          (梅花易数核心)                       │
│   ├── pallet-meihua-ai       (AI 解卦集成)                        │
│   ├── pallet-meihua-market   (占卜服务市场)                       │
│   └── pallet-meihua-nft      (卦象 NFT)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP API
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    AI 推理服务 (FastAPI :8000)                    │
│   ├── /api/v1/interpret      (卦象解读)                          │
│   ├── /api/v1/advice         (行动建议)                          │
│   └── /api/v1/dialogue       (对话式解卦)                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    数据存储层                                     │
│   ├── 链上存储 (Substrate Trie)  - 卦象结构、哈希                 │
│   ├── IPFS                       - 详细解读文本、图片              │
│   └── Subsquid                   - 历史数据索引                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 模块设计

### 1. pallet-meihua（核心排盘模块）

#### 1.1 数据结构

```rust
// pallets/meihua/src/types.rs

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::BoundedVec;

/// 八卦枚举
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum Bagua {
    Qian = 1,  // 乾 ☰ 111
    Dui = 2,   // 兑 ☱ 011
    Li = 3,    // 离 ☲ 101
    Zhen = 4,  // 震 ☳ 001
    Xun = 5,   // 巽 ☴ 110
    Kan = 6,   // 坎 ☵ 010
    Gen = 7,   // 艮 ☶ 100
    Kun = 0,   // 坤 ☷ 000
}

/// 五行枚举
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum WuXing {
    Jin,   // 金
    Mu,    // 木
    Shui,  // 水
    Huo,   // 火
    Tu,    // 土
}

/// 体用关系
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum TiYongRelation {
    BiHe,      // 比和 - 同五行
    Sheng,     // 生 - 用生体
    BeSheng,   // 被生 - 体生用
    Ke,        // 克 - 用克体
    BeKe,      // 被克 - 体克用
}

/// 起卦方式
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum DivinationMethod {
    /// 时间起卦（年月日时）
    DateTime,
    /// 双数起卦
    TwoNumbers { num1: u8, num2: u8 },
    /// 三位数起卦
    ThreeDigits { digits: u16 },
    /// 随机起卦（链上随机数）
    Random,
    /// 手动指定
    Manual { shang_gua: u8, xia_gua: u8, dong_yao: u8 },
}

/// 单卦结构（优化版 - 仅存储必要数据）
///
/// 存储优化说明：
/// - 原结构：约 50-80 bytes（含 BoundedVec）
/// - 优化后：仅 1 byte（Bagua 枚举）
/// - 一次完整排盘可节省约 200+ bytes
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct SingleGua {
    /// 卦名（所有其他属性均可由此推导）
    pub bagua: Bagua,
}

impl SingleGua {
    /// 从八卦枚举创建单卦
    pub fn new(bagua: Bagua) -> Self {
        Self { bagua }
    }

    /// 从数字创建单卦（1-8，0和8都对应坤卦）
    pub fn from_num(num: u8) -> Self {
        let bagua = match num % 8 {
            1 => Bagua::Qian,
            2 => Bagua::Dui,
            3 => Bagua::Li,
            4 => Bagua::Zhen,
            5 => Bagua::Xun,
            6 => Bagua::Kan,
            7 => Bagua::Gen,
            _ => Bagua::Kun, // 0 或 8
        };
        Self { bagua }
    }

    /// 获取二进制表示 (3 bits)
    pub fn binary(&self) -> u8 {
        match self.bagua {
            Bagua::Qian => 0b111, // 乾 ☰
            Bagua::Dui => 0b011,  // 兑 ☱
            Bagua::Li => 0b101,   // 离 ☲
            Bagua::Zhen => 0b001, // 震 ☳
            Bagua::Xun => 0b110,  // 巽 ☴
            Bagua::Kan => 0b010,  // 坎 ☵
            Bagua::Gen => 0b100,  // 艮 ☶
            Bagua::Kun => 0b000,  // 坤 ☷
        }
    }

    /// 获取五行属性
    pub fn wuxing(&self) -> WuXing {
        match self.bagua {
            Bagua::Qian | Bagua::Dui => WuXing::Jin,   // 金
            Bagua::Zhen | Bagua::Xun => WuXing::Mu,    // 木
            Bagua::Kan => WuXing::Shui,                 // 水
            Bagua::Li => WuXing::Huo,                   // 火
            Bagua::Gen | Bagua::Kun => WuXing::Tu,     // 土
        }
    }

    /// 获取自然象征
    pub fn nature_symbol(&self) -> &'static str {
        match self.bagua {
            Bagua::Qian => "天",
            Bagua::Dui => "泽",
            Bagua::Li => "火",
            Bagua::Zhen => "雷",
            Bagua::Xun => "风",
            Bagua::Kan => "水",
            Bagua::Gen => "山",
            Bagua::Kun => "地",
        }
    }

    /// 获取卦名（中文）
    pub fn name(&self) -> &'static str {
        match self.bagua {
            Bagua::Qian => "乾",
            Bagua::Dui => "兑",
            Bagua::Li => "离",
            Bagua::Zhen => "震",
            Bagua::Xun => "巽",
            Bagua::Kan => "坎",
            Bagua::Gen => "艮",
            Bagua::Kun => "坤",
        }
    }

    /// 获取 Unicode 符号
    pub fn symbol(&self) -> &'static str {
        match self.bagua {
            Bagua::Qian => "☰",
            Bagua::Dui => "☱",
            Bagua::Li => "☲",
            Bagua::Zhen => "☳",
            Bagua::Xun => "☴",
            Bagua::Kan => "☵",
            Bagua::Gen => "☶",
            Bagua::Kun => "☷",
        }
    }

    /// 从二进制创建单卦
    pub fn from_binary(binary: u8) -> Self {
        let bagua = match binary & 0b111 {
            0b111 => Bagua::Qian,
            0b011 => Bagua::Dui,
            0b101 => Bagua::Li,
            0b001 => Bagua::Zhen,
            0b110 => Bagua::Xun,
            0b010 => Bagua::Kan,
            0b100 => Bagua::Gen,
            _ => Bagua::Kun,
        };
        Self { bagua }
    }
}

/// 六十四卦结构（优化版）
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct Hexagram<AccountId, BlockNumber> {
    /// 卦象ID
    pub id: u64,
    /// 占卜者
    pub diviner: AccountId,
    /// 上卦
    pub shang_gua: SingleGua,
    /// 下卦
    pub xia_gua: SingleGua,
    /// 动爻 (1-6)
    pub dong_yao: u8,
    /// 体卦标识 (true=上卦为体, false=下卦为体)
    pub ti_is_shang: bool,
    /// 占卜问题哈希 (隐私保护)
    pub question_hash: [u8; 32],
    /// 起卦方式
    pub method: DivinationMethod,
    /// 起卦区块
    pub block_number: BlockNumber,
    /// 起卦时间戳
    pub timestamp: u64,
    /// AI 解读 IPFS CID (可选)
    pub interpretation_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    /// 是否公开
    pub is_public: bool,
}

impl<AccountId, BlockNumber> Hexagram<AccountId, BlockNumber> {
    /// 获取六十四卦名称
    pub fn hexagram_name(&self) -> &'static str {
        // 六十四卦名称表（上卦*8 + 下卦数）
        const HEXAGRAM_NAMES: [&str; 64] = [
            "坤为地", "地雷复", "地水师", "地泽临", "地山谦", "地火明夷", "地风升", "地天泰",
            "雷地豫", "震为雷", "雷水解", "雷泽归妹", "雷山小过", "雷火丰", "雷风恒", "雷天大壮",
            "水地比", "水雷屯", "坎为水", "水泽节", "水山蹇", "水火既济", "水风井", "水天需",
            "泽地萃", "泽雷随", "泽水困", "兑为泽", "泽山咸", "泽火革", "泽风大过", "泽天夬",
            "山地剥", "山雷颐", "山水蒙", "山泽损", "艮为山", "山火贲", "山风蛊", "山天大畜",
            "火地晋", "火雷噬嗑", "火水未济", "火泽睽", "火山旅", "离为火", "火风鼎", "火天大有",
            "风地观", "风雷益", "风水涣", "风泽中孚", "风山渐", "风火家人", "巽为风", "风天小畜",
            "天地否", "天雷无妄", "天水讼", "天泽履", "天山遁", "天火同人", "天风姤", "乾为天",
        ];

        let shang_num = match self.shang_gua.bagua {
            Bagua::Kun => 0, Bagua::Zhen => 1, Bagua::Kan => 2, Bagua::Dui => 3,
            Bagua::Gen => 4, Bagua::Li => 5, Bagua::Xun => 6, Bagua::Qian => 7,
        };
        let xia_num = match self.xia_gua.bagua {
            Bagua::Kun => 0, Bagua::Zhen => 1, Bagua::Kan => 2, Bagua::Dui => 3,
            Bagua::Gen => 4, Bagua::Li => 5, Bagua::Xun => 6, Bagua::Qian => 7,
        };

        HEXAGRAM_NAMES[shang_num * 8 + xia_num]
    }

    /// 获取完整的六爻二进制表示
    pub fn full_binary(&self) -> u8 {
        (self.shang_gua.binary() << 3) | self.xia_gua.binary()
    }
}

/// 完整卦象（含变卦、互卦）
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct FullDivination<AccountId, BlockNumber> {
    /// 本卦
    pub ben_gua: Hexagram<AccountId, BlockNumber>,
    /// 变卦
    pub bian_gua: Option<(SingleGua, SingleGua)>,
    /// 互卦
    pub hu_gua: (SingleGua, SingleGua),
    /// 体用关系（本卦）
    pub ben_gua_relation: TiYongRelation,
    /// 体用关系（变卦）
    pub bian_gua_relation: Option<TiYongRelation>,
    /// 综合吉凶判断
    pub fortune: Fortune,
}

/// 吉凶判断
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum Fortune {
    DaJi,      // 大吉
    XiaoJi,    // 小吉
    Ping,      // 平
    XiaoXiong, // 小凶
    DaXiong,   // 大凶
}
```

#### 1.2 存储设计

```rust
// pallets/meihua/src/lib.rs

#[pallet::storage]
pub type NextHexagramId<T> = StorageValue<_, u64, ValueQuery>;

/// 卦象存储 (HexagramId => FullDivination)
#[pallet::storage]
pub type Hexagrams<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64,
    FullDivination<T::AccountId, BlockNumberFor<T>>,
>;

/// 用户卦象索引 (AccountId => Vec<HexagramId>)
#[pallet::storage]
pub type UserHexagrams<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<u64, T::MaxUserHexagrams>,
    ValueQuery,
>;

/// 公开卦象列表
#[pallet::storage]
pub type PublicHexagrams<T: Config> = StorageValue<
    _,
    BoundedVec<u64, T::MaxPublicHexagrams>,
    ValueQuery,
>;

/// 六十四卦卦辞 (上卦数*8+下卦数 => 卦辞IPFS CID)
#[pallet::storage]
pub type GuaCi<T: Config> = StorageMap<
    _,
    Twox64Concat,
    u8,
    BoundedVec<u8, ConstU32<64>>,
>;

/// 每日占卜计数（防刷）
#[pallet::storage]
pub type DailyDivinationCount<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, T::AccountId,
    Twox64Concat, u32, // day number
    u32,
    ValueQuery,
>;

/// 错误类型
#[pallet::error]
pub enum Error<T> {
    /// 卦象不存在
    HexagramNotFound,
    /// 非卦象所有者
    NotOwner,
    /// 每日占卜次数超限
    DailyLimitExceeded,
    /// 年份超出支持范围（1900-2100）
    InvalidYear,
    /// 日期早于支持的最早日期
    DateTooEarly,
    /// 用户卦象列表已满
    UserHexagramsFull,
    /// 公开卦象列表已满
    PublicHexagramsFull,
}
```

#### 1.3 可调用函数

```rust
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// 时间起卦（使用农历）
    #[pallet::call_index(0)]
    #[pallet::weight(T::WeightInfo::divine_by_time())]
    pub fn divine_by_time(
        origin: OriginFor<T>,
        question_hash: [u8; 32],
        is_public: bool,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::check_daily_limit(&who)?;

        // 使用区块时间戳转换为农历
        let timestamp = T::UnixTime::now().as_secs();
        let lunar_date = Self::timestamp_to_lunar(timestamp)?;

        // 年数：使用地支数（子=1, 丑=2, ..., 亥=12）
        let year_num = lunar_date.year_zhi_num as u16;
        // 月数：农历月份（1-12）
        let month_num = lunar_date.month as u16;
        // 日数：农历日（1-30）
        let day_num = lunar_date.day as u16;
        // 时数：时辰地支数（子=1, 丑=2, ..., 亥=12）
        let hour_num = lunar_date.hour_zhi_num as u16;

        // 计算卦数（梅花易数传统公式）
        let shang_gua_num = Self::calc_gua_num((year_num + month_num + day_num) as u32);
        let xia_gua_num = Self::calc_gua_num((year_num + month_num + day_num + hour_num) as u32);
        let dong_yao = Self::calc_dong_yao((year_num + month_num + day_num + hour_num) as u32);

        Self::create_hexagram(
            who,
            shang_gua_num,
            xia_gua_num,
            dong_yao,
            DivinationMethod::DateTime,
            question_hash,
            is_public,
        )
    }

    /// 双数起卦
    #[pallet::call_index(1)]
    #[pallet::weight(T::WeightInfo::divine_by_numbers())]
    pub fn divine_by_numbers(
        origin: OriginFor<T>,
        num1: u8,
        num2: u8,
        question_hash: [u8; 32],
        is_public: bool,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::check_daily_limit(&who)?;

        // 获取当前时辰
        let timestamp = T::UnixTime::now().as_secs();
        let lunar_date = Self::timestamp_to_lunar(timestamp)?;
        let hour_num = lunar_date.hour_zhi_num as u32;

        // 计算卦数
        let shang_gua_num = Self::calc_gua_num(num1 as u32);
        let xia_gua_num = Self::calc_gua_num(num2 as u32);
        let dong_yao = Self::calc_dong_yao(num1 as u32 + num2 as u32 + hour_num);

        Self::create_hexagram(
            who,
            shang_gua_num,
            xia_gua_num,
            dong_yao,
            DivinationMethod::TwoNumbers { num1, num2 },
            question_hash,
            is_public,
        )
    }

    /// 随机起卦（使用链上随机数）
    #[pallet::call_index(2)]
    #[pallet::weight(T::WeightInfo::divine_random())]
    pub fn divine_random(
        origin: OriginFor<T>,
        question_hash: [u8; 32],
        is_public: bool,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;
        Self::check_daily_limit(&who)?;

        // 使用链上随机源
        let random_seed = T::Randomness::random(&b"meihua"[..]).0;
        let random_bytes: [u8; 32] = random_seed.as_ref().try_into().unwrap_or([0u8; 32]);

        let shang_gua_num = random_bytes[0] % 8;
        let xia_gua_num = random_bytes[1] % 8;
        let dong_yao = (random_bytes[2] % 6) + 1;

        Self::create_hexagram(
            who,
            shang_gua_num,
            xia_gua_num,
            dong_yao,
            DivinationMethod::Random,
            question_hash,
            is_public,
        )
    }

    /// 请求 AI 解卦
    #[pallet::call_index(3)]
    #[pallet::weight(T::WeightInfo::request_ai_interpretation())]
    pub fn request_ai_interpretation(
        origin: OriginFor<T>,
        hexagram_id: u64,
    ) -> DispatchResult {
        let who = ensure_signed(origin)?;

        let hexagram = Hexagrams::<T>::get(hexagram_id)
            .ok_or(Error::<T>::HexagramNotFound)?;

        ensure!(hexagram.ben_gua.diviner == who, Error::<T>::NotOwner);

        // 扣除 AI 解卦费用
        T::Currency::transfer(
            &who,
            &T::TreasuryAccount::get(),
            T::AiInterpretationFee::get(),
            ExistenceRequirement::KeepAlive,
        )?;

        // 触发链下工作机事件
        Self::deposit_event(Event::AiInterpretationRequested {
            hexagram_id,
            requester: who,
        });

        Ok(())
    }

    /// 提交 AI 解读结果（仅限授权节点）
    #[pallet::call_index(4)]
    #[pallet::weight(T::WeightInfo::submit_ai_interpretation())]
    pub fn submit_ai_interpretation(
        origin: OriginFor<T>,
        hexagram_id: u64,
        interpretation_cid: BoundedVec<u8, ConstU32<64>>,
    ) -> DispatchResult {
        T::AiOracleOrigin::ensure_origin(origin)?;

        Hexagrams::<T>::try_mutate(hexagram_id, |maybe_hexagram| {
            let hexagram = maybe_hexagram.as_mut()
                .ok_or(Error::<T>::HexagramNotFound)?;
            hexagram.ben_gua.interpretation_cid = Some(interpretation_cid.clone());
            Ok::<_, DispatchError>(())
        })?;

        Self::deposit_event(Event::AiInterpretationSubmitted {
            hexagram_id,
            cid: interpretation_cid,
        });

        Ok(())
    }
}
```

#### 1.4 农历转换模块

```rust
// pallets/meihua/src/lunar.rs

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// 地支枚举
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum DiZhi {
    Zi = 1,   // 子
    Chou = 2, // 丑
    Yin = 3,  // 寅
    Mao = 4,  // 卯
    Chen = 5, // 辰
    Si = 6,   // 巳
    Wu = 7,   // 午
    Wei = 8,  // 未
    Shen = 9, // 申
    You = 10, // 酉
    Xu = 11,  // 戌
    Hai = 12, // 亥
}

/// 农历日期结构
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct LunarDate {
    /// 农历年（如 2024）
    pub year: u16,
    /// 年地支数（子=1, 丑=2, ..., 亥=12）
    pub year_zhi_num: u8,
    /// 农历月（1-12，闰月为负数）
    pub month: i8,
    /// 农历日（1-30）
    pub day: u8,
    /// 时辰地支数（子=1, 丑=2, ..., 亥=12）
    pub hour_zhi_num: u8,
    /// 是否闰月
    pub is_leap_month: bool,
}

impl<T: Config> Pallet<T> {
    /// 时间戳转农历日期
    ///
    /// 农历算法核心：
    /// 1. 使用寿星万年历算法或查表法
    /// 2. 支持 1900-2100 年范围
    pub fn timestamp_to_lunar(timestamp: u64) -> Result<LunarDate, DispatchError> {
        // 转换为本地时间（UTC+8）
        let local_timestamp = timestamp + 8 * 3600;

        // 计算公历日期
        let days_since_epoch = local_timestamp / 86400;
        let (year, month, day) = Self::days_to_gregorian(days_since_epoch as i64);

        // 计算时辰（每两小时一个时辰，23:00-01:00为子时）
        let hour = ((local_timestamp % 86400) / 3600) as u8;
        let hour_zhi_num = Self::hour_to_zhi(hour);

        // 公历转农历（使用查表法）
        let (lunar_year, lunar_month, lunar_day, is_leap) =
            Self::gregorian_to_lunar(year, month, day)?;

        // 计算年地支
        let year_zhi_num = Self::year_to_zhi(lunar_year);

        Ok(LunarDate {
            year: lunar_year,
            year_zhi_num,
            month: if is_leap { -(lunar_month as i8) } else { lunar_month as i8 },
            day: lunar_day,
            hour_zhi_num,
            is_leap_month: is_leap,
        })
    }

    /// 计算年份对应的地支数
    /// 子=1, 丑=2, ..., 亥=12
    fn year_to_zhi(year: u16) -> u8 {
        // 1900年为庚子年，地支为子(1)
        let offset = (year as i32 - 1900) % 12;
        let zhi = if offset >= 0 { offset } else { offset + 12 };
        (zhi as u8) + 1
    }

    /// 小时转时辰地支数
    /// 23:00-01:00 子时(1), 01:00-03:00 丑时(2), ...
    fn hour_to_zhi(hour: u8) -> u8 {
        match hour {
            23 | 0 => 1,   // 子时
            1 | 2 => 2,    // 丑时
            3 | 4 => 3,    // 寅时
            5 | 6 => 4,    // 卯时
            7 | 8 => 5,    // 辰时
            9 | 10 => 6,   // 巳时
            11 | 12 => 7,  // 午时
            13 | 14 => 8,  // 未时
            15 | 16 => 9,  // 申时
            17 | 18 => 10, // 酉时
            19 | 20 => 11, // 戌时
            21 | 22 => 12, // 亥时
            _ => 1,
        }
    }

    /// 儒略日转公历日期
    fn days_to_gregorian(days: i64) -> (u16, u8, u8) {
        // 从1970-01-01起算
        let jd = days + 2440588; // 转为儒略日
        let a = jd + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - (146097 * b) / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - (1461 * d) / 4;
        let m = (5 * e + 2) / 153;

        let day = (e - (153 * m + 2) / 5 + 1) as u8;
        let month = (m + 3 - 12 * (m / 10)) as u8;
        let year = (100 * b + d - 4800 + m / 10) as u16;

        (year, month, day)
    }

    /// 公历转农历（查表法）
    ///
    /// 农历数据表：每年用16位表示
    /// - 高4位：闰月月份（0表示无闰月）
    /// - 低12位：每月大小月标记（1=30天，0=29天）
    fn gregorian_to_lunar(
        year: u16,
        month: u8,
        day: u8
    ) -> Result<(u16, u8, u8, bool), DispatchError> {
        // 农历数据表（1900-2100年）
        // 这里仅展示部分数据，完整数据需要约400个u32
        const LUNAR_INFO: [u32; 201] = [
            0x04bd8, 0x04ae0, 0x0a570, 0x054d5, 0x0d260, // 1900-1904
            0x0d950, 0x16554, 0x056a0, 0x09ad0, 0x055d2, // 1905-1909
            0x04ae0, 0x0a5b6, 0x0a4d0, 0x0d250, 0x1d255, // 1910-1914
            0x0b540, 0x0d6a0, 0x0ada2, 0x095b0, 0x14977, // 1915-1919
            0x04970, 0x0a4b0, 0x0b4b5, 0x06a50, 0x06d40, // 1920-1924
            // ... 完整数据省略，实际实现需要完整表
            // 可以从 https://github.com/6tail/lunar-rust 获取
            0x0, 0x0, 0x0, 0x0, 0x0, // 占位
            0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0,
            // ... 继续填充至201个元素
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];

        ensure!(year >= 1900 && year <= 2100, Error::<T>::InvalidYear);

        // 计算从1900年1月31日（农历正月初一）到目标日期的天数
        let base_date = Self::gregorian_to_days(1900, 1, 31);
        let target_date = Self::gregorian_to_days(year, month, day);
        let mut offset = target_date - base_date;

        ensure!(offset >= 0, Error::<T>::DateTooEarly);

        let mut lunar_year = 1900u16;
        let mut lunar_month = 1u8;
        let mut lunar_day = 1u8;
        let mut is_leap = false;

        // 计算农历年
        while lunar_year <= 2100 {
            let year_days = Self::lunar_year_days(LUNAR_INFO[(lunar_year - 1900) as usize]);
            if offset < year_days as i64 {
                break;
            }
            offset -= year_days as i64;
            lunar_year += 1;
        }

        // 计算农历月和日
        let info = LUNAR_INFO[(lunar_year - 1900) as usize];
        let leap_month = ((info >> 16) & 0xf) as u8;

        for m in 1..=12u8 {
            let month_days = if (info >> (16 - m)) & 1 == 1 { 30 } else { 29 };

            if offset < month_days {
                lunar_month = m;
                lunar_day = (offset + 1) as u8;
                break;
            }
            offset -= month_days;

            // 处理闰月
            if leap_month == m {
                let leap_days = if (info >> 16) & 0x10000 != 0 { 30 } else { 29 };
                if offset < leap_days {
                    lunar_month = m;
                    lunar_day = (offset + 1) as u8;
                    is_leap = true;
                    break;
                }
                offset -= leap_days;
            }
        }

        Ok((lunar_year, lunar_month, lunar_day, is_leap))
    }

    /// 计算农历年的总天数
    fn lunar_year_days(info: u32) -> u16 {
        let mut days = 0u16;
        for i in 0..12 {
            days += if (info >> (16 - i)) & 1 == 1 { 30 } else { 29 };
        }
        // 加上闰月天数
        let leap = (info >> 16) & 0xf;
        if leap > 0 {
            days += if (info >> 16) & 0x10000 != 0 { 30 } else { 29 };
        }
        days
    }

    /// 公历日期转天数（从公元1年起算）
    fn gregorian_to_days(year: u16, month: u8, day: u8) -> i64 {
        let y = year as i64;
        let m = month as i64;
        let d = day as i64;

        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;

        d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
    }

    /// 计算卦数（处理余数为0的情况）
    /// 余数为0时返回8（坤卦）
    #[inline]
    pub fn calc_gua_num(n: u32) -> u8 {
        let r = (n % 8) as u8;
        if r == 0 { 8 } else { r }
    }

    /// 计算动爻数（处理余数为0的情况）
    /// 余数为0时返回6（上爻）
    #[inline]
    pub fn calc_dong_yao(n: u32) -> u8 {
        let r = (n % 6) as u8;
        if r == 0 { 6 } else { r }
    }
}
```

#### 1.5 核心算法实现

```rust
// pallets/meihua/src/algorithm.rs

impl<T: Config> Pallet<T> {
    /// 数字转八卦
    pub fn num_to_bagua(num: u8) -> Bagua {
        match num % 8 {
            1 => Bagua::Qian,
            2 => Bagua::Dui,
            3 => Bagua::Li,
            4 => Bagua::Zhen,
            5 => Bagua::Xun,
            6 => Bagua::Kan,
            7 => Bagua::Gen,
            _ => Bagua::Kun, // 0 or 8
        }
    }

    /// 八卦转二进制
    pub fn bagua_to_binary(bagua: &Bagua) -> u8 {
        match bagua {
            Bagua::Qian => 0b111, // 乾
            Bagua::Dui => 0b011,  // 兑
            Bagua::Li => 0b101,   // 离
            Bagua::Zhen => 0b001, // 震
            Bagua::Xun => 0b110,  // 巽
            Bagua::Kan => 0b010,  // 坎
            Bagua::Gen => 0b100,  // 艮
            Bagua::Kun => 0b000,  // 坤
        }
    }

    /// 二进制转八卦
    pub fn binary_to_bagua(binary: u8) -> Bagua {
        match binary & 0b111 {
            0b111 => Bagua::Qian,
            0b011 => Bagua::Dui,
            0b101 => Bagua::Li,
            0b001 => Bagua::Zhen,
            0b110 => Bagua::Xun,
            0b010 => Bagua::Kan,
            0b100 => Bagua::Gen,
            _ => Bagua::Kun,
        }
    }

    /// 八卦转五行
    pub fn bagua_to_wuxing(bagua: &Bagua) -> WuXing {
        match bagua {
            Bagua::Qian | Bagua::Dui => WuXing::Jin,   // 金
            Bagua::Zhen | Bagua::Xun => WuXing::Mu,    // 木
            Bagua::Kan => WuXing::Shui,                 // 水
            Bagua::Li => WuXing::Huo,                   // 火
            Bagua::Gen | Bagua::Kun => WuXing::Tu,     // 土
        }
    }

    /// 计算体用关系
    pub fn calc_tiyong_relation(ti: &WuXing, yong: &WuXing) -> TiYongRelation {
        use WuXing::*;
        match (ti, yong) {
            // 比和 - 同五行
            (a, b) if a == b => TiYongRelation::BiHe,

            // 生 - 用生体
            (Jin, Tu) | (Mu, Shui) | (Shui, Jin) | (Huo, Mu) | (Tu, Huo)
                => TiYongRelation::Sheng,

            // 被生 - 体生用
            (Tu, Jin) | (Shui, Mu) | (Jin, Shui) | (Mu, Huo) | (Huo, Tu)
                => TiYongRelation::BeSheng,

            // 克 - 用克体
            (Jin, Huo) | (Mu, Jin) | (Shui, Tu) | (Huo, Shui) | (Tu, Mu)
                => TiYongRelation::Ke,

            // 被克 - 体克用
            (Huo, Jin) | (Jin, Mu) | (Tu, Shui) | (Shui, Huo) | (Mu, Tu)
                => TiYongRelation::BeKe,

            _ => TiYongRelation::BiHe, // fallback
        }
    }

    /// 计算变卦（优化版 - 使用简化的 SingleGua）
    pub fn calc_bian_gua(
        shang_gua: &SingleGua,
        xia_gua: &SingleGua,
        dong_yao: u8,
    ) -> (SingleGua, SingleGua) {
        // 6爻：上卦3爻(6,5,4) + 下卦3爻(3,2,1)
        let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

        // 翻转动爻位
        let bit_position = dong_yao - 1; // 0-indexed
        let flipped = full_binary ^ (1 << bit_position);

        let new_shang_binary = (flipped >> 3) & 0b111;
        let new_xia_binary = flipped & 0b111;

        (
            SingleGua::from_binary(new_shang_binary),
            SingleGua::from_binary(new_xia_binary),
        )
    }

    /// 计算互卦（优化版 - 使用简化的 SingleGua）
    pub fn calc_hu_gua(shang_gua: &SingleGua, xia_gua: &SingleGua) -> (SingleGua, SingleGua) {
        // 本卦6爻：上卦(6,5,4) + 下卦(3,2,1)
        // 互卦上卦：取本卦5,4,3爻
        // 互卦下卦：取本卦4,3,2爻

        let full_binary = (shang_gua.binary() << 3) | xia_gua.binary();

        // 上互卦：5,4,3爻 (bits 4,3,2)
        let hu_shang = (full_binary >> 2) & 0b111;

        // 下互卦：4,3,2爻 (bits 3,2,1)
        let hu_xia = (full_binary >> 1) & 0b111;

        (
            SingleGua::from_binary(hu_shang),
            SingleGua::from_binary(hu_xia),
        )
    }

    /// 判断体用卦
    pub fn determine_ti_gua(dong_yao: u8) -> bool {
        // 动爻在上卦(4,5,6)，上卦为用，下卦为体
        // 动爻在下卦(1,2,3)，下卦为用，上卦为体
        dong_yao > 3
    }

    /// 综合吉凶判断
    pub fn calc_fortune(
        ben_relation: &TiYongRelation,
        bian_relation: &Option<TiYongRelation>,
    ) -> Fortune {
        use TiYongRelation::*;
        use Fortune::*;

        let ben_score = match ben_relation {
            BiHe => 3,
            Sheng => 4,
            BeSheng => 2,
            Ke => 0,
            BeKe => 1,
        };

        let bian_score = bian_relation.as_ref().map(|r| match r {
            BiHe => 3,
            Sheng => 4,
            BeSheng => 2,
            Ke => 0,
            BeKe => 1,
        }).unwrap_or(3);

        let total = ben_score + bian_score;

        match total {
            7..=8 => DaJi,
            5..=6 => XiaoJi,
            4 => Ping,
            2..=3 => XiaoXiong,
            _ => DaXiong,
        }
    }
}
```

---

### 2. pallet-meihua-ai（AI 解卦模块）

#### 2.1 设计目标

- 链下 AI 推理，链上结果存证
- 支持多种 AI 模型（GPT、本地模型等）
- 流式对话能力
- 解读质量评分机制

#### 2.2 数据结构

```rust
// pallets/meihua-ai/src/types.rs

/// AI 解读请求
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct InterpretationRequest<AccountId, BlockNumber> {
    pub id: u64,
    pub hexagram_id: u64,
    pub requester: AccountId,
    pub model_type: AiModelType,
    pub status: RequestStatus,
    pub created_at: BlockNumber,
    pub completed_at: Option<BlockNumber>,
}

/// AI 模型类型
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub enum AiModelType {
    /// 基础模型（快速）
    Basic,
    /// 高级模型（详细）
    Advanced,
    /// 对话模式
    Dialogue,
}

/// 请求状态
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// AI 解读结果
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct InterpretationResult {
    /// 卦象总论
    pub overview: BoundedVec<u8, ConstU32<512>>,
    /// 本卦解读 IPFS CID
    pub ben_gua_cid: BoundedVec<u8, ConstU32<64>>,
    /// 变卦解读 IPFS CID
    pub bian_gua_cid: Option<BoundedVec<u8, ConstU32<64>>>,
    /// 行动建议
    pub advice: BoundedVec<u8, ConstU32<256>>,
    /// 解读质量评分 (0-100)
    pub quality_score: u8,
}
```

#### 2.3 AI 推理服务 API

```python
# ai-service/src/meihua_interpreter.py

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Optional
import openai

app = FastAPI()

# 系统提示词
SYS_PROMPT = """你是一位精通梅花易数的占卜大师。请根据提供的卦象信息进行详细解读。

解读要求：
1. 分析本卦、变卦、互卦的象义
2. 结合体用关系判断吉凶
3. 针对具体问题给出行动建议
4. 语言要通俗易懂，避免过于晦涩

卦象象征：
- 乾(☰)：天、父、首领、刚健
- 兑(☱)：泽、少女、喜悦、口舌
- 离(☲)：火、中女、文明、附丽
- 震(☳)：雷、长男、行动、震动
- 巽(☴)：风、长女、柔顺、进入
- 坎(☵)：水、中男、险难、智慧
- 艮(☶)：山、少男、止、稳定
- 坤(☷)：地、母、柔顺、包容

五行生克：
- 相生：金生水、水生木、木生火、火生土、土生金
- 相克：金克木、木克土、土克水、水克火、火克金
"""

class HexagramData(BaseModel):
    hexagram_id: int
    shang_gua: str       # 上卦名
    xia_gua: str         # 下卦名
    dong_yao: int        # 动爻
    bian_shang_gua: str  # 变卦上卦
    bian_xia_gua: str    # 变卦下卦
    hu_shang_gua: str    # 互卦上卦
    hu_xia_gua: str      # 互卦下卦
    ti_yong_relation: str  # 体用关系
    question: Optional[str] = None  # 占卜问题（可选）

class InterpretationResponse(BaseModel):
    overview: str
    ben_gua_analysis: str
    bian_gua_analysis: str
    hu_gua_analysis: str
    advice: str
    fortune: str

# 六十四卦卦辞
GUA_CI = {
    "乾乾": "乾为天：刚健中正，自强不息",
    "乾兑": "天泽履：脚踏实地，小心谨慎",
    # ... 完整64卦
}

@app.post("/api/v1/interpret")
async def interpret_hexagram(data: HexagramData) -> InterpretationResponse:
    """解读卦象"""

    # 构建卦象描述
    ben_gua_name = f"{data.shang_gua}{data.xia_gua}"
    bian_gua_name = f"{data.bian_shang_gua}{data.bian_xia_gua}"
    hu_gua_name = f"{data.hu_shang_gua}{data.hu_xia_gua}"

    prompt = f"""
请解读以下梅花易数卦象：

【本卦】{ben_gua_name}
- 上卦：{data.shang_gua}
- 下卦：{data.xia_gua}
- 动爻：第{data.dong_yao}爻
- 卦辞：{GUA_CI.get(ben_gua_name, "未知")}

【变卦】{bian_gua_name}
- 上卦：{data.bian_shang_gua}
- 下卦：{data.bian_xia_gua}
- 卦辞：{GUA_CI.get(bian_gua_name, "未知")}

【互卦】{hu_gua_name}
- 上卦：{data.hu_shang_gua}
- 下卦：{data.hu_xia_gua}

【体用关系】{data.ti_yong_relation}

{"【占卜问题】" + data.question if data.question else ""}

请从以下几个方面进行解读：
1. 卦象总论（100字以内）
2. 本卦分析（详细）
3. 变卦分析（事情发展趋势）
4. 互卦分析（内在因素）
5. 行动建议（具体可操作）
6. 吉凶判断
"""

    response = await openai.ChatCompletion.acreate(
        model="gpt-4",
        messages=[
            {"role": "system", "content": SYS_PROMPT},
            {"role": "user", "content": prompt}
        ],
        temperature=0.7,
        max_tokens=2000
    )

    # 解析响应
    content = response.choices[0].message.content

    # TODO: 更精细的解析逻辑
    return InterpretationResponse(
        overview=extract_section(content, "卦象总论"),
        ben_gua_analysis=extract_section(content, "本卦分析"),
        bian_gua_analysis=extract_section(content, "变卦分析"),
        hu_gua_analysis=extract_section(content, "互卦分析"),
        advice=extract_section(content, "行动建议"),
        fortune=extract_section(content, "吉凶判断"),
    )


@app.post("/api/v1/dialogue")
async def dialogue_interpret(
    hexagram_id: int,
    user_message: str,
    conversation_history: list[dict]
):
    """对话式解卦"""

    # 获取卦象上下文
    hexagram_context = await get_hexagram_context(hexagram_id)

    messages = [
        {"role": "system", "content": SYS_PROMPT + f"\n\n当前卦象：{hexagram_context}"},
        *conversation_history,
        {"role": "user", "content": user_message}
    ]

    response = await openai.ChatCompletion.acreate(
        model="gpt-4",
        messages=messages,
        temperature=0.7,
        stream=True
    )

    # 流式返回
    async def generate():
        async for chunk in response:
            if chunk.choices[0].delta.content:
                yield chunk.choices[0].delta.content

    return StreamingResponse(generate(), media_type="text/event-stream")
```

---

### 3. pallet-meihua-market（占卜服务市场）

#### 3.1 功能设计

- 占卜师注册与认证
- 服务定价与订单管理
- 评价与信誉系统
- 收益分成机制

#### 3.2 核心存储

```rust
// pallets/meihua-market/src/lib.rs

/// 占卜师信息
#[pallet::storage]
pub type Diviners<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    DivinerProfile<T::AccountId>,
>;

/// 占卜服务
#[pallet::storage]
pub type Services<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // service_id
    DivinationService<T::AccountId, BalanceOf<T>>,
>;

/// 订单
#[pallet::storage]
pub type Orders<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    u64, // order_id
    Order<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
>;

/// 评价
#[pallet::storage]
pub type Reviews<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat, u64, // service_id
    Blake2_128Concat, T::AccountId, // reviewer
    Review<T::AccountId>,
>;
```

#### 3.3 订单流程

```
用户下单 → 支付锁定 → 占卜师接单 → 排盘解读 → 用户确认 → 结算分成
    │                                               │
    └── 超时未接 → 自动退款              超时未确认 → 自动完成
```

---

### 4. pallet-meihua-nft（卦象 NFT）

#### 4.1 功能设计

- 将有意义的卦象铸造为 NFT
- 支持卦象艺术化展示
- 卦象收藏与交易

#### 4.2 NFT 元数据

```rust
/// 卦象 NFT 元数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct HexagramNftMetadata {
    /// 关联的卦象ID
    pub hexagram_id: u64,
    /// NFT 名称
    pub name: BoundedVec<u8, ConstU32<64>>,
    /// 描述
    pub description: BoundedVec<u8, ConstU32<256>>,
    /// 艺术图 IPFS CID
    pub image_cid: BoundedVec<u8, ConstU32<64>>,
    /// 稀有度
    pub rarity: NftRarity,
    /// 铸造时间
    pub minted_at: u64,
}

#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum NftRarity {
    Common,      // 普通
    Uncommon,    // 不常见
    Rare,        // 稀有
    Epic,        // 史诗
    Legendary,   // 传说
}
```

---

## API 接口设计

### 1. RPC 扩展

```rust
// runtime-api/src/lib.rs

sp_api::decl_runtime_apis! {
    pub trait MeihuaApi<AccountId, BlockNumber> {
        /// 获取卦象详情
        fn get_hexagram(id: u64) -> Option<FullDivination<AccountId, BlockNumber>>;

        /// 获取用户卦象列表
        fn get_user_hexagrams(account: AccountId) -> Vec<u64>;

        /// 预览起卦结果（不上链）
        fn preview_divination(
            method: DivinationMethod,
        ) -> FullDivination<AccountId, BlockNumber>;

        /// 获取六十四卦卦辞
        fn get_gua_ci(shang: u8, xia: u8) -> Option<Vec<u8>>;

        /// 获取每日占卜剩余次数
        fn get_daily_remaining(account: AccountId) -> u32;
    }
}
```

### 2. HTTP API（AI 服务）

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/v1/interpret` | POST | 获取卦象解读 |
| `/api/v1/dialogue` | POST | 对话式解卦 |
| `/api/v1/advice` | POST | 获取行动建议 |
| `/api/v1/fortune` | GET | 获取今日运势 |
| `/api/v1/models` | GET | 获取可用模型列表 |

### 3. GraphQL（Subsquid 索引）

```graphql
type Hexagram @entity {
  id: ID!
  diviner: Account!
  shangGua: String!
  xiaGua: String!
  dongYao: Int!
  method: String!
  fortune: String!
  isPublic: Boolean!
  createdAt: DateTime!
  interpretationCid: String
}

type Query {
  hexagrams(
    where: HexagramWhereInput
    orderBy: [HexagramOrderByInput!]
    limit: Int
    offset: Int
  ): [Hexagram!]!

  hexagramById(id: ID!): Hexagram

  publicHexagrams(limit: Int): [Hexagram!]!

  userHexagrams(account: String!): [Hexagram!]!

  hexagramStats: HexagramStats!
}
```

---

## 前端 DApp 设计

### 1. 页面结构

```
/                      # 首页 - 快速起卦
/divine                # 起卦页面
  /time               # 时间起卦
  /number             # 数字起卦
  /random             # 随机起卦
/hexagram/:id         # 卦象详情页
/history              # 历史记录
/ai-chat/:id          # AI 对话解卦
/market               # 占卜师市场
/profile              # 个人中心
/nft                  # 卦象 NFT 收藏
```

### 2. 起卦界面组件

```tsx
// components/DivinationForm.tsx

import { useState } from 'react';
import { useApi, useAccounts } from '@polkadot/react-hooks';

interface DivinationFormProps {
  method: 'time' | 'number' | 'random';
  onSuccess: (hexagramId: number) => void;
}

export function DivinationForm({ method, onSuccess }: DivinationFormProps) {
  const { api } = useApi();
  const { activeAccount } = useAccounts();
  const [num1, setNum1] = useState<number>();
  const [num2, setNum2] = useState<number>();
  const [question, setQuestion] = useState('');
  const [isPublic, setIsPublic] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleDivine = async () => {
    if (!api || !activeAccount) return;

    setLoading(true);

    // 计算问题哈希（隐私保护）
    const questionHash = await crypto.subtle.digest(
      'SHA-256',
      new TextEncoder().encode(question)
    );

    try {
      let tx;

      switch (method) {
        case 'time':
          tx = api.tx.meihua.divineByTime(
            new Uint8Array(questionHash),
            isPublic
          );
          break;
        case 'number':
          tx = api.tx.meihua.divineByNumbers(
            num1!,
            num2!,
            new Uint8Array(questionHash),
            isPublic
          );
          break;
        case 'random':
          tx = api.tx.meihua.divineRandom(
            new Uint8Array(questionHash),
            isPublic
          );
          break;
      }

      const result = await tx.signAndSend(activeAccount);

      // 从事件中获取卦象ID
      const hexagramId = extractHexagramId(result.events);
      onSuccess(hexagramId);

    } catch (error) {
      console.error('Divination failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="divination-form">
      <h2>梅花易数排盘</h2>

      {method === 'number' && (
        <div className="number-inputs">
          <input
            type="number"
            min="1"
            max="999"
            placeholder="第一个数字"
            value={num1}
            onChange={(e) => setNum1(Number(e.target.value))}
          />
          <input
            type="number"
            min="1"
            max="999"
            placeholder="第二个数字"
            value={num2}
            onChange={(e) => setNum2(Number(e.target.value))}
          />
        </div>
      )}

      <textarea
        placeholder="请输入您的问题（可选，仅存储哈希值保护隐私）"
        value={question}
        onChange={(e) => setQuestion(e.target.value)}
      />

      <label>
        <input
          type="checkbox"
          checked={isPublic}
          onChange={(e) => setIsPublic(e.target.checked)}
        />
        公开此卦象
      </label>

      <button onClick={handleDivine} disabled={loading}>
        {loading ? '起卦中...' : '起卦'}
      </button>
    </div>
  );
}
```

### 3. 卦象展示组件

```tsx
// components/HexagramDisplay.tsx

interface HexagramDisplayProps {
  hexagram: FullDivination;
  showInterpretation?: boolean;
}

export function HexagramDisplay({ hexagram, showInterpretation }: HexagramDisplayProps) {
  const baguaSymbols: Record<string, string> = {
    'Qian': '☰', 'Dui': '☱', 'Li': '☲', 'Zhen': '☳',
    'Xun': '☴', 'Kan': '☵', 'Gen': '☶', 'Kun': '☷'
  };

  const wuxingColors: Record<string, string> = {
    'Jin': '#FFD700', 'Mu': '#228B22', 'Shui': '#1E90FF',
    'Huo': '#FF4500', 'Tu': '#8B4513'
  };

  return (
    <div className="hexagram-display">
      {/* 本卦 */}
      <div className="gua-section ben-gua">
        <h3>本卦</h3>
        <div className="gua-symbols">
          <span className="shang-gua">
            {baguaSymbols[hexagram.ben_gua.shang_gua.bagua]}
          </span>
          <span className="xia-gua">
            {baguaSymbols[hexagram.ben_gua.xia_gua.bagua]}
          </span>
        </div>
        <div className="gua-info">
          <p>上卦：{hexagram.ben_gua.shang_gua.bagua}
            <span style={{ color: wuxingColors[hexagram.ben_gua.shang_gua.wuxing] }}>
              ({hexagram.ben_gua.shang_gua.wuxing})
            </span>
            {hexagram.ben_gua.ti_is_shang ? ' [体]' : ' [用]'}
          </p>
          <p>下卦：{hexagram.ben_gua.xia_gua.bagua}
            <span style={{ color: wuxingColors[hexagram.ben_gua.xia_gua.wuxing] }}>
              ({hexagram.ben_gua.xia_gua.wuxing})
            </span>
            {!hexagram.ben_gua.ti_is_shang ? ' [体]' : ' [用]'}
          </p>
          <p>动爻：第 {hexagram.ben_gua.dong_yao} 爻</p>
          <p>体用关系：{hexagram.ben_gua_relation}</p>
        </div>
      </div>

      {/* 变卦 */}
      {hexagram.bian_gua && (
        <div className="gua-section bian-gua">
          <h3>变卦</h3>
          <div className="gua-symbols">
            <span>{baguaSymbols[hexagram.bian_gua[0].bagua]}</span>
            <span>{baguaSymbols[hexagram.bian_gua[1].bagua]}</span>
          </div>
          <p>体用关系：{hexagram.bian_gua_relation}</p>
        </div>
      )}

      {/* 互卦 */}
      <div className="gua-section hu-gua">
        <h3>互卦</h3>
        <div className="gua-symbols">
          <span>{baguaSymbols[hexagram.hu_gua[0].bagua]}</span>
          <span>{baguaSymbols[hexagram.hu_gua[1].bagua]}</span>
        </div>
      </div>

      {/* 综合判断 */}
      <div className={`fortune fortune-${hexagram.fortune.toLowerCase()}`}>
        <h3>综合判断</h3>
        <span className="fortune-text">{hexagram.fortune}</span>
      </div>

      {/* AI 解读 */}
      {showInterpretation && hexagram.ben_gua.interpretation_cid && (
        <div className="ai-interpretation">
          <h3>AI 解读</h3>
          <IpfsContent cid={hexagram.ben_gua.interpretation_cid} />
        </div>
      )}
    </div>
  );
}
```

---

## 经济模型

### 1. 代币使用场景

| 场景 | DUST 消耗 | 说明 |
|------|-----------|------|
| 基础起卦 | 免费 | 每日限3次 |
| 超额起卦 | 1 DUST/次 | 超出免费额度 |
| AI 基础解读 | 5 DUST | 简短解读 |
| AI 详细解读 | 15 DUST | 完整分析 |
| AI 对话 | 2 DUST/轮 | 追问互动 |
| 卦象 NFT 铸造 | 10 DUST | 铸造费 |
| 占卜师认证 | 100 DUST | 一次性 |

### 2. 收益分配

```
用户付费 100 DUST
    │
    ├── 70% → 占卜师/AI服务
    ├── 15% → 国库
    ├── 10% → 销毁（通缩）
    └──  5% → 推荐人（如有）
```

---

## 安全考虑

### 1. 防刷机制

```rust
/// 每日占卜限制
const MAX_DAILY_DIVINATIONS: u32 = 50;

/// 防刷检查
fn check_daily_limit(who: &T::AccountId) -> DispatchResult {
    let today = Self::current_day();
    let count = DailyDivinationCount::<T>::get(who, today);

    ensure!(
        count < T::MaxDailyDivinations::get(),
        Error::<T>::DailyLimitExceeded
    );

    DailyDivinationCount::<T>::insert(who, today, count + 1);
    Ok(())
}
```

### 2. 隐私保护

- 问题内容只存储哈希值
- 支持设置卦象为私密
- AI 服务不存储原始问题

### 3. 随机数安全

- 使用 Substrate 的 `Randomness` trait
- 结合区块哈希和时间戳
- 防止预测和操纵

---

## 部署架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        负载均衡器                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│  Node 1       │     │  Node 2       │     │  Node 3       │
│  (Validator)  │     │  (Validator)  │     │  (RPC)        │
└───────────────┘     └───────────────┘     └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   AI 服务集群    │
                    │   (FastAPI)     │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   IPFS 网关      │
                    └─────────────────┘
```

---

## 开发路线图

### Phase 1: 核心功能（4周）

- [ ] pallet-meihua 核心排盘逻辑
- [ ] 基础前端界面
- [ ] 时间/数字/随机起卦
- [ ] 卦象展示与存储

### Phase 2: AI 集成（3周）

- [ ] AI 推理服务部署
- [ ] pallet-meihua-ai 模块
- [ ] 对话式解卦
- [ ] IPFS 存储集成

### Phase 3: 市场功能（3周）

- [ ] 占卜师注册系统
- [ ] 订单与支付流程
- [ ] 评价与信誉系统

### Phase 4: NFT 与优化（2周）

- [ ] 卦象 NFT 铸造
- [ ] 性能优化
- [ ] 移动端适配

---

## 附录

### A. 八卦速查表

| 卦名 | 数字 | 二进制 | 五行 | 自然 | Unicode |
|------|------|--------|------|------|---------|
| 乾 | 1 | 111 | 金 | 天 | ☰ |
| 兑 | 2 | 011 | 金 | 泽 | ☱ |
| 离 | 3 | 101 | 火 | 火 | ☲ |
| 震 | 4 | 001 | 木 | 雷 | ☳ |
| 巽 | 5 | 110 | 木 | 风 | ☴ |
| 坎 | 6 | 010 | 水 | 水 | ☵ |
| 艮 | 7 | 100 | 土 | 山 | ☶ |
| 坤 | 0/8 | 000 | 土 | 地 | ☷ |

### B. 五行生克关系表

```
     金    木    水    火    土
金   比和  克    生    被克  被生
木   被克  比和  被生  生    克
水   被生  生    比和  克    被克
火   克    被生  被克  比和  生
土   生    被克  克    被生  比和
```

### C. 参考资源

- [梅花易数原文](https://zh.wikisource.org/wiki/梅花易數)
- [Substrate 文档](https://docs.substrate.io/)
- [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)

---

*文档版本: 1.0.0*
*最后更新: 2025-11-29*
*作者: Stardust Team*
