# 前端数据显示问题调试指南

## 问题描述

基本盘中的"星运"、"空亡"、"神煞"、"自坐"字段显示为 `-` 或空白，说明这些字段没有从链端正确获取数据。

##  已完成的修复

✅ **后端实现**：所有字段都已在 Runtime API 中正确构建和返回
✅ **前端类型定义**：所有类型都已正确定义
✅ **前端解析函数**：所有解析函数都已添加
✅ **前端服务层**：`baziChainService.ts` 中的 `parseFullBaziChartV5` 已正确解析所有字段

## 调试步骤

### 1. 检查 Runtime API 是否正确返回数据

在浏览器控制台中查看日志：

```
[BaziChainService] V5 完整命盘原始数据: {...}
[BaziChainService] V5 JSON 数据: {...}
```

**预期输出示例**：
```json
{
  "chartId": 1,
  "owner": "0x...",
  "birthTime": { "year": 1998, "month": 7, "day": 31, "hour": 14, "minute": 10 },
  "gender": "Male",
  "zishiMode": "Modern",
  "sizhu": { ... },
  "dayun": { ... },
  "kongwang": {
    "yearKongwang": [4, 5],    // ⭐ 检查这个
    "monthKongwang": [4, 5],
    "dayKongwang": [4, 5],
    "hourKongwang": [4, 5],
    "yearIsKong": false,
    "monthIsKong": false,
    "dayIsKong": false,
    "hourIsKong": false
  },
  "shenshaList": [              // ⭐ 检查这个
    { "shensha": "TianYi", "position": "Year", "nature": "JiShen" },
    ...
  ],
  "xingyun": {                  // ⭐ 检查这个
    "yearChangsheng": "ChangSheng",
    "monthChangsheng": "LinGuan",
    "dayChangsheng": "DiWang",
    "hourChangsheng": "Shuai"
  },
  "ziZuo": {                    // ⭐ 检查这个（新增）
    "dizhi": 2,
    "benqiShishen": "BiJian",
    "cangganShishenList": ["BiJian", "ShiShen", "PianCai"]
  },
  ...
}
```

### 2. 检查字段名是否匹配

Polkadot JS API 会自动将 Rust 的 `snake_case` 转换为 `camelCase`：

| Rust 字段名 | JSON 字段名 | TypeScript 解析 |
|------------|-------------|----------------|
| `zi_zuo` | `ziZuo` | `jsonData.ziZuo` ✅ |
| `kongwang` | `kongwang` | `jsonData.kongwang` ✅ |
| `xingyun` | `xingyun` | `jsonData.xingyun` ✅ |
| `shensha_list` | `shenshaList` | `jsonData.shenshaList` ✅ |

### 3. 在浏览器控制台手动测试

打开浏览器控制台，运行以下代码：

```javascript
// 导入服务
import { getFullBaziChartV5 } from './services/baziChainService';

// 获取命盘数据
const chartId = 1;  // 替换为你的命盘 ID
const fullChart = await getFullBaziChartV5(chartId);

// 检查各字段
console.log('ziZuo:', fullChart?.ziZuo);
console.log('kongWang:', fullChart?.kongWang);
console.log('xingYun:', fullChart?.xingYun);
console.log('shenShaList:', fullChart?.shenShaList);
```

### 4. 检查 Runtime API 是否已注册

在浏览器控制台检查：

```javascript
const api = await window.polkadot?.api;
console.log('Runtime API available:', !!api?.call?.baziChartApi?.getFullBaziChart);
```

如果返回 `false`，说明 Runtime API 未正确注册。

### 5. 检查链端数据

如果 Runtime API 返回的数据中这些字段为空或null，需要检查链端：

```bash
cd /home/xiaodong/文档/stardust
cargo test -p pallet-bazi-chart test_get_full_bazi_chart -- --nocapture
```

预期输出应包含这些字段的非空值。

### 6. 常见问题排查

#### 问题A：Runtime API 未正确注册

**症状**：控制台显示 `getFullBaziChart Runtime API 不可用`

**解决方法**：
1. 确认 Runtime 中已添加 `baziChartApi`
2. 重新编译并重启节点
3. 确认前端连接到了正确的节点

#### 问题B：字段返回 null 或 undefined

**症状**：JSON 数据中字段存在但值为 `null`

**解决方法**：
1. 检查链上是否有数据：查看 `ChartById` storage
2. 检查 `get_full_bazi_chart` 函数的实现
3. 确认 `build_enhanced_sizhu` 正确构建了所有字段

#### 问题C：字段名大小写不匹配

**症状**：控制台报错或字段始终为默认值

**解决方法**：
1. 在 `parseFullBaziChartV5` 中添加 console.log 打印原始数据
2. 对比字段名是否匹配
3. 必要时调整解析代码

## 修复验证清单

使用以下清单验证修复是否成功：

- [ ] 控制台中能看到 `V5 JSON 数据` 日志
- [ ] JSON 数据中包含 `ziZuo`、`kongwang`、`xingyun`、`shenshaList` 字段
- [ ] 这些字段的值不为 `null`
- [ ] 前端解析后的数据结构正确（使用 console.log 验证）
- [ ] 页面上显示的数据不再是 `-`

## 完整测试示例

在 `BaziDetailPage.tsx` 的 `loadBaziData` 函数中添加调试日志：

```typescript
const loadBaziData = async () => {
  try {
    // ... 现有代码 ...

    const fullChartV5 = await getFullBaziChartV5(baziId);
    if (fullChartV5) {
      setFullChartDataV5(fullChartV5);

      // ⭐ 添加调试日志
      console.log('[DEBUG] V5 完整数据:', fullChartV5);
      console.log('[DEBUG] 自坐:', fullChartV5.ziZuo);
      console.log('[DEBUG] 空亡:', fullChartV5.kongWang);
      console.log('[DEBUG] 星运:', fullChartV5.xingYun);
      console.log('[DEBUG] 神煞:', fullChartV5.shenShaList);
    } else {
      console.error('[DEBUG] V5 数据获取失败');
    }
  } catch (error) {
    console.error('[DEBUG] 加载失败:', error);
  }
};
```

刷新页面，检查控制台输出。

## 预期结果

修复成功后，基本盘应该显示：

```
┌─────┬──────┬──────┬──────┬──────┐
│     │ 年柱 │ 月柱 │ 日柱 │ 时柱 │
├─────┼──────┼──────┼──────┼──────┤
│ 星运│ 长生 │ 临官 │ 帝旺 │  衰  │  ⭐ 应显示具体状态
├─────┼──────┼──────┼──────┼──────┤
│ 空亡│子丑◎│子丑  │戌亥  │戌亥  │  ⭐ 应显示地支+标记
├─────┼──────┼──────┼──────┼──────┤
│ 神煞│天乙  │文昌  │桃花  │ 劫煞 │  ⭐ 应显示神煞名称
└─────┴──────┴──────┴──────┴──────┘
```

## 需要帮助？

如果按照以上步骤仍然无法解决问题，请提供以下信息：

1. 浏览器控制台的完整日志（尤其是包含 `[BaziChainService]` 的日志）
2. 命盘 ID
3. 前端代码版本和节点版本
4. 是否重新编译并重启了节点

---

**最后更新**: 2025-12-21
**相关文档**:
- [ZI_ZUO_GUIDE.md](./ZI_ZUO_GUIDE.md) - 自坐功能使用指南
- [API_DESIGN.md](./API_DESIGN.md) - API 设计文档
