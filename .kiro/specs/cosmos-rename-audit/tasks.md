# Implementation Plan: Cosmos Rename Audit

## Overview

本实施计划将 Cosmos 重命名审计系统分解为一系列可执行的脚本开发任务。系统采用模块化的 Bash 脚本设计，包括扫描脚本、分类脚本、修复脚本生成器、验证脚本和报告生成器。所有脚本都将放置在 `scripts/cosmos-rename-audit/` 目录下，确保代码组织清晰且易于维护。

## Tasks

- [ ] 1. 创建项目结构和核心工具函数
  - 创建 `scripts/cosmos-rename-audit/` 目录
  - 创建 `scripts/cosmos-rename-audit/lib/common.sh` 包含通用工具函数（日志、错误处理、颜色输出）
  - 创建 `scripts/cosmos-rename-audit/config.sh` 包含配置常量（排除目录、框架类型白名单等）
  - _Requirements: 4.3, 4.4, 8.1, 8.2_

- [ ] 2. 实现 Cargo.toml 元数据扫描器
  - [ ] 2.1 创建 `scripts/cosmos-rename-audit/scan-cargo-metadata.sh` 脚本
    - 使用 `find` 定位所有 Cargo.toml 文件（排除 target/、node_modules/、.git/）
    - 使用 `grep` 搜索 repository、homepage 字段中的 "stardust"
    - 使用 `grep` 搜索 description 字段中的 "DUST"
    - 输出 JSON 格式的扫描结果（文件路径、字段名、当前值、行号）
    - _Requirements: 1.1, 1.2, 1.3_

  - [ ] 2.2 编写 Cargo.toml 扫描器的单元测试
    - 创建测试 Cargo.toml 文件（包含和不包含 stardust/DUST）
    - 验证扫描器正确识别所有匹配项
    - 验证扫描器正确排除不相关的目录
    - _Requirements: 1.1, 1.2, 1.3_

- [ ] 3. 实现代码标识符扫描器
  - [ ] 3.1 创建 `scripts/cosmos-rename-audit/scan-code-identifiers.sh` 脚本
    - 使用 `find` 定位所有 .rs 文件（排除 target/、.git/）
    - 使用 `grep` 搜索函数名模式：`fn.*dust.*\(`、`get_dust_`、`calculate_dust_`、`release_dust`
    - 使用 `grep` 搜索变量名模式：`let.*dust_`、`dust_amount`、`dust_qty`、`dust_to_usd_rate`
    - 使用 `grep` 搜索类型名模式：`type.*Dust`、`struct.*Dust`、`enum.*Dust`、`Dust.*Amount`
    - 使用 `grep` 搜索结构体字段：`dust_amount:`、`total_dust:`
    - 使用 `grep` 搜索事件名：`DustReleased`
    - 使用白名单排除 `DustRemoval`、`pallet_balances::DustRemoval`
    - 输出 JSON 格式的扫描结果
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ] 3.2 编写代码标识符扫描器的单元测试
    - 创建测试 Rust 文件（包含各种 dust 标识符）
    - 验证扫描器正确识别所有类型的标识符
    - 验证扫描器正确排除框架类型（DustRemoval）
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

- [ ] 4. 实现配置文件路径扫描器
  - [ ] 4.1 创建 `scripts/cosmos-rename-audit/scan-config-files.sh` 脚本
    - 使用 `find` 定位所有 .json 和 .sh 文件
    - 使用 `grep` 搜索 .claude 配置文件中的 "stardust" 路径
    - 使用 `grep` 搜索 shell 脚本中的 "stardust-node" 引用
    - 输出 JSON 格式的扫描结果
    - _Requirements: 3.1, 3.2_

  - [ ] 4.2 编写配置文件扫描器的单元测试
    - 创建测试配置文件和脚本
    - 验证扫描器正确识别硬编码路径
    - _Requirements: 3.1, 3.2_

- [ ] 5. 实现问题分类器
  - [ ] 5.1 创建 `scripts/cosmos-rename-audit/classify-issues.sh` 脚本
    - 读取扫描器输出的 JSON 结果
    - 根据问题类型和位置分配优先级（P0: Trait 定义、P1: 实现代码、P2: 元数据文档、P3: 配置文件）
    - 为每个问题生成建议的修复值
    - 输出分类后的 JSON 结果
    - _Requirements: 1.4, 1.5, 3.3, 3.4, 6.5_

  - [ ] 5.2 编写分类器的单元测试
    - 测试正确分配优先级
    - 测试正确生成建议修复值
    - _Requirements: 6.5_

- [ ] 6. 实现批量修复脚本生成器
  - [ ] 6.1 创建 `scripts/cosmos-rename-audit/generate-fix-script.sh` 脚本
    - 读取分类后的问题列表
    - 按照优先级（P0 → P1 → P2 → P3）生成修复阶段
    - 为每个阶段生成 `safe_replace` 函数调用
    - 生成排除目录的 find 命令（! -path "*/target/*" ! -path "*/node_modules/*" ! -path "*/.git/*"）
    - 添加框架类型保护逻辑（使用 grep -v 排除 DustRemoval）
    - 生成 dry-run 模式支持
    - 生成日志记录逻辑
    - 输出完整的可执行 Bash 脚本到 `scripts/cosmos-rename-audit/fix-rename-issues.sh`
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [ ] 6.2 编写脚本生成器的单元测试
    - 测试生成的脚本包含正确的 sed 命令
    - 测试生成的脚本包含排除目录逻辑
    - 测试生成的脚本按优先级分组
    - _Requirements: 4.1, 4.2, 4.3_

- [ ] 7. 实现验证脚本
  - [ ] 7.1 创建 `scripts/cosmos-rename-audit/validate-fixes.sh` 脚本
    - 搜索残留的 "stardust" 引用（使用 grep -r，排除 target/、node_modules/、.git/）
    - 搜索残留的 "dust_" 变量名前缀（排除 DustRemoval）
    - 搜索残留的 "Dust" 类型名前缀（排除 DustRemoval）
    - 确认 DustRemoval 框架类型仍然存在
    - 运行 `cargo check` 并捕获输出
    - 可选：运行 `cargo test` 并捕获输出
    - 输出 JSON 格式的验证结果
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ] 7.2 编写验证脚本的单元测试
    - 测试正确检测残留项
    - 测试正确确认框架类型未被修改
    - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 8. 实现报告生成器
  - [ ] 8.1 创建 `scripts/cosmos-rename-audit/generate-report.sh` 脚本
    - 读取扫描结果、分类结果、修复结果和验证结果
    - 生成 Markdown 格式的详细报告
    - 按照类别组织遗漏项（Cargo.toml、代码标识符、配置文件）
    - 为每个遗漏项提供文件路径、行号、修改前后对比、理由、优先级
    - 列出不应修改的项目（DustRemoval 等框架类型）
    - 包含验证结果摘要
    - 提供回滚命令和说明
    - 输出报告到 `scripts/cosmos-rename-audit/audit-report.md`
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 5.6_

  - [ ] 8.2 编写报告生成器的单元测试
    - 测试报告包含所有必需的部分
    - 测试报告格式正确（Markdown 语法）
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 9. 实现主控脚本
  - [ ] 9.1 创建 `scripts/cosmos-rename-audit/run-audit.sh` 主脚本
    - 检查 Git 工作目录状态，提示创建备份分支
    - 按顺序执行：扫描 → 分类 → 生成修复脚本 → 生成报告
    - 提供交互式菜单：1) 仅扫描和报告 2) 生成修复脚本 3) 执行修复（dry-run） 4) 执行修复（实际） 5) 验证修复 6) 回滚
    - 处理错误并提供清晰的用户提示
    - _Requirements: 7.1, 7.4_

  - [ ] 9.2 编写主控脚本的集成测试
    - 在测试项目上运行完整的审计流程
    - 验证所有步骤正确执行
    - _Requirements: 所有需求_

- [ ] 10. 实现回滚功能
  - [ ] 10.1 创建 `scripts/cosmos-rename-audit/generate-rollback-script.sh` 脚本
    - 读取修复日志文件
    - 为每个修改操作生成逆向操作
    - 生成可执行的回滚脚本 `scripts/cosmos-rename-audit/rollback-fixes.sh`
    - _Requirements: 7.5_

  - [ ] 10.2 编写回滚脚本的单元测试
    - 测试逆向脚本能够恢复原始状态
    - 测试处理各种修改类型
    - _Requirements: 7.5_

- [ ] 11. 创建使用文档
  - [ ] 11.1 创建 `scripts/cosmos-rename-audit/README.md` 文档
    - 说明系统的目的和功能
    - 提供安装和使用说明
    - 列出所有脚本及其用途
    - 提供使用示例和最佳实践
    - 说明如何解读报告和执行修复
    - 提供故障排除指南

- [ ] 12. Checkpoint - 确保所有脚本可执行并通过基本测试
  - 运行所有单元测试，确保通过
  - 在测试项目上运行完整的审计流程
  - 验证生成的报告准确且完整
  - 验证修复脚本在 dry-run 模式下正确工作
  - 如有问题，询问用户并进行调整

## Notes

- 所有脚本都使用 Bash 编写，确保在 Linux/macOS 环境下可执行
- 使用 `set -e` 确保脚本在遇到错误时立即退出
- 使用 `set -u` 确保使用未定义变量时报错
- 所有脚本都应该有清晰的注释和使用说明
- JSON 输出使用 `jq` 工具进行格式化（如果可用）
- 颜色输出使用 ANSI 转义码，支持禁用（NO_COLOR 环境变量）
- 所有文件路径使用相对于项目根目录的路径
- 日志文件使用时间戳命名，避免覆盖
- 任务已全部设为必需，包含完整的单元测试和集成测试
