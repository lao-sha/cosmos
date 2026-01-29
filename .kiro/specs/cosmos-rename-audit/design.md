# Design Document: Cosmos Rename Audit System

## Overview

本设计文档描述了一个系统化的审计和修复方案，用于识别和修复 Cosmos 区块链项目从 stardust 改名为 cosmos、代币从 DUST 改名为 COS 过程中遗漏的重命名项。系统采用模块化设计，包括扫描模块、分类模块、修复脚本生成模块和验证模块，确保修复过程的安全性和可追溯性。

## Architecture

系统采用管道式架构，包含以下主要阶段：

```mermaid
graph LR
    A[代码扫描] --> B[遗漏项分类]
    B --> C[修复脚本生成]
    C --> D[Dry-Run 预览]
    D --> E[批量修复执行]
    E --> F[验证检查]
    F --> G[生成报告]
```

### 核心模块

1. **Scanner Module**: 扫描项目文件，识别遗漏的重命名项
2. **Classifier Module**: 将遗漏项分类为不同类型（Cargo.toml、代码标识符、配置文件等）
3. **Script Generator Module**: 生成批量修复的 shell 脚本
4. **Validator Module**: 验证修复结果，确保没有遗漏或错误修改
5. **Reporter Module**: 生成详细的修复文档和报告

## Components and Interfaces

### 1. Scanner Module

**职责**: 扫描项目文件系统，使用正则表达式匹配遗漏的重命名项

**接口**:
```bash
scan_cargo_metadata(project_root: Path) -> List[CargoMetadataIssue]
scan_code_identifiers(project_root: Path) -> List[CodeIdentifierIssue]
scan_config_files(project_root: Path) -> List[ConfigFileIssue]
```

**实现策略**:
- 使用 `find` 命令定位特定类型的文件
- 使用 `grep` 或 `rg` (ripgrep) 进行模式匹配
- 排除 `target/`, `node_modules/`, `.git/` 等目录
- 使用白名单机制排除框架类型（如 `DustRemoval`）

**扫描模式**:

| 类别 | 文件类型 | 搜索模式 | 排除模式 |
|------|----------|----------|----------|
| Cargo 元数据 | `Cargo.toml` | `repository.*stardust`, `homepage.*stardust`, `description.*DUST` | - |
| 函数名 | `*.rs` | `fn.*dust.*\(`, `get_dust_`, `calculate_dust_`, `release_dust` | `DustRemoval` |
| 变量名 | `*.rs` | `let.*dust_`, `dust_amount`, `dust_qty`, `dust_to_usd_rate` | `DustRemoval` |
| 类型名 | `*.rs` | `type.*Dust`, `struct.*Dust`, `enum.*Dust` | `DustRemoval` |
| 配置路径 | `*.json`, `*.sh` | `stardust-node`, `/stardust/` | - |

### 2. Classifier Module

**职责**: 将扫描结果分类并确定修复优先级

**分类体系**:

```
遗漏项分类
├── P0: 编译依赖项（必须修复）
│   ├── Trait 定义中的函数名
│   ├── Config 类型名
│   └── 公共接口
├── P1: 实现层代码（高优先级）
│   ├── 函数实现
│   ├── 变量名
│   └── 结构体字段
├── P2: 元数据和文档（中优先级）
│   ├── Cargo.toml 元数据
│   ├── README 文档
│   └── 代码注释
└── P3: 配置文件（低优先级）
    ├── .claude 配置
    └── Shell 脚本
```

**接口**:
```bash
classify_issues(issues: List[Issue]) -> Dict[Priority, List[Issue]]
```

### 3. Script Generator Module

**职责**: 生成安全的批量修复脚本

**脚本结构**:
```bash
#!/bin/bash
set -e  # 遇到错误立即退出

# 配置
DRY_RUN=${DRY_RUN:-false}
LOG_FILE="rename-audit-$(date +%Y%m%d-%H%M%S).log"

# 函数：安全替换
safe_replace() {
    local pattern=$1
    local replacement=$2
    local file_pattern=$3
    local exclude_pattern=$4
    
    if [ "$DRY_RUN" = "true" ]; then
        echo "[DRY-RUN] Would replace '$pattern' with '$replacement' in $file_pattern"
        find . -type f -name "$file_pattern" \
            ! -path "*/target/*" \
            ! -path "*/node_modules/*" \
            ! -path "*/.git/*" \
            -exec grep -l "$pattern" {} \; | tee -a "$LOG_FILE"
    else
        find . -type f -name "$file_pattern" \
            ! -path "*/target/*" \
            ! -path "*/node_modules/*" \
            ! -path "*/.git/*" \
            -exec sed -i "s/$pattern/$replacement/g" {} \; \
            -exec echo "Modified: {}" \; | tee -a "$LOG_FILE"
    fi
}

# P0: Trait 定义（必须首先修改）
echo "=== Phase P0: Trait Definitions ==="
safe_replace "get_dust_to_usd_rate" "get_cos_to_usd_rate" "*.rs" ""
safe_replace "dust_qty" "cos_qty" "*.rs" ""

# P1: 实现层代码
echo "=== Phase P1: Implementation Code ==="
safe_replace "dust_amount" "cos_amount" "*.rs" ""
safe_replace "dust_to_usd_rate" "cos_to_usd_rate" "*.rs" ""
safe_replace "MinFirstPurchaseDustAmount" "MinFirstPurchaseCosAmount" "*.rs" ""
safe_replace "MaxFirstPurchaseDustAmount" "MaxFirstPurchaseCosAmount" "*.rs" ""
safe_replace "DustReleased" "CosReleased" "*.rs" ""
safe_replace "release_dust" "release_cos" "*.rs" ""
safe_replace "calculate_dust_amount" "calculate_cos_amount" "*.rs" ""
safe_replace "get_dust_market_price_weighted" "get_cos_market_price_weighted" "*.rs" ""

# P2: Cargo.toml 元数据
echo "=== Phase P2: Cargo Metadata ==="
safe_replace "https://github.com/stardust/stardust" "https://github.com/cosmos/cosmos" "Cargo.toml" ""
safe_replace "https://github.com/stardust-network/stardust" "https://github.com/cosmos-network/cosmos" "Cargo.toml" ""
safe_replace "DUST rewards" "COS rewards" "Cargo.toml" ""
safe_replace "DUST → USDT" "COS → USDT" "Cargo.toml" ""

# P2: 注释和文档
echo "=== Phase P2: Comments and Documentation ==="
safe_replace "COS 数量" "COS 数量" "*.rs" ""
safe_replace "COS 金额" "COS 金额" "*.rs" ""
safe_replace "释放 COS" "释放 COS" "*.rs" ""
safe_replace "USDT/COS" "USDT/COS" "*.rs" ""
safe_replace "COS/USD" "COS/USD" "*.rs" ""

# P3: 配置文件
echo "=== Phase P3: Configuration Files ==="
safe_replace "/home/xiaodong/文档/stardust/" "/home/xiaodong/文档/cosmos/" "*.json" ""
safe_replace "stardust-node" "cosmos-node" "*.sh" ""

echo "=== Rename Audit Complete ==="
echo "Log file: $LOG_FILE"
```

**接口**:
```bash
generate_script(classified_issues: Dict[Priority, List[Issue]]) -> String
```

### 4. Validator Module

**职责**: 验证修复结果的正确性和完整性

**验证检查项**:

1. **残留检查**: 搜索是否还有未修复的 stardust/DUST 引用
2. **框架类型检查**: 确认 DustRemoval 等框架类型未被错误修改
3. **编译检查**: 运行 `cargo check` 确保代码编译通过
4. **测试检查**: 运行 `cargo test` 确保测试通过

**接口**:
```bash
validate_no_residual(project_root: Path) -> ValidationResult
validate_framework_types(project_root: Path) -> ValidationResult
validate_compilation(project_root: Path) -> ValidationResult
validate_tests(project_root: Path) -> ValidationResult
```

**验证脚本**:
```bash
#!/bin/bash

echo "=== Validation Phase ==="

# 1. 检查残留的 stardust 引用（排除框架类型）
echo "Checking for residual 'stardust' references..."
STARDUST_COUNT=$(grep -r "stardust" --include="*.rs" --include="*.toml" --include="*.md" \
    --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git \
    . | grep -v "DustRemoval" | wc -l)

if [ $STARDUST_COUNT -gt 0 ]; then
    echo "❌ Found $STARDUST_COUNT residual 'stardust' references"
    grep -r "stardust" --include="*.rs" --include="*.toml" --include="*.md" \
        --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git \
        . | grep -v "DustRemoval"
else
    echo "✅ No residual 'stardust' references found"
fi

# 2. 检查残留的 dust_ 变量名前缀（排除框架类型）
echo "Checking for residual 'dust_' identifiers..."
DUST_VAR_COUNT=$(grep -r "dust_" --include="*.rs" \
    --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git \
    . | grep -v "DustRemoval" | wc -l)

if [ $DUST_VAR_COUNT -gt 0 ]; then
    echo "❌ Found $DUST_VAR_COUNT residual 'dust_' identifiers"
    grep -r "dust_" --include="*.rs" \
        --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git \
        . | grep -v "DustRemoval" | head -20
else
    echo "✅ No residual 'dust_' identifiers found"
fi

# 3. 确认框架类型未被错误修改
echo "Checking framework types are preserved..."
DUST_REMOVAL_COUNT=$(grep -r "DustRemoval" --include="*.rs" \
    --exclude-dir=target --exclude-dir=node_modules --exclude-dir=.git \
    . | wc -l)

if [ $DUST_REMOVAL_COUNT -gt 0 ]; then
    echo "✅ Framework type 'DustRemoval' is preserved ($DUST_REMOVAL_COUNT occurrences)"
else
    echo "⚠️  Warning: 'DustRemoval' not found - may have been incorrectly modified"
fi

# 4. 编译检查
echo "Running cargo check..."
if cargo check 2>&1 | tee cargo-check.log; then
    echo "✅ Cargo check passed"
else
    echo "❌ Cargo check failed - see cargo-check.log"
    exit 1
fi

# 5. 测试检查（可选，耗时较长）
if [ "${RUN_TESTS:-false}" = "true" ]; then
    echo "Running cargo test..."
    if cargo test 2>&1 | tee cargo-test.log; then
        echo "✅ Cargo test passed"
    else
        echo "❌ Cargo test failed - see cargo-test.log"
        exit 1
    fi
fi

echo "=== Validation Complete ==="
```

### 5. Reporter Module

**职责**: 生成详细的修复报告和文档

**报告格式**:
```markdown
# Cosmos Rename Audit Report

生成时间: 2024-XX-XX XX:XX:XX

## 执行摘要

- 扫描文件数: XXX
- 识别遗漏项: XXX
- 修复项数: XXX
- 验证状态: ✅ 通过 / ❌ 失败

## 修复详情

### P0: Trait 定义（3 项）

| 文件 | 行号 | 修改前 | 修改后 | 状态 |
|------|------|--------|--------|------|
| pallets/trading/common/src/traits.rs | 29 | get_dust_to_usd_rate | get_cos_to_usd_rate | ✅ |
| pallets/trading/common/src/traits.rs | 36 | dust_qty | cos_qty | ✅ |
| pallets/trading/common/src/traits.rs | 295 | dust_precision | cos_precision | ✅ |

### P1: 实现层代码（15 项）

...

### P2: 元数据和文档（20 项）

...

### P3: 配置文件（5 项）

...

## 验证结果

- ✅ 无残留 stardust 引用
- ✅ 无残留 dust_ 标识符
- ✅ 框架类型 DustRemoval 已保留
- ✅ Cargo check 通过
- ✅ Cargo test 通过

## 不应修改的项目

以下项目已正确保留，未被修改：

1. `type DustRemoval = ()` - Substrate 框架类型
2. `pallet_balances::DustRemoval` - 框架配置
3. `Cargo.lock` - 自动生成文件

## 回滚信息

如需回滚，执行以下命令：
\`\`\`bash
git checkout <backup-branch>
\`\`\`

或使用修改日志逆向脚本：
\`\`\`bash
./scripts/rollback-rename-audit.sh
\`\`\`
```

**接口**:
```bash
generate_report(scan_results: ScanResults, fix_results: FixResults, validation_results: ValidationResults) -> String
```

## Data Models

### Issue 数据结构

```rust
// 遗漏项基础结构
struct Issue {
    file_path: String,
    line_number: Option<u32>,
    category: IssueCategory,
    priority: Priority,
    old_value: String,
    suggested_value: String,
    context: String,  // 周围代码上下文
}

enum IssueCategory {
    CargoMetadata,
    FunctionName,
    VariableName,
    TypeName,
    StructField,
    EventName,
    ConfigPath,
    Comment,
    Documentation,
}

enum Priority {
    P0,  // 编译依赖，必须修复
    P1,  // 高优先级
    P2,  // 中优先级
    P3,  // 低优先级
}

// Cargo.toml 元数据问题
struct CargoMetadataIssue {
    base: Issue,
    field_name: String,  // repository, homepage, description
    package_name: String,
}

// 代码标识符问题
struct CodeIdentifierIssue {
    base: Issue,
    identifier_type: IdentifierType,  // Function, Variable, Type, Field, Event
    scope: String,  // 所在的模块或结构体
}

// 配置文件问题
struct ConfigFileIssue {
    base: Issue,
    config_type: ConfigType,  // ClaudeSettings, ShellScript
}
```

### 扫描结果数据结构

```rust
struct ScanResults {
    cargo_issues: Vec<CargoMetadataIssue>,
    code_issues: Vec<CodeIdentifierIssue>,
    config_issues: Vec<ConfigFileIssue>,
    total_files_scanned: u32,
    total_issues_found: u32,
    scan_timestamp: DateTime,
}
```

### 修复结果数据结构

```rust
struct FixResults {
    fixed_issues: Vec<Issue>,
    failed_issues: Vec<(Issue, String)>,  // (Issue, error_message)
    total_files_modified: u32,
    total_replacements: u32,
    fix_timestamp: DateTime,
    log_file_path: String,
}
```

### 验证结果数据结构

```rust
struct ValidationResults {
    residual_stardust_count: u32,
    residual_dust_identifiers: Vec<String>,
    framework_types_preserved: bool,
    compilation_passed: bool,
    tests_passed: bool,
    validation_timestamp: DateTime,
}
```

## Correctness Properties

*属性是一个特征或行为，应该在系统的所有有效执行中保持为真——本质上是关于系统应该做什么的正式陈述。属性作为人类可读规范和机器可验证正确性保证之间的桥梁。*

### Property 1: 完整的 Cargo.toml 元数据扫描
*For any* Cargo.toml 文件，扫描器应该识别所有包含 "stardust" 的 repository 和 homepage 字段，以及所有包含 "DUST" 的 description 字段
**Validates: Requirements 1.1, 1.2, 1.3**

### Property 2: 完整的代码标识符扫描
*For any* Rust 源代码文件，扫描器应该识别所有包含 "dust" 或 "Dust" 的标识符（函数名、变量名、类型名、结构体字段名、事件名）
**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 3: 框架类型保护不变量
*For any* 扫描、修复或验证操作，系统都不应该识别、修改或报告 Substrate 框架类型（DustRemoval、pallet_balances::DustRemoval）
**Validates: Requirements 2.6, 4.4, 5.4, 8.1, 8.2**

### Property 4: 配置文件路径扫描
*For any* .claude 配置文件或 shell 脚本文件，扫描器应该识别所有包含 "stardust" 或 "stardust-node" 的路径引用
**Validates: Requirements 3.1, 3.2**

### Property 5: 完整的报告信息
*For any* 识别出的遗漏项，生成的报告应该包含文件路径、行号（如适用）、当前值、建议修复值、修改理由和优先级
**Validates: Requirements 1.4, 1.5, 3.3, 3.4, 6.2, 6.3, 6.5**

### Property 6: 脚本安全性保证
*For any* 生成的批量修复脚本，应该包含排除目录的逻辑（target/、node_modules/、.git/），并且不包含会修改框架类型的命令
**Validates: Requirements 4.3, 4.4**

### Property 7: 脚本结构组织
*For any* 生成的批量修复脚本，应该按照优先级（P0、P1、P2、P3）和文件类型（.toml、.rs、.md、.sh、.json）分组执行替换
**Validates: Requirements 4.2**

### Property 8: 修复日志记录
*For any* 执行的批量修复操作，应该生成日志文件记录所有修改的文件路径和替换次数
**Validates: Requirements 4.5, 7.3**

### Property 9: 残留项验证完整性
*For any* 验证操作，应该搜索所有源代码文件中残留的 "stardust"、"dust_" 和 "Dust" 引用（排除框架类型），并报告数量和位置
**Validates: Requirements 5.1, 5.2, 5.3**

### Property 10: 框架类型验证
*For any* 验证操作，应该确认 DustRemoval 等框架类型仍然存在且未被修改
**Validates: Requirements 5.4**

### Property 11: Dry-run 模式幂等性
*For any* 批量修复脚本，在 dry-run 模式下执行应该不修改任何文件，并且可以多次执行产生相同的预览结果
**Validates: Requirements 7.2**

### Property 12: 回滚脚本可逆性
*For any* 修改日志文件，应该能够生成逆向脚本，使得执行修复脚本后再执行逆向脚本能够恢复到原始状态
**Validates: Requirements 7.5**

### Property 13: 注释中的代币符号替换
*For any* Rust 源代码文件中的注释，所有包含 "DUST" 的代币符号引用应该被识别并建议修改为 "COS"
**Validates: Requirements 8.3**

### Property 14: 测试文件常量名替换
*For any* 测试文件（包含 tests.rs 或 mock.rs），所有名为 "DUST" 的常量应该被识别并建议修改为 "COS"
**Validates: Requirements 8.5**

## Error Handling

### 扫描阶段错误处理

1. **文件访问错误**: 如果无法读取某个文件（权限问题、文件不存在等），记录警告但继续扫描其他文件
2. **编码错误**: 如果文件包含无效的 UTF-8 字符，尝试使用其他编码或跳过该文件
3. **正则表达式错误**: 如果正则表达式匹配失败，记录错误并使用备用的简单字符串匹配

### 修复阶段错误处理

1. **文件写入错误**: 如果无法写入文件，记录错误并继续处理其他文件，最后汇总失败列表
2. **备份失败**: 如果无法创建备份，中止修复操作并提示用户手动创建 Git 分支
3. **sed 命令失败**: 如果 sed 替换失败，记录详细错误信息（文件路径、命令、错误消息）

### 验证阶段错误处理

1. **编译失败**: 如果 cargo check 失败，保存完整的编译错误日志，并提示用户检查修复是否正确
2. **测试失败**: 如果 cargo test 失败，保存测试日志，但不阻止生成验证报告
3. **残留项检测失败**: 如果 grep 命令失败，记录错误但继续其他验证项

### 通用错误处理原则

1. **失败不中断**: 单个文件或操作的失败不应该中断整个流程
2. **详细日志**: 所有错误都应该记录到日志文件，包含时间戳、文件路径、错误类型和错误消息
3. **用户友好**: 错误消息应该清晰易懂，并提供可能的解决方案
4. **可恢复性**: 所有操作都应该支持从中断点恢复或完全回滚

## Testing Strategy

本项目采用双重测试策略，结合单元测试和属性测试，确保审计系统的正确性和可靠性。

### 单元测试

单元测试专注于具体的示例和边界情况：

1. **扫描器测试**:
   - 测试扫描包含特定模式的 Cargo.toml 文件
   - 测试扫描包含各种 dust 标识符的 Rust 文件
   - 测试正确排除 DustRemoval 框架类型
   - 测试处理空文件、大文件、特殊字符

2. **分类器测试**:
   - 测试正确分配优先级（P0、P1、P2、P3）
   - 测试正确分类问题类型
   - 测试处理边界情况（空列表、单个问题、大量问题）

3. **脚本生成器测试**:
   - 测试生成的脚本包含正确的 sed 命令
   - 测试生成的脚本包含排除目录逻辑
   - 测试 dry-run 模式生成正确的预览命令

4. **验证器测试**:
   - 测试检测残留的 stardust 引用
   - 测试确认框架类型未被修改
   - 测试处理编译错误和测试失败

### 属性测试

属性测试验证系统在所有输入下的通用正确性，每个测试至少运行 100 次迭代：

1. **Property 1 测试**: 生成随机的 Cargo.toml 文件（包含和不包含 stardust），验证扫描器识别所有包含 stardust/DUST 的元数据字段
   - **Tag**: Feature: cosmos-rename-audit, Property 1: 完整的 Cargo.toml 元数据扫描

2. **Property 2 测试**: 生成随机的 Rust 源文件（包含各种 dust 标识符），验证扫描器识别所有 dust/Dust 标识符
   - **Tag**: Feature: cosmos-rename-audit, Property 2: 完整的代码标识符扫描

3. **Property 3 测试**: 生成包含 DustRemoval 的随机代码，验证系统在所有阶段都不识别或修改框架类型
   - **Tag**: Feature: cosmos-rename-audit, Property 3: 框架类型保护不变量

4. **Property 5 测试**: 生成随机的遗漏项列表，验证报告包含所有必需信息
   - **Tag**: Feature: cosmos-rename-audit, Property 5: 完整的报告信息

5. **Property 6 测试**: 生成随机的问题列表，验证生成的脚本包含排除目录逻辑且不修改框架类型
   - **Tag**: Feature: cosmos-rename-audit, Property 6: 脚本安全性保证

6. **Property 11 测试**: 生成随机的文件系统状态，验证 dry-run 模式不修改文件且多次执行产生相同结果
   - **Tag**: Feature: cosmos-rename-audit, Property 11: Dry-run 模式幂等性

7. **Property 12 测试**: 生成随机的修改操作，验证逆向脚本能够恢复原始状态
   - **Tag**: Feature: cosmos-rename-audit, Property 12: 回滚脚本可逆性

### 测试工具和框架

- **属性测试库**: 根据实现语言选择（Python: Hypothesis, Rust: proptest, JavaScript: fast-check）
- **文件系统模拟**: 使用临时目录和模拟文件系统进行测试
- **Git 模拟**: 使用临时 Git 仓库测试备份和回滚功能

### 集成测试

1. **端到端测试**: 在真实的项目副本上运行完整的审计流程
2. **回归测试**: 使用已知的遗漏项列表验证系统能够正确识别
3. **性能测试**: 测试在大型项目（1000+ 文件）上的扫描性能

