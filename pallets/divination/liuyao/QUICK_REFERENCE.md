# 六爻解卦快速参考

## 核心类型速查表

### JiXiongLevel（吉凶等级）
| 值 | 名称 | 说明 |
|----|------|------|
| 0 | 大吉 | 诸事顺遂，心想事成 |
| 1 | 吉 | 事可成，宜进取 |
| 2 | 小吉 | 小有所得，不宜大动 |
| 3 | 平 | 平稳无波，守成为上 |
| 4 | 小凶 | 小有阻碍，谨慎行事 |
| 5 | 凶 | 事难成，宜退守 |
| 6 | 大凶 | 诸事不利，静待时机 |

### YongShenState（用神状态）
| 值 | 名称 | 说明 | 有利 |
|----|------|------|------|
| 0 | 旺相 | 得时得地，事情有利 | ✅ |
| 1 | 休囚 | 失时失地，事情不利 | ❌ |
| 2 | 动化进 | 动爻化进神，事情向好发展 | ✅ |
| 3 | 动化退 | 动爻化退神，事情有退步之象 | ❌ |
| 4 | 动化空 | 动爻化空亡，事情虚而不实 | ❌ |
| 5 | 伏藏 | 伏神状态，所求之事隐而未显 | ❌ |
| 6 | 空亡 | 日空或月空，所求之事虚而不实 | ❌ |
| 7 | 入墓 | 入墓库，事情受阻，需待时机 | ❌ |
| 8 | 受克 | 被克制，所求之事受阻 | ❌ |
| 9 | 得生 | 被生扶，所求之事有贵人相助 | ✅ |

### ShiXiangType（占问事项）
| 值 | 名称 | 默认用神 |
|----|------|---------|
| 0 | 财运 | 妻财 |
| 1 | 事业 | 官鬼 |
| 2 | 婚姻感情 | 妻财/官鬼 |
| 3 | 健康 | 世爻 |
| 4 | 考试学业 | 父母 |
| 5 | 官司诉讼 | 官鬼 |
| 6 | 出行 | 世爻 |
| 7 | 寻人寻物 | 用事之爻 |
| 8 | 天气 | 相关爻 |
| 9 | 其他 | 自定义 |

### YingQiType（应期类型）
| 值 | 名称 | 时间范围 |
|----|------|---------|
| 0 | 近期 | 日内 |
| 1 | 短期 | 月内 |
| 2 | 中期 | 季度内 |
| 3 | 长期 | 年内 |
| 4 | 远期 | 年后 |
| 5 | 不确定 | 需要进一步分析 |

## 使用示例

### 创建核心解卦结果
```rust
use pallet_liuyao::interpretation::*;

// 创建新的解卦结果
let mut interpretation = LiuYaoCoreInterpretation::new(1000000);

// 设置基础判断
interpretation.ji_xiong = JiXiongLevel::Ji;
interpretation.yong_shen_qin = LiuQin::QiCai;
interpretation.yong_shen_state = YongShenState::WangXiang;
interpretation.yong_shen_pos = 2; // 第2爻

// 设置动态分析
interpretation.shi_yao_state = YongShenState::WangXiang;
interpretation.ying_yao_state = YongShenState::XiuQiu;
interpretation.dong_yao_count = 1;
interpretation.dong_yao_bitmap = 0b000100; // 第2爻为动爻

// 设置应期
interpretation.ying_qi = YingQiType::DuanQi;
interpretation.ying_qi_zhi = 3; // 卯月

// 设置评分
interpretation.score = 75;
interpretation.confidence = 85;
```

### 检查用神状态
```rust
// 检查用神是否有利
if interpretation.is_yong_shen_favorable() {
    println!("用神有利，事情向好发展");
}

// 检查用神是否不利
if interpretation.is_yong_shen_unfavorable() {
    println!("用神不利，需要谨慎");
}
```

### 检查爻位状态
```rust
// 检查第2爻是否为动爻
if interpretation.is_dong_yao(2) {
    println!("第2爻为动爻");
}

// 检查第3爻是否逢空
if interpretation.is_xun_kong(3) {
    println!("第3爻逢空");
}

// 检查第4爻是否月破
if interpretation.is_yue_po(4) {
    println!("第4爻月破");
}

// 检查第5爻是否日冲
if interpretation.is_ri_chong(5) {
    println!("第5爻日冲");
}
```

### 获取文本描述
```rust
// 获取吉凶等级名称
println!("吉凶：{}", interpretation.ji_xiong.name());

// 获取用神状态名称
println!("用神状态：{}", interpretation.yong_shen_state.name());

// 获取应期名称
println!("应期：{}", interpretation.ying_qi.name());

// 获取解卦文本
let text = JieGuaTextType::YongShenWangXiang.text();
println!("解卦：{}", text);
```

## 位图操作指南

### 动爻位图（dong_yao_bitmap）
```
位置：5 4 3 2 1 0
爻位：上 五 四 三 二 初

例如：0b000101 表示初爻和三爻为动爻
```

### 旬空位图（xun_kong_bitmap）
```
同上，表示哪些爻逢空
```

### 月破位图（yue_po_bitmap）
```
同上，表示哪些爻月破
```

### 日冲位图（ri_chong_bitmap）
```
同上，表示哪些爻日冲
```

### 化空位图（hua_kong_bitmap）
```
同上，表示动爻变化为空亡的情况
```

## 编码大小

| 类型 | 大小 |
|------|------|
| JiXiongLevel | 1 byte |
| YongShenState | 1 byte |
| ShiXiangType | 1 byte |
| YingQiType | 1 byte |
| JieGuaTextType | 1 byte |
| LiuYaoCoreInterpretation | ~20 bytes |

## 常用判断逻辑

### 吉凶判断
```rust
// 判断是否为吉
if interpretation.ji_xiong.is_ji() {
    println!("吉");
}

// 判断是否为凶
if interpretation.ji_xiong.is_xiong() {
    println!("凶");
}
```

### 用神判断
```rust
// 用神旺相 + 无克制 → 吉
if interpretation.yong_shen_state.is_favorable() {
    println!("用神有利，事情向好");
}

// 用神休囚 + 有克制 → 凶
if interpretation.yong_shen_state.is_unfavorable() {
    println!("用神不利，事情受阻");
}
```

### 动爻判断
```rust
// 无动爻
if interpretation.dong_yao_count == 0 {
    println!("无动爻，事情平稳");
}

// 一爻独发
if interpretation.dong_yao_count == 1 {
    println!("一爻独发，吉凶易断");
}

// 多爻齐动
if interpretation.dong_yao_count > 1 {
    println!("多爻齐动，变数较多");
}

// 六爻皆动
if interpretation.dong_yao_count == 6 {
    println!("六爻皆动，大变之象");
}
```

## 测试命令

```bash
# 编译检查
cargo check

# 运行所有测试
cargo test --lib interpretation

# 运行特定测试
cargo test --lib interpretation::tests::test_core_interpretation_size

# 查看测试输出
cargo test --lib interpretation -- --nocapture
```

## 相关文件

- `src/interpretation.rs` - 核心实现
- `INTERPRETATION_DESIGN.md` - 详细设计文档
- `PHASE1_IMPLEMENTATION_SUMMARY.md` - 实现总结
- `QUICK_REFERENCE.md` - 本文件
