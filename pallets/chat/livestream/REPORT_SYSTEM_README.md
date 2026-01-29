# 直播间举报与申诉系统

## 📖 概述

为 `pallet-livestream` 添加了完整的举报与申诉系统，支持用户举报违规直播间、委员会审核、直播间封禁与申诉等功能。

## ✨ 核心特性

- ✅ **用户举报**: 支持5种举报类型，匿名举报可选
- ✅ **防滥用机制**: 10 COS押金 + 1天冷却期
- ✅ **灵活撤回**: 12小时内可撤回，退还80%押金
- ✅ **委员会审核**: 三种结果（成立/驳回/恶意）
- ✅ **自动过期**: 7天未处理自动过期，全额退款
- ✅ **申诉保障**: 被封禁可申诉，治理审核
- ✅ **完整测试**: 36个单元测试，100%通过

## 🚀 快速开始

### 举报直播间

```rust
Livestream::report_room(
    origin,
    room_id: 0,
    report_type: RoomReportType::IllegalContent,
    evidence_cid: b"QmXXXXX".to_vec(),
    description: b"违规内容描述".to_vec(),
    is_anonymous: false,
)
```

### 撤回举报

```rust
Livestream::withdraw_room_report(origin, report_id: 0)
```

### 审核举报（委员会）

```rust
Livestream::resolve_room_report(
    origin,
    report_id: 0,
    result: ReportStatus::Upheld,
    resolution_note: Some(b"处理说明".to_vec()),
)
```

### 申诉封禁

```rust
Livestream::appeal_room_ban(
    origin,
    room_id: 0,
    appeal_evidence_cid: b"QmYYYYY".to_vec(),
    appeal_reason: b"申诉理由".to_vec(),
)
```

## 📊 Call Index 分配

| 函数 | Call Index | 权限 |
|------|-----------|------|
| `report_room` | 70 | 任何用户 |
| `withdraw_room_report` | 71 | 举报者 |
| `resolve_room_report` | 72 | ContentCommittee |
| `expire_room_report` | 73 | 任何用户 |
| `appeal_room_ban` | 74 | 主播 |
| `resolve_room_ban_appeal` | 75 | GovernanceOrigin |

## 🔒 安全机制

### 防滥用保护
- 10 COS 押金要求
- 1天举报冷却期
- 不能举报自己的直播间

### 经济激励
- 正当举报：全额退还押金
- 恶意举报：没收押金
- 撤回举报：扣除20%罚金

### 超时保护
- 7天未处理自动过期
- 全额退还押金
- 任何人可触发

## 📈 业务流程

### 举报流程

```
用户举报 → [12h内可撤回] → 委员会审核 → 成立/驳回/恶意
                                    ↓
                              封禁直播间
                                    ↓
                              主播申诉 → 治理审核 → 成立/驳回
```

### 状态转换

```
Pending → Withdrawn (用户撤回)
Pending → Upheld (举报成立)
Pending → Rejected (举报驳回)
Pending → Malicious (恶意举报)
Pending → Expired (超时过期)
```

## 🧪 测试

```bash
# 运行所有测试
cargo test --package pallet-livestream --lib

# 运行举报系统测试
cargo test --package pallet-livestream --lib report
```

**测试结果**: ✅ 36/36 通过

## 📚 文档

- [实施总结](./REPORT_SYSTEM_IMPLEMENTATION.md) - 完整的实施文档
- [API使用指南](./REPORT_API_GUIDE.md) - API使用说明和示例
- [Phase 1 计划](../PHASE1_REPORT_APPEAL_IMPLEMENTATION.md) - 原始实施计划

## 🔧 配置参数

| 参数 | 值 | 说明 |
|------|-----|------|
| `MinReportDeposit` | 10 COS | 最小举报押金 |
| `ReportTimeout` | 7 天 | 举报处理超时 |
| `ReportCooldownPeriod` | 1 天 | 举报冷却期 |
| `ReportWithdrawWindow` | 12 小时 | 撤回窗口期 |
| `ContentCommittee` | 1/2 多数 | 内容审核委员会 |

## 📝 变更日志

### v1.0.0 (2026-01-19)

**新增**:
- 举报直播间功能
- 举报撤回机制
- 委员会审核流程
- 封禁与申诉系统
- 防滥用保护机制
- 完整单元测试

**技术细节**:
- 6个新函数
- 4个存储结构
- 9个事件
- 11个错误类型
- 13个单元测试

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

与主项目相同

---

**实施日期**: 2026-01-19
**状态**: ✅ 已完成并测试通过
**维护者**: Cosmos Team
