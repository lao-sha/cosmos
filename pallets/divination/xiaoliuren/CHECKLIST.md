# 小六壬解卦模块开发任务清单

> **总时间**：6个工作日
> **核心文档**：[IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md)

---

## 🎯 开发进度总览

```
[  ] 环境准备     (0.5天)  Day 0
[  ] 阶段1 数据结构 (1天)    Day 1
[  ] 阶段2 核心算法 (2天)    Day 2-3
[  ] 阶段3 Runtime API (1天)  Day 4
[  ] 阶段4 集成测试 (1天)    Day 5
[  ] 阶段5 文档优化 (0.5天)  Day 5.5
```

---

## 📅 Day 0：环境准备（0.5天，约4小时）

### ✅ 准备工作
- [ ] 阅读设计文档 `INTERPRETATION_DESIGN.md`
- [ ] 阅读实施计划 `IMPLEMENTATION_PLAN.md`
- [ ] 检查现有代码结构
- [ ] 验证编译环境

### 🔧 环境检查
```bash
cd /home/xiaodong/文档/stardust/pallets/divination/xiaoliuren
cargo check
cargo test
```

- [ ] 现有模块编译通过
- [ ] 现有测试通过
- [ ] Git 工作区干净

### 📁 创建新文件
```bash
# 创建解卦模块目录
mkdir -p src/interpretation

# 创建文件
touch src/interpretation.rs
touch src/interpretation/mod.rs
touch src/interpretation/enums.rs
touch src/interpretation/core_struct.rs
touch src/interpretation/algorithms.rs
touch src/runtime_api.rs
touch src/interpretation_tests.rs
touch tests/interpretation_integration.rs
```

- [ ] 所有文件创建成功
- [ ] 目录结构正确

---

## 📅 Day 1：阶段1 - 数据结构（1天，约8小时）

### Step 1.1：基础框架（1小时）
- [ ] 创建 `src/interpretation.rs` 入口文件
- [ ] 添加模块文档注释
- [ ] 导入必要依赖
- [ ] 编译通过

### Step 1.2：枚举类型（2小时）
**文件：** `src/interpretation/enums.rs`

- [ ] 实现 `JiXiongLevel` 枚举（7种）
- [ ] 实现 `AdviceType` 枚举（8种）
- [ ] 实现 `YingQiType` 枚举（6种）
- [ ] 实现 `SpecialPattern` 位标志结构
- [ ] 为所有枚举添加方法：name(), description()
- [ ] 添加单元测试
- [ ] 编译通过，无警告

### Step 1.3：核心结构体（2小时）
**文件：** `src/interpretation/core_struct.rs`

- [ ] 实现 `XiaoLiuRenInterpretation` 结构体
- [ ] 验证大小为 13 字节
- [ ] 实现 `MaxEncodedLen` trait
- [ ] 添加辅助方法
- [ ] 添加单元测试
- [ ] 大小测试通过

### Step 1.4：更新 lib.rs（1小时）
- [ ] 在 `lib.rs` 声明 interpretation 模块
- [ ] 添加 `Interpretations` 存储项
- [ ] 导出公共类型
- [ ] 编译通过

### Step 1.5：验收（1小时）
```bash
cargo build
cargo test interpretation::enums
cargo test interpretation::core_struct
cargo doc --open
cargo clippy
cargo fmt -- --check
```

- [ ] 所有测试通过
- [ ] 无编译警告
- [ ] 文档生成成功
- [ ] Clippy 无警告

### 🎉 阶段1提交
```bash
git add src/interpretation/
git add src/lib.rs
git commit -m "feat(xiaoliuren): 实现解卦数据结构（阶段1）"
```

---

## 📅 Day 2-3：阶段2 - 核心算法（2天，约16小时）

### Day 2：基础算法（8小时）

#### Step 2.1：吉凶等级计算（3小时）
**文件：** `src/interpretation/algorithms.rs`

- [ ] 实现 `calculate_ji_xiong_level()` 函数
- [ ] 综合考虑：时宫、三宫、格局、体用
- [ ] 测试用例：全吉、全凶、纯宫、体用影响
- [ ] 边界测试通过

#### Step 2.2：综合评分计算（3小时）
- [ ] 实现 `calculate_overall_score()` 函数
- [ ] 五个维度计分：时宫40% + 三宫20% + 五行20% + 体用10% + 格局10%
- [ ] 分数范围验证（0-100）
- [ ] 各种情况测试通过

#### Step 2.3：特殊格局识别（2小时）
- [ ] 实现 `identify_special_pattern()` 函数
- [ ] 识别8种格局：纯宫、全吉、全凶、相生环、相克环、阴阳和、特殊时辰
- [ ] 位标志设置正确
- [ ] 测试所有格局

### Day 3：高级算法（8小时）

#### Step 2.4：应期计算（2小时）
- [ ] 实现 `calculate_ying_qi()` 函数
- [ ] 根据时宫判断：速喜→即刻，大安/小吉→当日，留连→延迟，空亡→难验，赤口→化解
- [ ] 测试所有六宫

#### Step 2.5：建议类型确定（2小时）
- [ ] 实现 `determine_advice_type()` 函数
- [ ] 综合吉凶等级和五行关系
- [ ] 特殊情况处理（化解）
- [ ] 测试所有建议类型

#### Step 2.6：核心解卦函数（3小时）
- [ ] 实现 `interpret()` 核心函数
- [ ] 整合所有算法步骤
- [ ] 完整流程测试
- [ ] 无时辰情况测试

#### Step 2.7：验收（1小时）
```bash
cargo test interpretation::algorithms
cargo test --all
cargo bench # 性能测试
```

- [ ] 单元测试覆盖率 > 90%
- [ ] 1000次解卦 < 10ms
- [ ] 集成测试通过

### 🎉 阶段2提交
```bash
git add src/interpretation/algorithms.rs
git commit -m "feat(xiaoliuren): 实现核心解卦算法（阶段2）"
```

---

## 📅 Day 4：阶段3 - Runtime API（1天，约8小时）

### Step 3.1：定义 Runtime API（2小时）
**文件：** `src/runtime_api.rs`

- [ ] 创建 Runtime API trait
- [ ] 定义 `get_interpretation()` 方法
- [ ] 定义 `get_interpretations_batch()` 方法
- [ ] 编译通过

### Step 3.2：实现 Pallet 方法（2小时）
**文件：** `src/lib.rs`

- [ ] 实现 `get_or_create_interpretation()` 懒加载
- [ ] 实现缓存机制
- [ ] 实现批量查询优化
- [ ] 单元测试通过

### Step 3.3：Runtime 集成（2小时）
**文件：** `runtime/src/apis.rs`

- [ ] 在 runtime 中实现 API trait
- [ ] 编译 runtime
- [ ] 测试 API 调用

### Step 3.4：验收（2小时）
```bash
cd runtime
cargo build --release
../target/release/solochain-template-node --dev --tmp
```

- [ ] Runtime 编译成功
- [ ] 节点启动正常
- [ ] RPC 调用成功（polkadot-js）

### 🎉 阶段3提交
```bash
git add src/runtime_api.rs src/lib.rs runtime/src/apis.rs
git commit -m "feat(xiaoliuren): 实现 Runtime API（阶段3）"
```

---

## 📅 Day 5：阶段4 - 集成测试（1天，约8小时）

### Step 4.1：单元测试完善（2小时）
**文件：** `src/interpretation_tests.rs`

- [ ] 完整流程测试
- [ ] 216种六宫组合测试
- [ ] 特殊格局测试
- [ ] 边界情况测试

### Step 4.2：集成测试（2小时）
**文件：** `tests/interpretation_integration.rs`

- [ ] 排盘→解卦→查询 完整流程
- [ ] Runtime API 测试
- [ ] 批量查询测试
- [ ] 懒加载测试

### Step 4.3：性能测试（2小时）
- [ ] Benchmark: 单次解卦 < 10微秒
- [ ] Benchmark: 批量100次 < 1ms
- [ ] 存储大小验证 = 13字节
- [ ] 内存占用测试

### Step 4.4：验收（2小时）
```bash
cargo test --all
cargo tarpaulin --out Html  # 覆盖率
cargo bench
```

- [ ] 所有测试通过
- [ ] 覆盖率 > 90%
- [ ] 性能达标
- [ ] 无内存泄漏

### 🎉 阶段4提交
```bash
git add tests/ src/interpretation_tests.rs
git commit -m "test(xiaoliuren): 完成解卦模块测试（阶段4）"
```

---

## 📅 Day 5.5：阶段5 - 文档与优化（0.5天，约4小时）

### Step 5.1：完善文档（2小时）
- [ ] 生成 API 文档 `cargo doc --no-deps`
- [ ] 编写 README.md 使用示例
- [ ] 更新设计文档（如有变化）
- [ ] 添加代码注释

### Step 5.2：代码优化（1小时）
```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt
cargo check --all-features
```

- [ ] Clippy 建议全部修复
- [ ] 代码格式化
- [ ] 性能热点优化（如需要）

### Step 5.3：最终验收（1小时）
```bash
# 完整测试流程
cargo check --all-features
cargo test --all
cargo doc --no-deps --open
cargo bench
```

- [ ] 所有检查通过
- [ ] 文档完整
- [ ] 代码质量达标

### 🎉 最终提交
```bash
git add .
git commit -m "docs(xiaoliuren): 完善文档和最终优化（阶段5）"
git push origin main
```

---

## 📊 质量检查清单

### 功能完整性
- [ ] 吉凶等级判断
- [ ] 综合评分算法
- [ ] 特殊格局识别
- [ ] 应期推算
- [ ] 建议生成
- [ ] Runtime API
- [ ] 懒加载缓存

### 性能指标
- [ ] 单次解卦 < 10微秒 ✅
- [ ] 批量100次 < 1ms ✅
- [ ] 存储大小 = 13字节 ✅
- [ ] 内存占用 < 1KB ✅

### 代码质量
- [ ] 测试覆盖率 > 90% ✅
- [ ] 无 Clippy 警告 ✅
- [ ] 代码格式化 ✅
- [ ] 文档完整 ✅

### 兼容性
- [ ] 道家流派支持
- [ ] 传统流派支持（预留）
- [ ] 无时辰情况兼容
- [ ] 批量查询支持

---

## 🚀 快速命令参考

### 开发命令
```bash
# 检查编译
cargo check

# 运行测试
cargo test

# 运行特定测试
cargo test interpretation

# 格式化
cargo fmt

# Lint
cargo clippy

# 文档
cargo doc --open

# Benchmark
cargo bench
```

### Git 提交模板
```bash
# 功能
git commit -m "feat(xiaoliuren): <描述>"

# 测试
git commit -m "test(xiaoliuren): <描述>"

# 文档
git commit -m "docs(xiaoliuren): <描述>"

# 修复
git commit -m "fix(xiaoliuren): <描述>"
```

---

## 📞 问题反馈

如遇到问题，参考：
1. [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - 详细实施计划
2. [INTERPRETATION_DESIGN.md](./INTERPRETATION_DESIGN.md) - 设计文档
3. [../liuyao/INTERPRETATION_DESIGN.md](../liuyao/INTERPRETATION_DESIGN.md) - 六爻参考

---

**创建时间**：2025-12-12
**预计完成**：2025-12-18（6个工作日）
**当前状态**：🟡 待开始
