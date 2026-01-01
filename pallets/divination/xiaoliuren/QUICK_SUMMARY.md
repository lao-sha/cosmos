# 小六壬解卦数据结构 - 快速总结

## 核心设计理念

基于对 `xuanxue/xiaoliuren` 代码和 `pallets/divination/xiaoliuren` 模块的深入分析，我们设计了一套**极致优化**的解卦数据结构。

## 一、核心数据结构（13 bytes）

```rust
pub struct XiaoLiuRenInterpretation {
    pub ji_xiong_level: JiXiongLevel,        // 1 byte - 吉凶等级
    pub overall_score: u8,                    // 1 byte - 综合评分(0-100)
    pub wu_xing_relation: WuXingRelation,    // 1 byte - 五行关系
    pub ti_yong_relation: Option<TiYongRelation>, // 2 bytes - 体用关系
    pub ba_gua: Option<BaGua>,               // 2 bytes - 八卦索引
    pub special_pattern: SpecialPattern,      // 1 byte - 特殊格局位标志
    pub advice_type: AdviceType,             // 1 byte - 建议类型
    pub school: XiaoLiuRenSchool,            // 1 byte - 流派
    pub ying_qi: Option<YingQiType>,         // 2 bytes - 应期类型
    pub reserved: u8,                         // 1 byte - 预留
}
```

**总大小：仅 13 bytes！**（比六爻的20字节更精简）

## 二、核心枚举

### 2.1 吉凶等级 (7种)

```rust
pub enum JiXiongLevel {
    DaJi,      // 大吉
    Ji,        // 吉
    XiaoJi,    // 小吉
    Ping,      // 平
    XiaoXiong, // 小凶
    Xiong,     // 凶
    DaXiong,   // 大凶
}
```

### 2.2 特殊格局（位标志，8种）

```rust
pub struct SpecialPattern(u8);
// 使用位运算，一个字节表示8种格局：
// - 纯宫（三宫相同）
// - 全吉/全凶
// - 五行相生/相克成环
// - 阴阳和合
// - 特殊时辰
```

### 2.3 建议类型 (8种)

```rust
pub enum AdviceType {
    JinQu,     // 大胆进取
    WenBu,     // 稳步前进
    ShouCheng, // 守成为主
    GuanWang,  // 谨慎观望
    TuiShou,   // 退守待时
    JingDai,   // 静待时机
    XunQiu,    // 寻求帮助
    HuaJie,    // 化解冲克
}
```

### 2.4 应期类型 (6种)

```rust
pub enum YingQiType {
    JiKe,      // 即刻应验（速喜）
    DangRi,    // 当日应验（大安、小吉）
    ShuRi,     // 数日应验（3-7天）
    YanChi,    // 延迟应验（留连，10天+）
    NanYi,     // 难以应验（空亡）
    XuHuaJie,  // 需要化解（赤口）
}
```

## 三、核心算法

### 3.1 吉凶等级计算公式

```rust
最终分数 = 基础分(时宫) + 格局调整 + 体用调整
吉凶等级 = 分数映射(1-7 → 大吉到大凶)
```

### 3.2 综合评分计算（0-100分）

| 维度 | 权重 | 说明 |
|------|------|------|
| 时宫吉凶 | 40% | 结果最重要 |
| 三宫整体 | 20% | 过程也重要 |
| 五行关系 | 20% | 生克影响 |
| 体用关系 | 10% | 天时配合 |
| 特殊格局 | 10% | 特殊加成 |

### 3.3 应期判断规则

直接根据时宫（结果）的六神：

- **速喜** → 即刻应验
- **大安、小吉** → 当日应验
- **留连** → 延迟应验（10天+）
- **空亡** → 难以应验
- **赤口** → 需要化解

## 四、设计优势

### 4.1 存储优化

- ✅ **仅13字节**：比传统方案节省90%+存储
- ✅ **位标志**：一个字节表示8种特殊格局
- ✅ **枚举索引**：所有字符串用u8索引

### 4.2 计算优化

- ✅ **懒加载**：首次查询时计算并缓存
- ✅ **Runtime API**：免费链下查询
- ✅ **可升级**：算法升级无需数据迁移

### 4.3 功能完整

- ✅ **流派支持**：道家/传统流派
- ✅ **体用分析**：高级道家分析
- ✅ **八卦具象**：三宫转八卦
- ✅ **应期推断**：实用性强

### 4.4 AI友好

- ✅ **结构化数据**：JSON格式提供给AI
- ✅ **分层描述**：总论→详解→建议
- ✅ **三宫卦辞**：传统文化底蕴

## 五、对比分析

### 5.1 vs 六爻解卦

| 项目 | 小六壬 | 六爻 |
|------|--------|------|
| 核心数据大小 | 13 bytes | 20 bytes |
| 复杂度 | ★★☆☆☆ | ★★★★★ |
| 用神判断 | 简单直接 | 复杂多变 |
| 应期推算 | 6种类型 | 8种类型 |
| 特殊格局 | 8种位标志 | 复杂枚举 |

### 5.2 vs mysterious Web应用

| 功能 | 区块链版本 | Web版本 |
|------|-----------|---------|
| 核心算法 | ✅ 开源 | ❌ 私有 |
| 六宫排盘 | ✅ 完整 | ✅ 完整 |
| 五行属性 | ✅ 双流派 | ✅ 单流派 |
| 体用分析 | ✅ 道家高级 | ❓ 未知 |
| AI解卦 | ✅ 多AI支持 | ✅ Gemini+Claude |
| 数据持久 | ✅ 永久链上 | ❌ 临时存储 |

## 六、使用示例

### 6.1 获取解卦数据

```rust
// Runtime API调用（免费）
let interpretation = runtime_api.get_interpretation(pan_id)?;

// 查看结果
println!("吉凶：{}", interpretation.ji_xiong_level.name());
println!("评分：{}/100", interpretation.overall_score);
println!("建议：{}", interpretation.advice_type.name());
```

### 6.2 生成详细文本

```rust
// 链下调用（免费）
let text = generate_interpretation_text(
    &interpretation,
    &san_gong,
    Some(shi_chen),
);

// 输出JSON格式
println!("{}", serde_json::to_string_pretty(&text)?);
```

### 6.3 AI解卦数据

```rust
// 为AI提供结构化数据
let ai_data = generate_ai_prompt_data(
    pan_id,
    &interpretation,
    &san_gong,
    Some(shi_chen),
    Some("今日运势如何？"),
);

// 提交到AI解卦系统
pallet_divination_ai::request_interpretation(
    DivinationType::XiaoLiuRen,
    pan_id,
    ai_data,
)?;
```

## 七、实施步骤

### 阶段1：数据结构（1天）
- [ ] 创建 `interpretation.rs` 模块
- [ ] 实现所有枚举类型
- [ ] 实现 `SpecialPattern` 位标志

### 阶段2：核心算法（2天）
- [ ] 实现 `interpret()` 函数
- [ ] 实现吉凶等级计算
- [ ] 实现综合评分算法
- [ ] 实现特殊格局识别

### 阶段3：Runtime API（1天）
- [ ] 定义 Runtime API
- [ ] 实现懒加载机制
- [ ] 实现文本生成

### 阶段4：测试（1天）
- [ ] 单元测试（覆盖率>90%）
- [ ] 集成测试
- [ ] 边界测试

### 总计：约5个工作日

## 八、参考文献

- **详细设计文档**：[INTERPRETATION_DESIGN.md](./INTERPRETATION_DESIGN.md)
- **类型定义**：[src/types.rs](./src/types.rs)
- **算法实现**：[src/algorithm.rs](./src/algorithm.rs)
- **六爻参考**：[../liuyao/INTERPRETATION_DESIGN.md](../liuyao/INTERPRETATION_DESIGN.md)

---

**设计者**：Claude Code
**日期**：2025-12-12
**版本**：v1.0
