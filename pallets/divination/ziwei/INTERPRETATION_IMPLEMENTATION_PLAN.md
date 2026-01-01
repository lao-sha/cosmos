# 紫微斗数解卦功能开发计划

## 开发阶段总览

```
阶段一：核心数据结构定义
    ↓
阶段二：评分算法实现
    ↓
阶段三：格局识别算法
    ↓
阶段四：四化飞星分析
    ↓
阶段五：Runtime API 实现
    ↓
阶段六：单元测试
    ↓
阶段七：前端组件开发（可选）
```

---

## 阶段一：核心数据结构定义

### 任务清单

- [ ] **1.1** 创建 `src/interpretation.rs` 文件
- [ ] **1.2** 定义吉凶等级枚举 `FortuneLevel`
- [ ] **1.3** 定义格局类型枚举 `PatternType`（32种）
- [ ] **1.4** 定义宫位解读结构 `PalaceInterpretation`
- [ ] **1.5** 定义格局信息结构 `PatternInfo`
- [ ] **1.6** 定义四化分析结构 `SiHuaAnalysis`
- [ ] **1.7** 定义大限解读结构 `DaXianInterpretation`
- [ ] **1.8** 定义整体评分结构 `ChartOverallScore`
- [ ] **1.9** 定义完整解卦结构 `ZiweiInterpretation`
- [ ] **1.10** 在 `lib.rs` 中导出 interpretation 模块

### 文件结构

```
pallets/divination/ziwei/src/
├── lib.rs                 # 添加 mod interpretation
├── types.rs               # 已有类型
├── algorithm.rs           # 已有算法
├── interpretation.rs      # 新增：解卦数据结构
├── interpretation/
│   ├── mod.rs            # 模块入口
│   ├── enums.rs          # 枚举定义
│   ├── structs.rs        # 结构体定义
│   ├── score.rs          # 评分算法
│   ├── pattern.rs        # 格局识别
│   ├── sihua.rs          # 四化分析
│   └── keywords.rs       # 关键词表
└── runtime_api.rs         # Runtime API（后续添加）
```

---

## 阶段二：评分算法实现

### 任务清单

- [ ] **2.1** 创建 `src/interpretation/score.rs`
- [ ] **2.2** 实现 `calculate_palace_score()` - 单宫评分
- [ ] **2.3** 实现 `calculate_star_strength()` - 主星强度
- [ ] **2.4** 实现 `calculate_si_hua_impact()` - 四化影响
- [ ] **2.5** 实现 `calculate_overall_score()` - 整体评分
- [ ] **2.6** 实现 `determine_fortune_level()` - 吉凶等级判断
- [ ] **2.7** 实现 `determine_ming_ge_level()` - 命格等级判断
- [ ] **2.8** 实现 `calculate_wu_xing_distribution()` - 五行分布

### 评分权重配置

```rust
// 宫位评分权重
const STAR_BRIGHTNESS_WEIGHT: i32 = 25;   // 主星亮度
const LIU_JI_WEIGHT: i32 = 5;             // 六吉星每颗
const LIU_SHA_WEIGHT: i32 = -5;           // 六煞星每颗
const SI_HUA_LU_WEIGHT: i32 = 15;         // 化禄
const SI_HUA_QUAN_WEIGHT: i32 = 10;       // 化权
const SI_HUA_KE_WEIGHT: i32 = 8;          // 化科
const SI_HUA_JI_WEIGHT: i32 = -20;        // 化忌
const LU_CUN_WEIGHT: i32 = 10;            // 禄存
const TIAN_MA_WEIGHT: i32 = 5;            // 天马

// 整体评分权重
const MING_GONG_WEIGHT: u32 = 40;         // 命宫 40%
const CAI_BO_WEIGHT: u32 = 15;            // 财帛 15%
const GUAN_LU_WEIGHT: u32 = 15;           // 官禄 15%
const FU_QI_WEIGHT: u32 = 15;             // 夫妻 15%
const OTHER_WEIGHT: u32 = 15;             // 其他 15%
```

---

## 阶段三：格局识别算法

### 任务清单

- [ ] **3.1** 创建 `src/interpretation/pattern.rs`
- [ ] **3.2** 实现辅助函数：
  - `gong_has_star()` - 检查宫位是否有某星
  - `gong_has_all_stars()` - 检查宫位是否有所有指定星
  - `gong_has_any_star()` - 检查宫位是否有任一指定星
  - `get_san_fang_indices()` - 获取三方四正宫位索引
  - `get_jia_gong_indices()` - 获取夹宫索引

### 格局检测函数（按优先级）

**富贵格局（高优先级）**
- [ ] **3.3** `check_zi_fu_tong_gong()` - 紫府同宫
- [ ] **3.4** `check_zi_fu_chao_yuan()` - 紫府朝垣
- [ ] **3.5** `check_jun_chen_qing_hui()` - 君臣庆会
- [ ] **3.6** `check_fu_xiang_chao_yuan()` - 府相朝垣
- [ ] **3.7** `check_ji_yue_tong_liang()` - 机月同梁
- [ ] **3.8** `check_ri_yue_bing_ming()` - 日月并明
- [ ] **3.9** `check_yue_lang_tian_men()` - 月朗天门
- [ ] **3.10** `check_ri_zhao_lei_men()` - 日照雷门

**权贵格局**
- [ ] **3.11** `check_san_qi_jia_hui()` - 三奇嘉会
- [ ] **3.12** `check_shuang_lu_jia_ming()` - 双禄夹命
- [ ] **3.13** `check_zuo_you_jia_ming()` - 左右夹命
- [ ] **3.14** `check_chang_qu_jia_ming()` - 昌曲夹命
- [ ] **3.15** `check_kui_yue_jia_ming()` - 魁钺夹命
- [ ] **3.16** `check_lu_ma_jiao_chi()` - 禄马交驰

**特殊格局**
- [ ] **3.17** `check_huo_tan_ge()` - 火贪格
- [ ] **3.18** `check_ling_tan_ge()` - 铃贪格
- [ ] **3.19** `check_tan_wu_tong_xing()` - 贪武同行

**凶格**
- [ ] **3.20** `check_yang_tuo_jia_ming()` - 羊陀夹命
- [ ] **3.21** `check_huo_ling_jia_ming()` - 火铃夹命
- [ ] **3.22** `check_kong_jie_jia_ming()` - 空劫夹命
- [ ] **3.23** `check_yang_tuo_jia_ji()` - 羊陀夹忌
- [ ] **3.24** `check_si_sha_chong_ming()` - 四煞冲命
- [ ] **3.25** `check_ma_tou_dai_jian()` - 马头带箭
- [ ] **3.26** `check_ling_chang_tuo_wu()` - 铃昌陀武
- [ ] **3.27** `check_ming_wu_zheng_yao()` - 命无正曜

**汇总函数**
- [ ] **3.28** `identify_all_patterns()` - 识别所有格局
- [ ] **3.29** `calculate_pattern_bonus()` - 计算格局加成分

---

## 阶段四：四化飞星分析

### 任务清单

- [ ] **4.1** 创建 `src/interpretation/sihua.rs`
- [ ] **4.2** 实现 `get_gong_gan_si_hua()` - 获取宫干四化
- [ ] **4.3** 实现 `calculate_fei_hua()` - 计算飞化落宫
- [ ] **4.4** 实现 `check_zi_hua()` - 检查自化
- [ ] **4.5** 实现 `check_hua_ji_chong_po()` - 检查化忌冲破
- [ ] **4.6** 实现 `analyze_si_hua()` - 综合四化分析
- [ ] **4.7** 实现 `get_si_hua_impact_text()` - 四化影响文本

### 四化飞星表

```rust
/// 天干四化表（禄权科忌）
const SI_HUA_TABLE: [[SiHuaStar; 4]; 10] = [
    // 甲：廉贞化禄、破军化权、武曲化科、太阳化忌
    [SiHuaStar::LianZhen, SiHuaStar::PoJun, SiHuaStar::WuQu, SiHuaStar::TaiYang],
    // 乙：天机化禄、天梁化权、紫微化科、太阴化忌
    [SiHuaStar::TianJi, SiHuaStar::TianLiang, SiHuaStar::ZiWei, SiHuaStar::TaiYin],
    // ... 其他天干
];
```

---

## 阶段五：Runtime API 实现

### 任务清单

- [ ] **5.1** 创建 `src/runtime_api.rs`
- [ ] **5.2** 定义 `ZiweiInterpretationApi` trait
- [ ] **5.3** 实现 `generate_interpretation()` - 生成解卦数据
- [ ] **5.4** 实现 `get_palace_detail()` - 获取宫位详情
- [ ] **5.5** 实现 `get_pattern_detail()` - 获取格局详情
- [ ] **5.6** 实现 `get_da_xian_detail()` - 获取大限详情
- [ ] **5.7** 实现 `get_liu_nian_fortune()` - 获取流年运势
- [ ] **5.8** 在 runtime 中注册 API

### API 接口定义

```rust
sp_api::decl_runtime_apis! {
    pub trait ZiweiInterpretationApi<AccountId, BlockNumber, Moment>
    where
        AccountId: codec::Codec,
        BlockNumber: codec::Codec,
        Moment: codec::Codec,
    {
        /// 生成命盘解读
        fn generate_interpretation(chart_id: u64) -> Option<ZiweiInterpretation>;

        /// 获取宫位详细解读
        fn get_palace_detail(chart_id: u64, gong_wei: GongWei) -> Option<PalaceDetailText>;

        /// 获取格局详细说明
        fn get_pattern_detail(pattern_type: PatternType) -> PatternDetailText;

        /// 获取指定年龄的大限详情
        fn get_da_xian_detail(chart_id: u64, age: u8) -> Option<DaXianDetailText>;

        /// 获取流年运势
        fn get_liu_nian_fortune(chart_id: u64, year: u16) -> Option<LiuNianFortune>;
    }
}
```

---

## 阶段六：单元测试

### 任务清单

- [ ] **6.1** 创建测试用例文件结构
- [ ] **6.2** 评分算法测试：
  - `test_palace_score_calculation()`
  - `test_overall_score_calculation()`
  - `test_fortune_level_determination()`
- [ ] **6.3** 格局识别测试：
  - `test_zi_fu_tong_gong_pattern()`
  - `test_san_qi_jia_hui_pattern()`
  - `test_yang_tuo_jia_ming_pattern()`
  - `test_multiple_patterns()`
- [ ] **6.4** 四化分析测试：
  - `test_gong_gan_si_hua()`
  - `test_fei_hua_calculation()`
  - `test_zi_hua_detection()`
- [ ] **6.5** 集成测试：
  - `test_full_interpretation_generation()`
  - `test_interpretation_with_real_chart()`

### 测试命令

```bash
# 运行所有测试
cargo test -p pallet-ziwei --lib -- --nocapture

# 运行特定测试
cargo test -p pallet-ziwei test_palace_score --lib -- --nocapture

# 运行解卦相关测试
cargo test -p pallet-ziwei interpretation --lib -- --nocapture
```

---

## 阶段七：前端组件开发（可选）

### 任务清单

- [ ] **7.1** 创建 `src/features/ziwei/interpretation/` 目录
- [ ] **7.2** 实现 `InterpretationPanel.tsx` - 主解读面板
- [ ] **7.3** 实现 `OverallScoreCard.tsx` - 整体评分卡片
- [ ] **7.4** 实现 `PalaceInterpretationList.tsx` - 十二宫解读
- [ ] **7.5** 实现 `PatternBadges.tsx` - 格局标签
- [ ] **7.6** 实现 `SiHuaFlowChart.tsx` - 四化流程图
- [ ] **7.7** 实现 `DaXianTimeline.tsx` - 大限时间线
- [ ] **7.8** 实现 `useInterpretation.ts` - 数据Hook
- [ ] **7.9** 实现 `interpretationText.ts` - 文案库
- [ ] **7.10** 集成到 `ZiweiDetailPage.tsx`

---

## 开发顺序建议

### 第一批（核心功能）

```
1. interpretation/enums.rs      - 枚举定义
2. interpretation/structs.rs    - 结构体定义
3. interpretation/score.rs      - 评分算法
4. interpretation/mod.rs        - 模块导出
```

### 第二批（格局识别）

```
5. interpretation/pattern.rs    - 格局识别
6. interpretation/keywords.rs   - 关键词表
```

### 第三批（四化分析）

```
7. interpretation/sihua.rs      - 四化分析
```

### 第四批（API与测试）

```
8. runtime_api.rs               - Runtime API
9. tests.rs                     - 单元测试
```

### 第五批（前端，可选）

```
10. 前端组件开发
```

---

## 依赖关系图

```
                    ┌─────────────────┐
                    │   types.rs      │
                    │  (已有类型)      │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
    │  enums.rs   │  │ keywords.rs │  │ algorithm.rs│
    │ (枚举定义)   │  │ (关键词表)   │  │  (已有算法)  │
    └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
           │                │                │
           └────────────────┼────────────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │   structs.rs    │
                   │  (结构体定义)    │
                   └────────┬────────┘
                            │
           ┌────────────────┼────────────────┐
           │                │                │
           ▼                ▼                ▼
    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
    │  score.rs   │  │ pattern.rs  │  │  sihua.rs   │
    │ (评分算法)   │  │ (格局识别)   │  │ (四化分析)   │
    └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
           │                │                │
           └────────────────┼────────────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │    mod.rs       │
                   │ (模块汇总导出)   │
                   └────────┬────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │  runtime_api.rs │
                   │  (Runtime API)  │
                   └─────────────────┘
```

---

## 验收标准

### 阶段一验收
- [ ] 所有数据结构可编译通过
- [ ] 实现 `Default` trait
- [ ] 实现 `MaxEncodedLen` trait
- [ ] 类型大小符合预期（~410 bytes）

### 阶段二验收
- [ ] 评分算法返回 0-100 范围
- [ ] 吉凶等级判断正确
- [ ] 边界条件处理正确

### 阶段三验收
- [ ] 至少识别 20 种格局
- [ ] 格局分数计算正确
- [ ] 无误报/漏报

### 阶段四验收
- [ ] 四化飞星计算正确
- [ ] 自化检测正确
- [ ] 化忌冲破检测正确

### 阶段五验收
- [ ] Runtime API 可正常调用
- [ ] 返回数据结构完整
- [ ] 性能满足要求

### 阶段六验收
- [ ] 测试覆盖率 > 80%
- [ ] 所有测试通过
- [ ] 无 clippy 警告

---

## 风险与注意事项

1. **存储大小**：确保 `ZiweiInterpretation` 不超过链上存储限制
2. **计算复杂度**：格局识别算法需优化，避免 O(n²) 复杂度
3. **兼容性**：新增字段需考虑向后兼容
4. **测试数据**：准备多种命盘测试用例（吉格、凶格、普通）
5. **文案准确性**：关键词和解读文案需经专业人士审核

---

## 参考资料

- `xuanxue/ziwei/ZhouYiLab/` - C++ 参考实现
- `pallets/divination/meihua/src/interpretation.rs` - 梅花解卦参考
- `pallets/divination/xiaoliuren/src/interpretation/` - 小六壬解卦参考
- `INTERPRETATION_DATA_STRUCTURE_DESIGN.md` - 数据结构设计文档
- `INTERPRETATION_PLAN.md` - 原始规划文档
