# 自坐功能使用指南

## 📌 概述

自坐（日主自坐）是八字命理中最重要的关系之一，专指**日主（日柱天干）与日柱地支的十神关系**。

通过 V5 版本的更新，我们在 `FullBaziChartForApi` 中新增了 `zi_zuo` 字段，前端可以直接获取自坐信息，无需再从 `day_zhu` 中手动提取。

---

## 🆚 自坐 vs 星运的区别

| 维度 | 自坐（日柱十神） | 星运（十二长生） |
|------|----------------|----------------|
| **描述对象** | 十神关系（性格） | 能量状态（旺衰） |
| **范围** | 仅日柱地支（1个） | 四柱所有地支（4个） |
| **取值** | 10种十神类型 | 12种长生状态 |
| **作用** | 判断性格、能力、六亲 | 判断能量强弱、运势起伏 |
| **命理地位** | 核心分析之一 | 辅助分析 |

### 举例说明

**日柱：甲寅**
- **自坐分析**：
  - 地支：寅木
  - 本气十神：比肩（甲木见甲木）
  - 藏干十神：[比肩、食神、偏财]
  - 命理含义：性格独立自主，有主见，具备创造力和经营能力

- **星运分析**：
  - 日主甲木在日支寅：临官（很旺）
  - 命理含义：日主在日支得到强大的能量支持

---

## 📦 数据结构

### Rust 后端类型

```rust
/// 自坐信息（Runtime API 专用，无泛型）
pub struct ZiZuoInfo {
    /// 日柱地支（日主所坐的地支）
    pub dizhi: DiZhi,
    /// 本气十神（最重要，主导性格特质）
    pub benqi_shishen: ShiShen,
    /// 藏干十神列表（辅助性格、能力）
    pub canggan_shishen_list: Vec<ShiShen>,
}
```

### TypeScript 前端类型

```typescript
/**
 * 自坐信息
 */
export interface ZiZuoInfo {
  /** 日柱地支（日主所坐的地支） */
  diZhi: DiZhi;
  /** 本气十神（最重要，主导性格特质） */
  benQiShiShen: ShiShen;
  /** 藏干十神列表（辅助性格、能力） */
  cangGanShiShenList: ShiShen[];
}
```

---

## 🚀 使用方法

### 后端 Runtime API

自坐信息会自动包含在 `get_full_bazi_chart` 的返回结果中：

```rust
// Runtime API 调用
let full_chart = Pallet::<T>::get_full_bazi_chart(chart_id)?;

// 访问自坐信息
let zi_zuo = full_chart.zi_zuo;
println!("日主坐下地支: {:?}", zi_zuo.dizhi);
println!("本气十神: {:?}", zi_zuo.benqi_shishen);
println!("藏干十神列表: {:?}", zi_zuo.canggan_shishen_list);
```

### 前端使用

```typescript
// 获取八字命盘
const baziChart: FullBaziChartV5 = await api.getBaziChart(chartId);

// ⭐ 直接访问自坐信息（推荐）
const ziZuo = baziChart.ziZuo;

// 基本信息
console.log('日主:', TIAN_GAN_NAMES[baziChart.siZhu.rizhu]);
console.log('自坐地支:', DI_ZHI_NAMES[ziZuo.diZhi]);
console.log('本气十神:', SHI_SHEN_NAMES[ziZuo.benQiShiShen]);

// 藏干十神
console.log('藏干十神:', ziZuo.cangGanShiShenList.map(s => SHI_SHEN_NAMES[s]));

// 格式化显示
const displayZiZuo = (ziZuo: ZiZuoInfo) => {
  return `自坐${SHI_SHEN_NAMES[ziZuo.benQiShiShen]}`;
};

console.log(displayZiZuo(ziZuo)); // "自坐比肩"
```

### React 组件示例

```tsx
import { FullBaziChartV5, ZiZuoInfo, SHI_SHEN_NAMES, DI_ZHI_NAMES } from '@/types/bazi';

interface ZiZuoCardProps {
  ziZuo: ZiZuoInfo;
}

const ZiZuoCard: React.FC<ZiZuoCardProps> = ({ ziZuo }) => {
  return (
    <div className="zi-zuo-card">
      <h3>自坐分析</h3>
      <div className="zi-zuo-info">
        <p>
          <strong>日主自坐：</strong>
          {DI_ZHI_NAMES[ziZuo.diZhi]}
        </p>
        <p>
          <strong>本气十神：</strong>
          {SHI_SHEN_NAMES[ziZuo.benQiShiShen]}
        </p>
        <p>
          <strong>藏干十神：</strong>
          {ziZuo.cangGanShiShenList.map(s => SHI_SHEN_NAMES[s]).join('、')}
        </p>
      </div>
      <div className="zi-zuo-interpretation">
        {getZiZuoInterpretation(ziZuo.benQiShiShen)}
      </div>
    </div>
  );
};

// 自坐十神含义解释
const getZiZuoInterpretation = (shishen: ShiShen): string => {
  const interpretations = {
    [ShiShen.BiJian]: '性格独立自主，有主见，意志坚定，不易受他人影响。',
    [ShiShen.JieCai]: '有竞争意识，善于争取机会，果断行动。',
    [ShiShen.ShiShen]: '富有创造力，表达能力强，思维活跃，重视生活品质。',
    [ShiShen.ShangGuan]: '聪明机敏，才华横溢，表现欲强，追求自由。',
    [ShiShen.PianCai]: '善于经营理财，机会意识强，灵活变通。',
    [ShiShen.ZhengCai]: '务实踏实，重视财富积累，稳健经营，家庭责任感强。',
    [ShiShen.QiSha]: '意志坚定，执行力强，有魄力，不怕困难。',
    [ShiShen.ZhengGuan]: '正统保守，重视规矩秩序，责任心强，循规蹈矩。',
    [ShiShen.PianYin]: '思维独特，善于学习，多才多艺，但易孤独。',
    [ShiShen.ZhengYin]: '仁慈宽厚，重视学习和修养，稳重可靠，关爱他人。',
  };
  return interpretations[shishen] || '暂无解释';
};
```

---

## 💡 实际应用场景

### 1. 性格分析

```typescript
const analyzePersonality = (ziZuo: ZiZuoInfo): string => {
  const mainTrait = SHI_SHEN_NAMES[ziZuo.benQiShiShen];
  const subTraits = ziZuo.cangGanShiShenList
    .slice(1) // 跳过本气（已分析）
    .map(s => SHI_SHEN_NAMES[s])
    .join('、');

  return `命主自坐${mainTrait}，性格${getMainPersonalityTrait(ziZuo.benQiShiShen)}。` +
         `同时坐下藏有${subTraits}，具备${getSubPersonalityTraits(ziZuo.cangGanShiShenList)}。`;
};
```

### 2. 六亲关系分析

```typescript
const analyzeLiuQin = (ziZuo: ZiZuoInfo, gender: Gender): string => {
  // 男命看财（正财=妻、偏财=父）
  // 女命看官（正官=夫、七杀=情人）
  if (gender === Gender.Male) {
    if (ziZuo.benQiShiShen === ShiShen.ZhengCai) {
      return '自坐正财，夫妻关系和睦，妻子能力强，对家庭贡献大。';
    }
  } else {
    if (ziZuo.benQiShiShen === ShiShen.ZhengGuan) {
      return '自坐正官，夫妻关系稳定，丈夫有责任心，家庭美满。';
    }
  }
  // ...其他情况
};
```

### 3. 能力倾向分析

```typescript
const analyzeTalent = (ziZuo: ZiZuoInfo): string[] => {
  const talents: string[] = [];

  ziZuo.cangGanShiShenList.forEach(shishen => {
    switch (shishen) {
      case ShiShen.ShiShen:
        talents.push('艺术创作、演艺表演');
        break;
      case ShiShen.ShangGuan:
        talents.push('口才表达、技术创新');
        break;
      case ShiShen.PianCai:
      case ShiShen.ZhengCai:
        talents.push('商业经营、投资理财');
        break;
      case ShiShen.ZhengYin:
      case ShiShen.PianYin:
        talents.push('学术研究、教育培训');
        break;
      // ...其他情况
    }
  });

  return talents;
};
```

---

## 🔄 从旧版本迁移

如果你之前使用 `day_zhu` 手动提取自坐信息：

### 旧方式（不推荐）

```typescript
// ❌ 旧方式：从 day_zhu 中手动提取
const dayZhu = baziChart.siZhu.dayZhu;
const ziZuo = {
  diZhi: dayZhu.ganzhi.zhi,
  benQiShiShen: dayZhu.dizhiBenqiShishen,
  cangGanShiShenList: dayZhu.cangganList.map(c => c.shishen),
};
```

### 新方式（推荐）

```typescript
// ✅ 新方式：直接使用 ziZuo 字段
const ziZuo = baziChart.ziZuo;
```

---

## 📊 完整示例

```typescript
import { FullBaziChartV5 } from '@/types/bazi';

const displayBaziChart = (chart: FullBaziChartV5) => {
  console.log('=== 八字命盘分析 ===');
  console.log(`命盘ID: ${chart.chartId}`);
  console.log(`日主: ${TIAN_GAN_NAMES[chart.siZhu.rizhu]}`);

  // ⭐ 自坐分析
  console.log('\n--- 自坐分析 ---');
  console.log(`日主坐下: ${DI_ZHI_NAMES[chart.ziZuo.diZhi]}`);
  console.log(`本气十神: ${SHI_SHEN_NAMES[chart.ziZuo.benQiShiShen]}`);
  console.log(`藏干十神: ${chart.ziZuo.cangGanShiShenList.map(s => SHI_SHEN_NAMES[s]).join('、')}`);

  // 星运分析（对比）
  console.log('\n--- 星运分析 ---');
  console.log(`日支长生: ${CHANG_SHENG_NAMES[chart.xingYun.dayChangsheng]}`);

  // 综合分析
  console.log('\n--- 综合分析 ---');
  console.log(`性格特质: ${analyzePersonality(chart.ziZuo)}`);
  console.log(`能力倾向: ${analyzeTalent(chart.ziZuo).join('、')}`);
};
```

---

## ⚠️ 注意事项

1. **字段名称变化**：
   - Rust: `zi_zuo` (snake_case)
   - TypeScript: `ziZuo` (camelCase)

2. **与 day_zhu 的关系**：
   - `ziZuo` 是从 `day_zhu` 提取的核心信息
   - 两者数据来源相同，但 `ziZuo` 更方便直接使用
   - 如果需要更详细的日柱信息（如纳音、长生），仍需使用 `day_zhu`

3. **本气十神的重要性**：
   - `benQiShiShen` 是自坐分析的核心
   - 本气权重最高，对性格影响最大
   - 藏干十神为辅助，体现多面性格

4. **不要混淆自坐和星运**：
   - 自坐看十神关系（性格、能力）
   - 星运看长生状态（旺衰、能量）
   - 两者配合使用，分析更全面

---

## 📚 参考资料

- **类型定义**: `pallets/divination/bazi/src/types.rs`
- **前端类型**: `stardust-dapp/src/types/bazi.ts`
- **构建逻辑**: `pallets/divination/bazi/src/lib.rs:1340`
- **API 文档**: `pallets/divination/bazi/docs/API_DESIGN.md`

---

## 🎉 总结

通过新增的 `zi_zuo` 字段，前端可以：

✅ 直接获取自坐信息，无需手动提取
✅ 代码更简洁、可读性更强
✅ 类型安全，避免提取错误
✅ 命理含义清晰，便于分析展示

开始使用吧！如有问题，请参考本文档或查看源代码。
