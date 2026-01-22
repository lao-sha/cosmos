# 直播间举报系统 API 使用指南

## 快速开始

### 1. 举报直播间

```rust
// 用户举报违规直播间
Livestream::report_room(
    origin,
    room_id: 0,                                    // 直播间ID
    report_type: RoomReportType::IllegalContent,   // 举报类型
    evidence_cid: b"QmXXXXX".to_vec(),            // IPFS证据CID
    description: b"违规内容描述".to_vec(),          // 举报描述
    is_anonymous: false,                           // 是否匿名
)
```

**要求**:
- 押金: 10 DUST
- 不能举报自己的直播间
- 冷却期: 1天

### 2. 撤回举报

```rust
// 在12小时内撤回举报
Livestream::withdraw_room_report(
    origin,
    report_id: 0,  // 举报ID
)
```

**限制**:
- 只能在12小时内撤回
- 退还80%押金，扣除20%
- 只能撤回待审核状态的举报

### 3. 审核举报（委员会）

```rust
// 内容委员会审核举报
Livestream::resolve_room_report(
    origin,                                  // ContentCommittee权限
    report_id: 0,                           // 举报ID
    result: ReportStatus::Upheld,          // 审核结果
    resolution_note: Some(b"处理说明".to_vec()),
)
```

**审核结果**:
- `Upheld`: 举报成立 → 封禁直播间 + 退还押金
- `Rejected`: 举报驳回 → 退还押金
- `Malicious`: 恶意举报 → 没收押金

### 4. 处理过期举报

```rust
// 任何人都可以调用
Livestream::expire_room_report(
    origin,
    report_id: 0,
)
```

**条件**:
- 举报超过7天未处理
- 全额退还押金

### 5. 申诉封禁

```rust
// 被封禁的主播申诉
Livestream::appeal_room_ban(
    origin,                                      // 主播签名
    room_id: 0,                                 // 直播间ID
    appeal_evidence_cid: b"QmYYYYY".to_vec(),  // 申诉证据
    appeal_reason: b"申诉理由".to_vec(),         // 申诉理由
)
```

**要求**:
- 直播间必须处于封禁状态
- 只有主播本人可以申诉
- 每个封禁只能申诉一次

### 6. 处理申诉（治理）

```rust
// 治理委员会处理申诉
Livestream::resolve_room_ban_appeal(
    origin,                           // GovernanceOrigin权限
    room_id: 0,                      // 直播间ID
    result: AppealResult::Upheld,   // 申诉结果
)
```

**申诉结果**:
- `Upheld`: 申诉成立 → 解除封禁
- `Rejected`: 申诉驳回 → 维持封禁

## 📋 举报类型

```rust
pub enum RoomReportType {
    IllegalContent,      // 违规内容（涉黄、暴力等）
    FalseAdvertising,    // 虚假宣传
    Harassment,          // 骚扰观众
    Fraud,              // 诈骗行为
    Other,              // 其他
}
```

## 📊 举报状态

```rust
pub enum ReportStatus {
    Pending,      // 待审核
    UnderReview,  // 审核中
    Upheld,       // 举报成立
    Rejected,     // 举报驳回
    Malicious,    // 恶意举报
    Withdrawn,    // 已撤回
    Expired,      // 已过期
}
```

## 🔄 完整流程示例

### 场景1: 举报成立并封禁

```rust
// 1. BOB 举报 ALICE 的直播间
Livestream::report_room(
    RuntimeOrigin::signed(BOB),
    0,
    RoomReportType::IllegalContent,
    b"QmEvidence".to_vec(),
    b"违规内容".to_vec(),
    false,
);
// 押金被锁定: 10 DUST

// 2. 委员会审核：举报成立
Livestream::resolve_room_report(
    RuntimeOrigin::root(),
    0,
    ReportStatus::Upheld,
    Some(b"确认违规".to_vec()),
);
// 结果:
// - BOB 押金退还
// - ALICE 直播间被封禁
// - 创建封禁记录
```

### 场景2: 申诉成功解封

```rust
// 1. ALICE 申诉封禁
Livestream::appeal_room_ban(
    RuntimeOrigin::signed(ALICE),
    0,
    b"QmAppealEvidence".to_vec(),
    b"这是误判".to_vec(),
);

// 2. 治理审核：申诉成功
Livestream::resolve_room_ban_appeal(
    RuntimeOrigin::root(),
    0,
    AppealResult::Upheld,
);
// 结果:
// - 直播间解除封禁
// - 状态恢复为 Ended
// - 删除封禁记录
```

### 场景3: 撤回举报

```rust
// 1. BOB 举报
Livestream::report_room(
    RuntimeOrigin::signed(BOB),
    0,
    RoomReportType::Harassment,
    b"QmEvidence".to_vec(),
    b"骚扰观众".to_vec(),
    false,
);

// 2. BOB 在12小时内撤回
Livestream::withdraw_room_report(
    RuntimeOrigin::signed(BOB),
    0,
);
// 结果:
// - 退还 80% 押金 (8 DUST)
// - 没收 20% 押金 (2 DUST)
```

## 🎯 最佳实践

### 举报者

1. **收集充分证据**: 使用IPFS存储截图、录屏等证据
2. **详细描述**: 清楚说明违规行为
3. **谨慎举报**: 恶意举报会被没收押金
4. **及时撤回**: 发现错误在12小时内撤回

### 主播

1. **遵守规则**: 避免违规内容
2. **保存证据**: 保留直播录像以备申诉
3. **及时申诉**: 被误封后立即申诉
4. **配合审核**: 提供完整的申诉材料

### 委员会

1. **公正审核**: 基于证据做出判断
2. **及时处理**: 避免举报超时
3. **详细说明**: 提供清晰的处理理由
4. **一致标准**: 保持审核标准统一

## ⚠️ 注意事项

### 举报限制

- ❌ 不能举报自己的直播间
- ❌ 冷却期内不能重复举报同一直播间
- ❌ 余额不足10 DUST无法举报

### 撤回限制

- ❌ 超过12小时无法撤回
- ❌ 已处理的举报无法撤回
- ❌ 只有举报者本人可以撤回

### 申诉限制

- ❌ 只有被封禁的直播间可以申诉
- ❌ 只有主播本人可以申诉
- ❌ 每个封禁只能申诉一次

## 📞 查询接口

### 查询举报记录

```rust
// 通过 report_id 查询
let report = Livestream::room_reports(report_id);
```

### 查询封禁记录

```rust
// 通过 room_id 查询
let ban_record = Livestream::room_ban_records(room_id);
```

### 查询冷却期

```rust
// 检查用户对某直播间的冷却期
let last_report_time = Livestream::room_report_cooldown(reporter, room_id);
```

## 🔔 事件监听

### 举报相关事件

```rust
// 直播间被举报
RoomReported { report_id, reporter, room_id, report_type }

// 举报已撤回
RoomReportWithdrawn { report_id, reporter, refund_amount }

// 举报成立
RoomReportUpheld { report_id, room_id }

// 举报驳回
RoomReportRejected { report_id }

// 恶意举报
MaliciousRoomReport { report_id, reporter }

// 举报已过期
RoomReportExpired { report_id }
```

### 申诉相关事件

```rust
// 封禁申诉提交
RoomBanAppealed { room_id, host }

// 申诉成功
RoomBanAppealUpheld { room_id }

// 申诉驳回
RoomBanAppealRejected { room_id }
```

## 💡 使用示例

### 前端集成示例

```typescript
// 举报直播间
async function reportRoom(
  roomId: number,
  reportType: RoomReportType,
  evidenceCid: string,
  description: string,
  isAnonymous: boolean
) {
  const tx = api.tx.livestream.reportRoom(
    roomId,
    reportType,
    evidenceCid,
    description,
    isAnonymous
  );

  await tx.signAndSend(signer, ({ status, events }) => {
    if (status.isInBlock) {
      console.log('举报已提交');
      // 监听 RoomReported 事件
    }
  });
}

// 撤回举报
async function withdrawReport(reportId: number) {
  const tx = api.tx.livestream.withdrawRoomReport(reportId);
  await tx.signAndSend(signer);
}

// 申诉封禁
async function appealBan(
  roomId: number,
  evidenceCid: string,
  reason: string
) {
  const tx = api.tx.livestream.appealRoomBan(
    roomId,
    evidenceCid,
    reason
  );
  await tx.signAndSend(signer);
}
```

## 📚 相关文档

- [实施总结](./REPORT_SYSTEM_IMPLEMENTATION.md)
- [Phase 1 实施计划](../PHASE1_REPORT_APPEAL_IMPLEMENTATION.md)
- [举报入口点文档](../REPORT_APPEAL_ENTRY_POINTS.md)

---

**文档版本**: v1.0.0
**最后更新**: 2026-01-19
**维护者**: Stardust Team
