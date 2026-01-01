# 十二长生表校准文档

## 📌 校准概述

根据权威命理典籍（《渊海子平》、《三命通会》、《滴天髓》），对 `xingyun.rs` 模块的十二长生查询表进行精确校准。

## ✅ 校准完成状态

- ✅ 阳干（甲丙戊庚壬）十二长生表
- ✅ 阴干（乙丁己辛癸）十二长生表
- ✅ 19个单元测试全部通过
- ✅ 编译无警告无错误

## 🎯 十二长生规则

### 阳干顺行规则

阳干从长生位开始**顺时针**经过十二长生状态：

| 天干 | 长生 | 沐浴 | 冠带 | 临官 | 帝旺 | 衰 | 病 | 死 | 墓 | 绝 | 胎 | 养 |
|------|------|------|------|------|------|-----|-----|-----|-----|-----|-----|-----|
| **甲木** | 亥 | 子 | 丑 | 寅 | 卯 | 辰 | 巳 | 午 | 未 | 申 | 酉 | 戌 |
| **丙火** | 寅 | 卯 | 辰 | 巳 | 午 | 未 | 申 | 酉 | 戌 | 亥 | 子 | 丑 |
| **戊土** | 寅 | 卯 | 辰 | 巳 | 午 | 未 | 申 | 酉 | 戌 | 亥 | 子 | 丑 |
| **庚金** | 巳 | 午 | 未 | 申 | 酉 | 戌 | 亥 | 子 | 丑 | 寅 | 卯 | 辰 |
| **壬水** | 申 | 酉 | 戌 | 亥 | 子 | 丑 | 寅 | 卯 | 辰 | 巳 | 午 | 未 |

### 阴干逆行规则

阴干从长生位开始**逆时针**经过十二长生状态：

| 天干 | 长生 | 沐浴 | 冠带 | 临官 | 帝旺 | 衰 | 病 | 死 | 墓 | 绝 | 胎 | 养 |
|------|------|------|------|------|------|-----|-----|-----|-----|-----|-----|-----|
| **乙木** | 午 | 巳 | 辰 | 卯 | 寅 | 丑 | 子 | 亥 | 戌 | 酉 | 申 | 未 |
| **丁火** | 酉 | 申 | 未 | 午 | 巳 | 辰 | 卯 | 寅 | 丑 | 子 | 亥 | 戌 |
| **己土** | 酉 | 申 | 未 | 午 | 巳 | 辰 | 卯 | 寅 | 丑 | 子 | 亥 | 戌 |
| **辛金** | 子 | 亥 | 戌 | 酉 | 申 | 未 | 午 | 巳 | 辰 | 卯 | 寅 | 丑 |
| **癸水** | 卯 | 寅 | 丑 | 子 | 亥 | 戌 | 酉 | 申 | 未 | 午 | 巳 | 辰 |

## 📚 命理口诀

### 阳干长生诀

```
甲木长生在亥方，寅临官卯帝旺
丙火寅中寻长生，巳临官午帝旺
戊土同推丙火寻，寅中长生火生土
庚金巳中寻长生，申临官酉帝旺
壬水申中寻长生，亥临官子帝旺
```

### 阴干长生诀

```
乙木长生午上行，从午逆数至未养
丁火酉中寻长生，己土同推不用更
辛金子位是长生，癸水卯中福禄增
```

## 🔍 关键概念

### 1. 长生

- 如人之初生，生命开始
- 代表新生、希望、发展的起点
- **影响**：主贵气，有发展潜力

### 2. 沐浴（败地）

- 如婴儿沐浴，脆弱之时
- 代表不稳定、易受外界影响
- **影响**：多桃花、易破财

### 3. 冠带

- 如人戴冠束带，渐成气象
- 代表逐步成长、逐渐稳定
- **影响**：吉中带平

### 4. 临官（建禄）

- 如人临官任职，得意之时
- 代表能力充分发挥
- **影响**：主贵，事业有成

### 5. 帝旺

- 如帝王当朝，最旺盛时
- 代表能量最强、最辉煌的阶段
- **影响**：主富贵，但物极必反

### 6. 衰

- 如人年老体衰
- 代表能量开始衰退
- **影响**：需要休养生息

### 7. 病

- 如人疾病缠身
- 代表身体或事业出现问题
- **影响**：多灾病、挫折

### 8. 死

- 如人气绝身亡
- 代表能量极弱
- **影响**：主凶，但非真死

### 9. 墓（库）

- 如人入墓归土
- 代表收藏、隐藏、终结
- **影响**：有财库之象，但也主困顿

### 10. 绝

- 如人形骸俱灭
- 代表能量完全消失
- **影响**：主凶，但绝处逢生

### 11. 胎

- 如人受胎于母腹
- 代表新的开始、孕育
- **影响**：潜藏希望

### 12. 养

- 如人在母腹中成形
- 代表培育、成长
- **影响**：需要养护

## 🎯 命理应用

### 1. 判断日主旺衰

- **日主在月令的长生状态最重要**（月令决定旺衰80%）
- 帝旺、临官、长生、冠带 → 身旺
- 死、墓、绝、病、衰 → 身弱

### 2. 判断六亲旺衰

- 财星在帝旺、临官 → 财运佳
- 官星在帝旺、临官 → 官运亨通
- 印星在帝旺、临官 → 得母助

### 3. 判断大运流年

- 大运走到帝旺、临官运 → 发达之期
- 大运走到死、墓、绝运 → 低谷之期

## 📊 测试验证

### 测试覆盖

共19个测试用例，覆盖：

**阳干测试（9个）**：
- ✅ 甲木：长生在亥、临官在寅、帝旺在卯
- ✅ 丙火：长生在寅、帝旺在午
- ✅ 庚金：长生在巳、帝旺在酉
- ✅ 壬水：长生在申、帝旺在子

**阴干测试（8个）**：
- ✅ 乙木：长生在午、帝旺在寅
- ✅ 丁火：长生在酉、帝旺在巳
- ✅ 辛金：长生在子、帝旺在申
- ✅ 癸水：长生在卯、帝旺在子

**状态判断测试（2个）**：
- ✅ `is_prosperous()` - 判断旺相状态
- ✅ `is_declining()` - 判断衰败状态

### 测试结果

```bash
$ cargo test -p pallet-bazi-chart xingyun

running 19 tests
test xingyun::tests::test_binghuo_changsheng_at_yin ... ok
test xingyun::tests::test_binghuo_diwang_at_wu ... ok
test xingyun::tests::test_dinghuo_changsheng_at_you ... ok
test xingyun::tests::test_dinghuo_diwang_at_si ... ok
test xingyun::tests::test_gengjin_changsheng_at_si ... ok
test xingyun::tests::test_gengjin_diwang_at_you ... ok
test xingyun::tests::test_guishui_changsheng_at_mao ... ok
test xingyun::tests::test_guishui_diwang_at_zi ... ok
test xingyun::tests::test_is_declining ... ok
test xingyun::tests::test_is_prosperous ... ok
test xingyun::tests::test_jiamu_changsheng_at_hai ... ok
test xingyun::tests::test_jiamu_diwang_at_mao ... ok
test xingyun::tests::test_jiamu_linguan_at_yin ... ok
test xingyun::tests::test_renshui_changsheng_at_shen ... ok
test xingyun::tests::test_renshui_diwang_at_zi ... ok
test xingyun::tests::test_xinjin_changsheng_at_zi ... ok
test xingyun::tests::test_xinjin_diwang_at_shen ... ok
test xingyun::tests::test_yimu_changsheng_at_wu ... ok
test xingyun::tests::test_yimu_diwang_at_yin ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured
```

**✅ 全部通过！**

## 📝 代码示例

### Rust 调用示例

```rust
use pallet_bazi_chart::xingyun::get_changsheng;
use pallet_bazi_chart::types::{TianGan, DiZhi, ShiErChangSheng};

// 查询甲木在寅的十二长生状态
let rizhu = TianGan(0); // 甲木
let dizhi = DiZhi(2);   // 寅
let changsheng = get_changsheng(rizhu, dizhi);

assert_eq!(changsheng, ShiErChangSheng::LinGuan); // 临官（建禄）

// 判断是否为旺相状态
if changsheng.is_prosperous() {
    println!("日主得力，身旺！");
}
```

### 四柱星运示例

```rust
use pallet_bazi_chart::xingyun::calculate_xingyun;

// 假设日主为甲木
let sizhu = SiZhu { ... };
let xingyun = calculate_xingyun(&sizhu);

// 查看月令长生状态（最重要）
match xingyun.month_changsheng {
    ShiErChangSheng::DiWang => println!("月令帝旺，身旺之命"),
    ShiErChangSheng::LinGuan => println!("月令临官，富贵之命"),
    ShiErChangSheng::ChangSheng => println!("月令长生，有发展"),
    ShiErChangSheng::Si => println!("月令死地，身弱"),
    _ => println!("其他状态"),
}
```

## 🔗 参考资料

### 权威典籍

1. **《渊海子平》** - 宋代徐大升著
   - 最早系统论述十二长生的典籍
   - 确立了阳顺阴逆的规则

2. **《三命通会》** - 明代万民英著
   - 集命理之大成
   - 详细论述十二长生在实际命理中的应用

3. **《滴天髓》** - 清代刘基著
   - 命理精髓之作
   - 论述十二长生与日主旺衰的关系

### 现代参考

- lunar-java 项目的十二长生实现
- BaziGo 项目的长生查询表
- bazi-mcp 项目的星运计算

## ⚠️ 注意事项

### 1. 阴阳顺逆的争议

十二长生的阴干规则在命理界有两种说法：

**主流派（本项目采用）**：
- 阴干从长生位开始**逆行**
- 如乙木长生在午，从午逆数：午→巳→辰→卯（临官）→寅（帝旺）

**少数派**：
- 阴干与阳干相同，都顺行
- 但长生位不同

**选择理由**：
- 主流派有《渊海子平》、《三命通会》等经典支持
- 符合阴阳对立的哲学思想
- 经过大量实践验证

### 2. 戊己土的争议

戊土和己土的长生位有争议：

**主流派（本项目采用）**：
- 戊土同丙火（长生在寅）
- 己土同丁火（长生在酉）
- 理由：火生土，土依火而生

**四季派**：
- 戊己土寄生四季（辰戌丑未）
- 理由：土旺于四季末

**本项目选择主流派**，因为：
- 更符合五行相生的逻辑
- 实践验证效果更好

## ✅ 校准验收

- ✅ 查询表完全符合《渊海子平》规则
- ✅ 阳干顺行、阴干逆行规则正确
- ✅ 19个测试用例全部通过
- ✅ 编译无警告无错误
- ✅ 代码注释详细，包含口诀和参考资料

---

**校准完成时间**: 2025-12-20
**校准人员**: Claude Code
**文档版本**: v1.0
**参考典籍**: 《渊海子平》《三命通会》《滴天髓》
