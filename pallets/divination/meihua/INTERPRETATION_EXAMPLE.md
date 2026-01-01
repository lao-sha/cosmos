# 梅花易数解卦详细例子

## 案例背景

**占卜时间**：2024年农历冬月二十日子时（2024-12-20 23:30）
**占卜问题**：近期工作是否会有新的机会？
**起卦方式**：时间起卦
**占卜者**：男性，30岁，从事IT行业
**占卜类别**：事业运

---

## 第一步：起卦计算

### 1.1 农历时间

```
公历：2024年12月20日 23:30
农历：甲辰年 冬月（十一月）二十日 子时
```

### 1.2 起卦数据

```rust
年支数：辰年 = 5
月数：冬月（十一月）= 11
日数：二十日 = 20
时支数：子时 = 1

上卦数 = (5 + 11 + 20) % 8 = 36 % 8 = 4 → 震卦 ☳
下卦数 = (5 + 11 + 20 + 1) % 8 = 37 % 8 = 5 → 巽卦 ☴
动爻数 = (5 + 11 + 20 + 1) % 6 = 37 % 6 = 1 → 初爻动
```

### 1.3 数据结构

```rust
let lunar_date = LunarDateInfo {
    year: 2024,
    month: 11,
    day: 20,
    hour_zhi_num: 1,
    is_leap_month: false,
};

let basic_info = InterpretationBasicInfo {
    timestamp: 1703088600, // 2024-12-20 23:30:00 UTC
    lunar_date,
    method: DivinationMethod::DateTime,
    gender: 1, // 男
    category: 1, // 事业
};
```

---

## 第二步：本卦分析

### 2.1 本卦：雷风恒（震上巽下）

```
卦名：雷风恒
卦象：☳☴
上卦：震 ☳ (雷) - 五行属木
下卦：巽 ☴ (风) - 五行属木
动爻：初爻（下卦初爻）
```

### 2.2 卦象结构

```
上爻 ━━━  震卦
五爻 ━ ━  (雷)
四爻 ━━━

三爻 ━━━  巽卦
二爻 ━━━  (风)
初爻 ━ ━  ← 动爻
```

### 2.3 卦辞

**恒卦卦辞**：恒，亨，无咎，利贞，利有攸往。

**白话解释**：恒久之道，亨通顺利，没有过失，利于坚守正道，利于有所前进。

### 2.4 动爻爻辞

**初六爻辞**：浚恒，贞凶，无攸利。

**白话解释**：刚开始就急于求成，即使坚守也会有凶险，没有什么好处。

### 2.5 数据结构

```rust
let hexagram_core = HexagramCoreData {
    shang_gua: SingleGua::from_num(4), // 震
    xia_gua: SingleGua::from_num(5),   // 巽
    dong_yao: 1,
    ti_is_shang: true, // 动爻在下卦，上卦为体
};
```

---

## 第三步：体用判断

### 3.1 体用规则

**梅花易数规则**：动爻在哪卦，哪卦为用，另一卦为体

- 动爻在初爻（下卦）
- 下卦为用卦：巽（木）
- 上卦为体卦：震（木）

### 3.2 体用关系

```
体卦：震（木）
用卦：巽（木）
关系：比和（体用五行相同）
```

**比和关系**：
- 五行相同，气势相当
- 主次吉，事情平稳发展
- 无生克关系，需看旺衰

### 3.3 数据结构

```rust
let tiyong_analysis = TiYongAnalysis {
    ti_wuxing: WuXing::Mu,  // 体卦木
    yong_wuxing: WuXing::Mu, // 用卦木
    ben_gua_relation: TiYongRelation::BiHe, // 比和
    bian_gua_relation: TiYongRelation::YongKeTi, // 变卦用克体
    hu_gua_relation: TiYongRelation::YongKeTi, // 互卦用克体
    ti_wangshuai: WangShuai::Xiang, // 体卦相（次旺）
    fortune: Fortune::XiaoXiong, // 小凶
    fortune_level: 1, // 1/4
};
```

---

## 第四步：变卦分析

### 4.1 变卦计算

初爻动，阴爻变阳爻：

```
本卦：雷风恒 ☳☴
      震 巽
      111 110

初爻变：110 → 111

变卦：雷天大壮 ☳☰
      震 乾
      111 111
```

### 4.2 变卦：雷天大壮（震上乾下）

```
卦名：雷天大壮
卦象：☳☰
上卦：震 ☳ (雷) - 五行属木
下卦：乾 ☰ (天) - 五行属金
```

### 4.3 变卦体用关系

```
体卦：震（木）- 位置不变
用卦：乾（金）- 初爻变化
关系：用克体（金克木）- 大凶
```

**用克体关系**：
- 用卦克制体卦
- 主大凶，事情不利
- 外部环境对自己不利
- 需谨慎行事

### 4.4 变卦卦辞

**大壮卦卦辞**：大壮，利贞。

**白话解释**：阳气强盛，利于坚守正道。但过于强盛也需警惕。

### 4.5 数据结构

```rust
let auxiliary_hexagrams = AuxiliaryHexagrams {
    bian_gua: (
        SingleGua::from_num(4), // 震
        SingleGua::from_num(1), // 乾
    ),
    // ... 其他卦象
};
```

---

## 第五步：互卦分析

### 5.1 互卦计算

**互卦规则**：
- 互卦上卦：取本卦第3、4、5爻
- 互卦下卦：取本卦第2、3、4爻

```
本卦六爻：111 110
         震  巽

互卦上卦：取345爻 = 111 → 震
互卦下卦：取234爻 = 111 → 震

互卦：雷为震 ☳☳
```

### 5.2 互卦：雷为震（震上震下）

```
卦名：雷为震
卦象：☳☳
上卦：震 ☳ (雷) - 五行属木
下卦：震 ☳ (雷) - 五行属木
```

### 5.3 互卦体用关系

```
体卦：震（木）
用卦：震（木）
关系：比和
```

**互卦含义**：代表事物发展的过程

---

## 第六步：错卦、综卦、伏卦

### 6.1 错卦：风雷益（巽上震下）

**错卦规则**：所有爻阴阳互变

```
本卦：111 110 → 错卦：000 001
      震  巽          坤  艮

实际错卦：风雷益 ☴☳
```

### 6.2 综卦：风雷益（巽上震下）

**综卦规则**：上下颠倒（180°旋转）

```
本卦：雷风恒 ☳☴
综卦：风雷益 ☴☳
```

### 6.3 伏卦：风雷益（巽上震下）

**伏卦规则**：八卦各有其对应的伏卦关系

```
震的伏卦：巽
巽的伏卦：震

伏卦：风雷益 ☴☳
```

### 6.4 数据结构

```rust
let auxiliary_hexagrams = AuxiliaryHexagrams {
    bian_gua: (震, 乾),
    hu_gua: (震, 震),
    cuo_gua: (巽, 震),
    zong_gua: (巽, 震),
    fu_gua: (巽, 震),
};
```

---

## 第七步：五行旺衰分析

### 7.1 当前季节

```
农历：冬月（十一月）
季节：冬季
当令五行：水旺
```

### 7.2 体卦旺衰

```
体卦：震（木）
季节：冬季（水旺）
关系：水生木
旺衰：相（次旺）
```

**旺衰五等级**：
- 旺：当令，最强（如冬天的水）
- 相：被当令所生，次强（如冬天的木，水生木）✅ 当前状态
- 休：生当令五行，休息（如冬天的金，金生水）
- 囚：克当令五行，受制（如冬天的土，土克水）
- 死：被当令所克，最弱（如冬天的火，水克火）

### 7.3 旺衰影响

**体卦相（次旺）**：
- 体卦有力，但不是最强
- 得季节之助，有一定优势
- 适合稳步发展，不宜冒进

### 7.4 数据结构

```rust
let ti_wangshuai = WangShuai::Xiang; // 相（次旺）
```

---

## 第八步：应期推算

### 8.1 应期规则

**梅花易数应期推算**：
1. 体卦旺相时：应期在生体之五行的卦数，或体用卦数之和
2. 体卦休囚时：应期在体所生之五行的卦数，或体卦卦数
3. 用卦克体时：应期在克用之五行的卦数
4. 用卦生体时：应期较快，在用卦卦数

### 8.2 应期计算

```
体卦：震（木），卦数 = 4
用卦：巽（木），卦数 = 5
体卦旺衰：相（次旺）
体用关系：比和

主要应期数 = 体用卦数之和 = 4 + 5 = 9
次要应期数 = 生体五行（水）的卦数 = 6（坎卦）
```

### 8.3 应期解释

**主要应期**：9
- 可应9日、9月、9年
- 或农历九月
- 或9个时间单位

**次要应期**：6
- 可应6日、6月、6年
- 或农历六月
- 或6个时间单位

**喜神**：水（生体）
- 水旺时节有利（冬季）
- 遇水日、水月有利

**忌神**：金（克体）
- 金旺时节不利（秋季）
- 遇金日、金月需谨慎

### 8.4 数据结构

```rust
let yingqi_analysis = YingQiAnalysis {
    ti_gua_num: 4,
    yong_gua_num: 5,
    primary_num: 9,
    secondary_nums: [6, 6],
    sheng_ti_wuxing: WuXing::Shui, // 水生木
    ke_ti_wuxing: WuXing::Jin,     // 金克木
    analysis: BoundedVec::try_from(
        "体卦木相，比和。喜神为水。应期数：9（可应年、月、日、时）。体卦数4，用卦数5。"
            .as_bytes()
            .to_vec()
    ).unwrap(),
};
```

---

## 第九步：综合解卦

### 9.1 吉凶判断

**本卦**：比和 → 次吉（3/4分）
**变卦**：用克体 → 大凶（0/4分）
**综合评分**：(3×0.6 + 0×0.4) = 1.8 → **小凶偏平**

### 9.2 详细解读

#### 1. 整体运势

**雷风恒卦**，恒者久也。本卦体用比和，震巽皆木，表示当前工作状态稳定，但缺乏突破性变化。动爻在初爻，说明变化刚刚开始，尚未显现。

**关键点**：
- 本卦比和：稳定但缺乏动力
- 动爻初爻：变化刚开始
- 体卦相旺：有一定优势

#### 2. 本卦分析

**体用比和**：
- ✅ 工作环境平稳
- ✅ 同事关系和谐
- ✅ 能力与岗位匹配
- ⚠️ 容易陷入舒适区
- ⚠️ 缺乏突破性进展

**震巽同气**：
- 震为雷，主动、进取
- 巽为风，主顺从、渗透
- 两者皆木，同气相求
- 但缺乏外部助力

**初爻动**：
- 基础层面有变动迹象
- 可能是小的调整
- 尚未形成大的变化

#### 3. 变卦警示 ⚠️

**变卦为大壮，用克体（金克木）**，这是关键警示：

**新机会可能带来压力**：
- 表面看似机会，实则挑战大于收益
- 金克木：可能遇到强势的领导或竞争对手
- 需要付出更多努力才能应对

**不宜贸然行动**：
- 不宜跳槽或接受新项目
- 需要充分评估风险
- 等待更合适的时机

#### 4. 互卦提示

**互卦为震为雷**，比和关系：
- 事物发展过程中保持稳定
- 需要持续努力
- 不会有太大波折

#### 5. 时机建议

**当前时机**：
- 冬季体卦得相（次旺）
- 时机尚可，但不是最佳
- 适合积累，不宜冒进

**最佳时机**：
- 春季（木旺）：体卦最强
- 农历九月或六月：应期到来
- 水旺时节：得喜神之助

**不利时机**：
- 秋季（金旺）：忌神当令
- 金日、金月：需谨慎

#### 6. 行动建议

**✅ 宜**：
1. **保持现状，稳扎稳打**
   - 当前工作稳定，不宜轻易改变
   - 继续深耕现有领域

2. **提升专业技能，积累人脉**
   - 利用稳定期提升自己
   - 为未来机会做准备

3. **观察市场动向，不急于行动**
   - 了解行业趋势
   - 等待更合适的时机

4. **等待春季或应期**
   - 春季木旺，体卦最强
   - 农历九月或六月行动更佳

**❌ 忌**：
1. **盲目跳槽或接受新项目**
   - 变卦用克体，外部环境不利
   - 新机会可能是陷阱

2. **与领导或同事发生冲突**
   - 金克木，需避免正面冲突
   - 保持和谐关系

3. **急于求成，操之过急**
   - 初爻爻辞：浚恒，贞凶
   - 急于求成反而不利

4. **秋季或金旺时节行动**
   - 忌神当令，不利行动
   - 需等待更好时机

#### 7. 最终结论

**近期工作会有新机会出现**，但这些机会**并非真正的良机**。

**核心判断**：
- 表面看似光鲜，实则暗藏风险（用克体）
- 需要付出更多努力才能应对
- 不如保持现状，等待更好时机

**建议**：
1. 保持现状，稳扎稳打
2. 提升自己，积累实力
3. 等待春季（木旺）或应期（九月、六月）
4. 如果必须做出选择，应在农历九月或六月再行动

**应期**：
- 主要应期：9日/月（农历九月）
- 次要应期：6日/月（农历六月）
- 最佳时机：春季（木旺）

---

## 第十步：数据结构总结

### 10.1 完整解卦数据

```rust
let interpretation_data = InterpretationData {
    basic_info: InterpretationBasicInfo {
        timestamp: 1703088600,
        lunar_date: LunarDateInfo {
            year: 2024,
            month: 11,
            day: 20,
            hour_zhi_num: 1,
            is_leap_month: false,
        },
        method: DivinationMethod::DateTime,
        gender: 1,
        category: 1,
    },
    hexagram_core: HexagramCoreData {
        shang_gua: SingleGua::from_num(4), // 震
        xia_gua: SingleGua::from_num(5),   // 巽
        dong_yao: 1,
        ti_is_shang: true,
    },
    tiyong_analysis: TiYongAnalysis {
        ti_wuxing: WuXing::Mu,
        yong_wuxing: WuXing::Mu,
        ben_gua_relation: TiYongRelation::BiHe,
        bian_gua_relation: TiYongRelation::YongKeTi,
        hu_gua_relation: TiYongRelation::BiHe,
        ti_wangshuai: WangShuai::Xiang,
        fortune: Fortune::XiaoXiong,
        fortune_level: 1,
    },
    yingqi_analysis: YingQiAnalysis {
        ti_gua_num: 4,
        yong_gua_num: 5,
        primary_num: 9,
        secondary_nums: [6, 6],
        sheng_ti_wuxing: WuXing::Shui,
        ke_ti_wuxing: WuXing::Jin,
        analysis: BoundedVec::try_from(
            "体卦木相，比和。喜神为水。应期数：9（可应年、月、日、时）。体卦数4，用卦数5。"
                .as_bytes()
                .to_vec()
        ).unwrap(),
    },
    auxiliary_hexagrams: AuxiliaryHexagrams {
        bian_gua: (SingleGua::from_num(4), SingleGua::from_num(1)),
        hu_gua: (SingleGua::from_num(4), SingleGua::from_num(4)),
        cuo_gua: (SingleGua::from_num(5), SingleGua::from_num(4)),
        zong_gua: (SingleGua::from_num(5), SingleGua::from_num(4)),
        fu_gua: (SingleGua::from_num(5), SingleGua::from_num(4)),
    },
};
```

### 10.2 JSON格式（用于前端展示）

```json
{
  "basic_info": {
    "timestamp": 1703088600,
    "lunar_date": {
      "year": 2024,
      "month": 11,
      "day": 20,
      "hour_zhi_num": 1,
      "is_leap_month": false
    },
    "method": "DateTime",
    "gender": 1,
    "category": 1
  },
  "hexagram_core": {
    "shang_gua": "震",
    "xia_gua": "巽",
    "dong_yao": 1,
    "ti_is_shang": true
  },
  "tiyong_analysis": {
    "ti_wuxing": "木",
    "yong_wuxing": "木",
    "ben_gua_relation": "比和",
    "bian_gua_relation": "用克体",
    "hu_gua_relation": "比和",
    "ti_wangshuai": "相",
    "fortune": "小凶",
    "fortune_level": 1
  },
  "yingqi_analysis": {
    "ti_gua_num": 4,
    "yong_gua_num": 5,
    "primary_num": 9,
    "secondary_nums": [6, 6],
    "sheng_ti_wuxing": "水",
    "ke_ti_wuxing": "金",
    "analysis": "体卦木相，比和。喜神为水。应期数：9（可应年、月、日、时）。体卦数4，用卦数5。"
  },
  "auxiliary_hexagrams": {
    "bian_gua": ["震", "乾"],
    "hu_gua": ["震", "震"],
    "cuo_gua": ["巽", "震"],
    "zong_gua": ["巽", "震"],
    "fu_gua": ["巽", "震"]
  },
  "summary": {
    "ben_gua": "雷风恒",
    "bian_gua": "雷天大壮",
    "hu_gua": "雷为震",
    "fortune": "小凶偏平",
    "fortune_score": 1.8,
    "advice": "保持现状，等待时机",
    "best_timing": "春季或农历九月、六月",
    "avoid_timing": "秋季或金旺时节"
  }
}
```

---

## 总结

这是一个完整的梅花易数解卦流程，从起卦计算到最终建议，每一步都有详细的推理过程。

**核心要点**：
1. **本卦比和**：稳定但缺乏动力
2. **变卦用克体**：新机会暗藏风险
3. **体卦相旺**：有一定优势但不是最强
4. **应期在9或6**：农历九月或六月行动更佳
5. **最佳时机**：春季木旺时

**最终建议**：保持现状，稳扎稳打，等待更合适的时机（春季或应期）。

---

**文档版本**：v1.0
**创建时间**：2025-12-11
**案例类型**：事业运占卜
**解卦方法**：传统梅花易数 + 五行旺衰 + 应期推算
