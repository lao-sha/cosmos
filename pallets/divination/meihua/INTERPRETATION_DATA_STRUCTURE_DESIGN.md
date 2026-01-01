# 梅花易数解卦数据结构设计文档

## 一、对比分析总结

### 1.1 xuanxue/meihua 项目分析

通过分析 `xuan-utils-pro` Java 实现，发现其梅花易数系统包含以下核心数据：

#### 基础信息
- **姓名、占事、性别、年龄**
- **公历/农历日期**（多种格式）
- **四柱干支**：年月日时的天干地支
- **五行、纳音、空亡**
- **节气信息**：上下节气、距离天数
- **月相、月将、星座、生肖**

#### 卦象信息
- **上卦、下卦、本卦、变卦、互卦、错卦、综卦**
- **卦名、卦象符号、卦辞**
- **六爻爻名、爻象、爻辞**
- **动爻位置**（1-6）
- **卦码**（唯一标识）

#### 体用关系
- **体卦、用卦判断**（动爻在哪卦，哪卦为用）
- **五行生克关系**
- **体用关系描述**：
  - 用生体（大吉）
  - 比和（次吉）
  - 体克用（中平）
  - 体生用（小凶）
  - 用克体（大凶）

#### 排盘模式
- 日期起卦（年月日时）
- 自动起卦（随机）
- 数字起卦（3位数）
- 单数起卦（多位数拆分）
- 双数起卦（两个数字）

### 1.2 pallets/divination/meihua 现有结构

当前区块链实现已包含：

#### 核心类型（types.rs）
- `Bagua`：八卦枚举（乾、兑、离、震、巽、坎、艮、坤）
- `WuXing`：五行枚举（金、木、水、火、土）
- `TiYongRelation`：体用关系枚举
- `DivinationMethod`：起卦方式枚举
- `SingleGua`：单卦结构（仅存储 Bagua，其他属性可推导）
- `Hexagram`：六十四卦结构
- `FullDivination`：完整卦象（本卦、变卦、互卦）
- `HexagramDetail`：卦象详细信息（含文本）
- `FullDivinationDetail`：完整排盘详细信息
- `WangShuai`：五行旺衰状态（旺、相、休、囚、死）
- `YingQiResult`：应期推算结果
- `Season`：季节枚举

#### 核心算法（algorithm.rs）
- 时间起卦、双数起卦、单数起卦、随机起卦
- 变卦、互卦、错卦、综卦、伏卦计算
- 体用判断、吉凶判断
- 卦气旺衰计算
- 应期推算

#### 常量数据（constants.rs）
- 六十四卦名称、卦辞、爻辞
- 八卦名称、符号、五行
- 体用关系名称、吉凶名称

### 1.3 差异对比

| 功能模块 | xuanxue/meihua | pallets/meihua | 差异说明 |
|---------|----------------|----------------|----------|
| **基础信息** | ✅ 完整（姓名、占事、性别等） | ❌ 缺失 | 区块链版本未存储占卜者个人信息 |
| **四柱干支** | ✅ 完整 | ⚠️ 部分（仅用于起卦） | 未作为解卦数据存储 |
| **节气信息** | ✅ 完整 | ⚠️ 部分（仅用于旺衰） | 未详细存储节气数据 |
| **卦象计算** | ✅ 完整 | ✅ 完整 | 功能一致 |
| **体用关系** | ✅ 完整 | ✅ 完整 | 功能一致 |
| **五行旺衰** | ❌ 缺失 | ✅ 完整 | 区块链版本更完善 |
| **应期推算** | ❌ 缺失 | ✅ 完整 | 区块链版本独有 |
| **错卦综卦** | ✅ 完整 | ✅ 完整 | 功能一致 |
| **伏卦** | ❌ 缺失 | ✅ 完整 | 区块链版本新增 |
| **AI解读** | ❌ 缺失 | ✅ 完整 | 区块链版本独有 |

---

## 二、解卦数据结构设计

### 2.1 设计原则

1. **链上存储最小化**：仅存储核心数据，可推导数据通过 Runtime API 计算
2. **隐私保护**：敏感信息（问题、姓名）仅存储哈希值
3. **可扩展性**：预留扩展字段，支持未来功能升级
4. **AI友好**：结构化数据便于AI解读和分析
5. **前端友好**：提供完整的查询API，减少前端计算负担

### 2.2 核心数据结构

#### 2.2.1 解卦基础信息（InterpretationBasicInfo）

```rust
/// 解卦基础信息
///
/// 存储占卜的基础上下文信息，用于AI解读和人工分析
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct InterpretationBasicInfo {
    /// 占卜时间戳（Unix秒）
    pub timestamp: u64,

    /// 农历年月日时（用于旺衰判断）
    pub lunar_date: LunarDateInfo,

    /// 起卦方式
    pub method: DivinationMethod,

    /// 占卜者性别（可选，用于某些流派的解卦）
    /// 0: 未指定, 1: 男, 2: 女
    pub gender: u8,

    /// 占卜类别（可选）
    /// 0: 未指定, 1: 事业, 2: 财运, 3: 感情, 4: 健康, 5: 学业, 6: 其他
    pub category: u8,
}

/// 农历日期信息（精简版）
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct LunarDateInfo {
    /// 农历年份
    pub year: u16,
    /// 农历月份（1-12）
    pub month: u8,
    /// 农历日（1-30）
    pub day: u8,
    /// 时辰地支数（1-12）
    pub hour_zhi_num: u8,
    /// 是否闰月
    pub is_leap_month: bool,
}
```

#### 2.2.2 卦象核心数据（HexagramCoreData）

```rust
/// 卦象核心数据
///
/// 存储排盘的核心结果，所有其他信息可从此推导
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct HexagramCoreData {
    /// 上卦（外卦）
    pub shang_gua: SingleGua,

    /// 下卦（内卦）
    pub xia_gua: SingleGua,

    /// 动爻位置（1-6）
    pub dong_yao: u8,

    /// 体卦位置：true=上卦为体，false=下卦为体
    pub ti_is_shang: bool,
}
```

#### 2.2.3 体用分析结果（TiYongAnalysis）

```rust
/// 体用分析结果
///
/// 梅花易数核心：体用关系决定吉凶
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct TiYongAnalysis {
    /// 体卦五行
    pub ti_wuxing: WuXing,

    /// 用卦五行
    pub yong_wuxing: WuXing,

    /// 本卦体用关系
    pub ben_gua_relation: TiYongRelation,

    /// 变卦体用关系
    pub bian_gua_relation: TiYongRelation,

    /// 互卦体用关系
    pub hu_gua_relation: TiYongRelation,

    /// 体卦旺衰状态
    pub ti_wangshuai: WangShuai,

    /// 综合吉凶判断
    pub fortune: Fortune,

    /// 吉凶等级（0-4，4最吉）
    pub fortune_level: u8,
}
```

#### 2.2.4 应期推算结果（YingQiAnalysis）

```rust
/// 应期推算结果
///
/// 预测事情应验的时间
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct YingQiAnalysis {
    /// 体卦卦数
    pub ti_gua_num: u8,

    /// 用卦卦数
    pub yong_gua_num: u8,

    /// 主要应期数（基于体用卦数）
    pub primary_num: u8,

    /// 次要应期数（基于五行卦数）
    pub secondary_nums: [u8; 2],

    /// 生体五行（喜神）
    pub sheng_ti_wuxing: WuXing,

    /// 克体五行（忌神）
    pub ke_ti_wuxing: WuXing,

    /// 应期分析文本（简短）
    pub analysis: BoundedVec<u8, ConstU32<256>>,
}
```

#### 2.2.5 辅助卦象数据（AuxiliaryHexagrams）

```rust
/// 辅助卦象数据
///
/// 变卦、互卦、错卦、综卦、伏卦
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct AuxiliaryHexagrams {
    /// 变卦（上卦，下卦）
    pub bian_gua: (SingleGua, SingleGua),

    /// 互卦（上卦，下卦）
    pub hu_gua: (SingleGua, SingleGua),

    /// 错卦（上卦，下卦）
    pub cuo_gua: (SingleGua, SingleGua),

    /// 综卦（上卦，下卦）
    pub zong_gua: (SingleGua, SingleGua),

    /// 伏卦（上卦，下卦）
    pub fu_gua: (SingleGua, SingleGua),
}
```

#### 2.2.6 完整解卦数据（InterpretationData）

```rust
/// 完整解卦数据
///
/// 包含所有解卦所需的信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct InterpretationData {
    /// 基础信息
    pub basic_info: InterpretationBasicInfo,

    /// 卦象核心数据
    pub hexagram_core: HexagramCoreData,

    /// 体用分析
    pub tiyong_analysis: TiYongAnalysis,

    /// 应期推算
    pub yingqi_analysis: YingQiAnalysis,

    /// 辅助卦象
    pub auxiliary_hexagrams: AuxiliaryHexagrams,
}
```

### 2.3 详细解读数据（用于前端展示）

#### 2.3.1 单卦详细信息（SingleGuaDetail）

```rust
/// 单卦详细信息
///
/// 包含单个卦的所有文本信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct SingleGuaDetail {
    /// 卦名（如"乾"）
    pub name: BoundedVec<u8, ConstU32<16>>,

    /// 卦象符号（如"☰"）
    pub symbol: BoundedVec<u8, ConstU32<8>>,

    /// 五行（如"金"）
    pub wuxing: BoundedVec<u8, ConstU32<8>>,

    /// 卦数（1-8）
    pub number: u8,

    /// 二进制表示（3 bits）
    pub binary: u8,

    /// 卦象含义（如"天"、"泽"）
    pub meaning: BoundedVec<u8, ConstU32<32>>,
}
```

#### 2.3.2 六十四卦详细信息（HexagramFullDetail）

```rust
/// 六十四卦详细信息
///
/// 包含完整的卦辞、爻辞等文本信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct HexagramFullDetail {
    /// 六十四卦名称（如"乾为天"）
    pub name: BoundedVec<u8, ConstU32<32>>,

    /// 上卦详细信息
    pub shang_gua: SingleGuaDetail,

    /// 下卦详细信息
    pub xia_gua: SingleGuaDetail,

    /// 卦辞
    pub guaci: BoundedVec<u8, ConstU32<256>>,

    /// 动爻名称（如"初爻"）
    pub dong_yao_name: BoundedVec<u8, ConstU32<16>>,

    /// 动爻爻名（如"初九"、"六二"）
    pub dong_yao_ming: BoundedVec<u8, ConstU32<16>>,

    /// 动爻爻辞
    pub dong_yao_ci: BoundedVec<u8, ConstU32<256>>,

    /// 六爻爻名列表
    pub liuyao_names: BoundedVec<BoundedVec<u8, ConstU32<16>>, ConstU32<6>>,

    /// 六爻爻辞列表
    pub liuyao_yaoci: BoundedVec<BoundedVec<u8, ConstU32<256>>, ConstU32<6>>,
}
```

#### 2.3.3 完整解读详情（InterpretationFullDetail）

```rust
/// 完整解读详情
///
/// 用于前端展示的完整数据
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct InterpretationFullDetail {
    /// 基础信息
    pub basic_info: InterpretationBasicInfo,

    /// 本卦详细信息
    pub ben_gua: HexagramFullDetail,

    /// 变卦详细信息
    pub bian_gua: HexagramFullDetail,

    /// 互卦详细信息
    pub hu_gua: HexagramFullDetail,

    /// 错卦详细信息
    pub cuo_gua: HexagramFullDetail,

    /// 综卦详细信息
    pub zong_gua: HexagramFullDetail,

    /// 伏卦详细信息
    pub fu_gua: HexagramFullDetail,

    /// 体用分析
    pub tiyong_analysis: TiYongAnalysis,

    /// 体用关系详细解读
    pub tiyong_interpretation: BoundedVec<u8, ConstU32<512>>,

    /// 应期推算
    pub yingqi_analysis: YingQiAnalysis,

    /// 综合解读建议（简短）
    pub summary: BoundedVec<u8, ConstU32<512>>,
}
```

### 2.4 AI解读数据结构

#### 2.4.1 AI解读请求数据（AiInterpretationRequest）

```rust
/// AI解读请求数据
///
/// 发送给AI的结构化数据
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Debug)]
pub struct AiInterpretationRequest {
    /// 卦象ID
    pub hexagram_id: u64,

    /// 完整解卦数据
    pub interpretation_data: InterpretationData,

    /// 占卜问题（加密或哈希）
    pub question_hash: [u8; 32],

    /// 占卜类别
    pub category: u8,

    /// 请求时间戳
    pub request_timestamp: u64,
}
```

#### 2.4.2 AI解读结果（AiInterpretationResult）

```rust
/// AI解读结果
///
/// AI返回的解读内容
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct AiInterpretationResult {
    /// 卦象ID
    pub hexagram_id: u64,

    /// 解读内容的IPFS CID
    pub interpretation_cid: BoundedVec<u8, ConstU32<64>>,

    /// 解读摘要（链上存储）
    pub summary: BoundedVec<u8, ConstU32<512>>,

    /// 吉凶评分（0-100）
    pub fortune_score: u8,

    /// 可信度评分（0-100）
    pub confidence_score: u8,

    /// 提交时间戳
    pub submit_timestamp: u64,

    /// AI模型版本
    pub model_version: BoundedVec<u8, ConstU32<32>>,
}
```

---

## 三、存储优化策略

### 3.1 链上存储（On-Chain）

**仅存储核心数据**：
- `InterpretationData`（约 200-300 bytes）
- AI解读结果的CID和摘要（约 100 bytes）

**总计**：每个卦象约 300-400 bytes

### 3.2 链下存储（Off-Chain）

**IPFS存储**：
- 完整的AI解读文本（可能数KB）
- 详细的卦辞爻辞（已在constants中）
- 历史解读记录

### 3.3 Runtime API计算

**实时计算**：
- `InterpretationFullDetail`（通过 Runtime API 查询时计算）
- 所有文本信息（从constants查表）
- 辅助卦象（从核心数据推导）

---

## 四、API设计

### 4.1 链上可调用函数（Extrinsics）

```rust
/// 创建解卦数据（起卦时自动生成）
pub fn create_interpretation(
    origin: OriginFor<T>,
    hexagram_id: u64,
) -> DispatchResult;

/// 请求AI解读
pub fn request_ai_interpretation(
    origin: OriginFor<T>,
    hexagram_id: u64,
    category: u8,
) -> DispatchResult;

/// 提交AI解读结果（仅限Oracle）
pub fn submit_ai_interpretation(
    origin: OriginFor<T>,
    hexagram_id: u64,
    result: AiInterpretationResult,
) -> DispatchResult;
```

### 4.2 Runtime API查询

```rust
/// 获取完整解读详情
fn get_interpretation_full_detail(
    hexagram_id: u64
) -> Option<InterpretationFullDetail>;

/// 获取解卦数据（核心）
fn get_interpretation_data(
    hexagram_id: u64
) -> Option<InterpretationData>;

/// 获取AI解读结果
fn get_ai_interpretation(
    hexagram_id: u64
) -> Option<AiInterpretationResult>;

/// 批量查询用户的解卦记录
fn get_user_interpretations(
    account: AccountId,
    limit: u32,
    offset: u32,
) -> Vec<(u64, InterpretationData)>;
```

---

## 五、实现优先级

### P0（核心功能）
1. ✅ `InterpretationData` 数据结构
2. ✅ `TiYongAnalysis` 体用分析
3. ✅ `YingQiAnalysis` 应期推算
4. ✅ Runtime API 查询接口

### P1（完善功能）
1. `InterpretationFullDetail` 详细信息
2. `AiInterpretationRequest/Result` AI解读
3. 存储优化和缓存机制

### P2（扩展功能）
1. 历史记录查询
2. 统计分析功能
3. 批量解卦功能

---

## 六、使用示例

### 6.1 创建解卦数据

```rust
// 用户起卦后自动创建解卦数据
let interpretation_data = InterpretationData {
    basic_info: InterpretationBasicInfo {
        timestamp: current_timestamp,
        lunar_date: lunar_info,
        method: DivinationMethod::DateTime,
        gender: 1, // 男
        category: 1, // 事业
    },
    hexagram_core: HexagramCoreData {
        shang_gua: SingleGua::from_num(4), // 震
        xia_gua: SingleGua::from_num(5),   // 巽
        dong_yao: 1,
        ti_is_shang: true,
    },
    tiyong_analysis: calculate_tiyong_analysis(...),
    yingqi_analysis: calculate_yingqi_analysis(...),
    auxiliary_hexagrams: calculate_auxiliary_hexagrams(...),
};
```

### 6.2 查询完整解读

```rust
// 前端调用 Runtime API
let full_detail = runtime_api.get_interpretation_full_detail(hexagram_id)?;

// 返回包含所有文本信息的完整数据
println!("本卦：{}", String::from_utf8(full_detail.ben_gua.name.to_vec())?);
println!("卦辞：{}", String::from_utf8(full_detail.ben_gua.guaci.to_vec())?);
println!("体用关系：{}", String::from_utf8(full_detail.tiyong_interpretation.to_vec())?);
```

### 6.3 AI解读流程

```rust
// 1. 用户请求AI解读
pallet_meihua::request_ai_interpretation(origin, hexagram_id, category)?;

// 2. Oracle监听事件，获取解卦数据
let interpretation_data = runtime_api.get_interpretation_data(hexagram_id)?;

// 3. Oracle调用AI模型生成解读

// 4. Oracle提交结果
pallet_meihua::submit_ai_interpretation(
    oracle_origin,
    hexagram_id,
    AiInterpretationResult {
        interpretation_cid: ipfs_cid,
        summary: ai_summary,
        fortune_score: 75,
        confidence_score: 90,
        ...
    },
)?;
```

---

## 七、总结

本设计方案：

1. **完整性**：涵盖梅花易数解卦的所有核心要素
2. **高效性**：链上存储最小化，可推导数据通过API计算
3. **可扩展性**：预留扩展字段，支持未来功能升级
4. **AI友好**：结构化数据便于AI解读和分析
5. **隐私保护**：敏感信息仅存储哈希值
6. **前端友好**：提供完整的查询API，减少前端计算负担

相比 xuanxue/meihua 的 Java 实现，本方案：
- ✅ 保留了所有核心解卦功能
- ✅ 新增了五行旺衰、应期推算、伏卦等高级功能
- ✅ 集成了AI解读能力
- ✅ 优化了存储结构，适合区块链环境
- ✅ 提供了完整的查询API，便于前端使用
