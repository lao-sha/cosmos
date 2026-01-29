# Requirements Document

## Introduction

本规格文档旨在系统地审计和修复 Cosmos 区块链项目从 stardust 改名为 cosmos、代币从 DUST 改名为 COS 的过程中遗漏的重命名项。通过代码搜索和分析，发现了多个类别的遗漏项，包括 Cargo.toml 元数据、代码中的变量名和函数名、注释文档、配置文件路径等。本项目将提供系统化的修复方案，确保代码一致性，同时避免破坏性修改，保留 Substrate 框架相关的 Dust 命名。

## Glossary

- **System**: 重命名审计系统，负责识别、分类和修复遗漏的重命名项
- **Rename_Item**: 需要从旧名称（stardust/DUST）更新为新名称（cosmos/COS）的代码、配置或文档项
- **Framework_Type**: Substrate 框架定义的类型名（如 DustRemoval），不应修改
- **Cargo_Metadata**: Cargo.toml 文件中的 repository、homepage、description 等元数据字段
- **Code_Identifier**: 代码中的变量名、函数名、类型名、结构体字段名等标识符
- **Batch_Script**: 用于批量修复遗漏项的自动化脚本

## Requirements

### Requirement 1: 识别 Cargo.toml 元数据中的遗漏项

**User Story:** 作为开发者，我希望识别所有 Cargo.toml 文件中包含 stardust 相关 URL 的元数据字段，以便统一更新为 cosmos 相关 URL。

#### Acceptance Criteria

1. WHEN 扫描项目中的 Cargo.toml 文件 THEN THE System SHALL 识别所有 repository 字段中包含 "stardust" 的文件
2. WHEN 扫描项目中的 Cargo.toml 文件 THEN THE System SHALL 识别所有 homepage 字段中包含 "stardust" 的文件
3. WHEN 扫描项目中的 Cargo.toml 文件 THEN THE System SHALL 识别所有 description 字段中包含 "DUST" 的文件
4. WHEN 生成识别报告 THEN THE System SHALL 列出文件路径、字段名称和当前值
5. WHEN 生成识别报告 THEN THE System SHALL 提供建议的修复值

### Requirement 2: 识别代码标识符中的遗漏项

**User Story:** 作为开发者，我希望识别代码中所有包含 dust 的变量名、函数名和类型名，以便统一更新为 cos 相关命名。

#### Acceptance Criteria

1. WHEN 扫描 Rust 源代码文件 THEN THE System SHALL 识别所有包含 "dust" 的函数名（如 get_dust_to_usd_rate）
2. WHEN 扫描 Rust 源代码文件 THEN THE System SHALL 识别所有包含 "dust" 的变量名（如 dust_amount、dust_qty）
3. WHEN 扫描 Rust 源代码文件 THEN THE System SHALL 识别所有包含 "Dust" 的 Config 类型名（如 MinFirstPurchaseDustAmount）
4. WHEN 扫描 Rust 源代码文件 THEN THE System SHALL 识别所有包含 "dust" 的结构体字段名
5. WHEN 扫描 Rust 源代码文件 THEN THE System SHALL 识别所有包含 "Dust" 的事件名（如 DustReleased）
6. WHEN 识别到标识符 THEN THE System SHALL 排除 Substrate 框架类型（如 DustRemoval、pallet_balances::DustRemoval）

### Requirement 3: 识别配置文件中的硬编码路径

**User Story:** 作为开发者，我希望识别配置文件中所有包含 stardust 的硬编码路径，以便更新为 cosmos 路径。

#### Acceptance Criteria

1. WHEN 扫描 .claude 配置文件 THEN THE System SHALL 识别所有包含 "stardust" 路径的 JSON 字段
2. WHEN 扫描 shell 脚本文件 THEN THE System SHALL 识别所有包含 "stardust-node" 的引用
3. WHEN 生成识别报告 THEN THE System SHALL 列出文件路径和具体的硬编码内容
4. WHEN 生成识别报告 THEN THE System SHALL 提供建议的修复值

### Requirement 4: 生成批量修复脚本

**User Story:** 作为开发者，我希望获得自动化的批量修复脚本，以便高效地修复所有遗漏项。

#### Acceptance Criteria

1. WHEN 生成修复脚本 THEN THE Batch_Script SHALL 使用 sed 命令进行文本替换
2. WHEN 生成修复脚本 THEN THE Batch_Script SHALL 按照文件类型分组执行替换（Cargo.toml、.rs、.md、.sh、.json）
3. WHEN 生成修复脚本 THEN THE Batch_Script SHALL 排除不应修改的目录（target/、node_modules/、.git/）
4. WHEN 生成修复脚本 THEN THE Batch_Script SHALL 排除 Substrate 框架类型（DustRemoval）
5. WHEN 执行修复脚本 THEN THE Batch_Script SHALL 记录修改的文件数量和匹配数量
6. WHEN 执行修复脚本失败 THEN THE Batch_Script SHALL 提供清晰的错误信息

### Requirement 5: 验证修复结果

**User Story:** 作为开发者，我希望验证所有修复是否正确完成，以确保没有遗漏或错误修改。

#### Acceptance Criteria

1. WHEN 执行验证 THEN THE System SHALL 搜索所有源代码文件中残留的 "stardust" 引用
2. WHEN 执行验证 THEN THE System SHALL 搜索所有源代码文件中残留的 "dust_" 变量名前缀
3. WHEN 执行验证 THEN THE System SHALL 搜索所有源代码文件中残留的 "Dust" 类型名前缀（排除 DustRemoval）
4. WHEN 执行验证 THEN THE System SHALL 确认 Substrate 框架类型未被错误修改
5. WHEN 执行验证 THEN THE System SHALL 运行 cargo check 确保代码编译通过
6. WHEN 生成验证报告 THEN THE System SHALL 列出所有残留项和验证结果

### Requirement 6: 生成修复文档

**User Story:** 作为开发者，我希望获得详细的修复文档，记录所有修改项和修复理由。

#### Acceptance Criteria

1. WHEN 生成修复文档 THEN THE System SHALL 按照类别组织遗漏项（Cargo.toml、代码标识符、配置文件等）
2. WHEN 生成修复文档 THEN THE System SHALL 为每个遗漏项提供文件路径、行号和修改前后对比
3. WHEN 生成修复文档 THEN THE System SHALL 说明每个修改的理由和影响范围
4. WHEN 生成修复文档 THEN THE System SHALL 列出不应修改的项目和原因
5. WHEN 生成修复文档 THEN THE System SHALL 提供修复优先级建议（高/中/低）

### Requirement 7: 支持安全回滚

**User Story:** 作为开发者，我希望在修复出现问题时能够安全回滚，以避免破坏性影响。

#### Acceptance Criteria

1. WHEN 开始修复前 THEN THE System SHALL 提示创建 Git 备份分支
2. WHEN 执行批量修复 THEN THE Batch_Script SHALL 支持 dry-run 模式预览修改
3. WHEN 执行批量修复 THEN THE Batch_Script SHALL 生成修改日志文件
4. WHEN 需要回滚 THEN THE System SHALL 提供 Git 回滚命令
5. WHEN 需要回滚 THEN THE System SHALL 提供基于修改日志的逆向脚本

### Requirement 8: 处理特殊情况

**User Story:** 作为开发者，我希望系统能够正确处理特殊情况，避免误修改。

#### Acceptance Criteria

1. WHEN 遇到 "DustRemoval" 类型 THEN THE System SHALL 保留不修改（Substrate 框架类型）
2. WHEN 遇到 "pallet_balances::DustRemoval" 配置 THEN THE System SHALL 保留不修改
3. WHEN 遇到注释中的 "DUST" THEN THE System SHALL 修改为 "COS"
4. WHEN 遇到字符串字面量中的 "dust" THEN THE System SHALL 根据上下文决定是否修改
5. WHEN 遇到测试文件中的常量名 "DUST" THEN THE System SHALL 修改为 "COS"
