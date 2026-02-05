# Entity-Shop 分离架构 - 集成测试场景

## 概述

本文档描述 Entity-Shop 分离架构的集成测试场景，用于验证新架构的正确性和完整性。

---

## 测试场景 1: Entity + Primary Shop 创建

### 场景描述
用户创建 Entity 时，系统自动创建 Primary Shop。

### 测试步骤
```rust
// 1. 用户调用 create_shop (原 Entity 创建入口)
EntityRegistry::create_shop(
    origin,
    name: "My Shop",
    logo_cid: None,
    description_cid: None,
);

// 2. 验证 Entity 创建
assert!(EntityRegistry::entities(1).is_some());
let entity = EntityRegistry::entities(1).unwrap();
assert_eq!(entity.primary_shop_id, 1);
assert!(entity.shop_ids.contains(&1));

// 3. 验证 Primary Shop 创建 (通过 ShopProvider)
assert!(EntityShop::shops(1).is_some());
let shop = EntityShop::shops(1).unwrap();
assert_eq!(shop.entity_id, 1);
assert!(shop.is_primary);
```

### 验收标准
- [ ] Entity 正确创建，包含 `shop_ids` 和 `primary_shop_id`
- [ ] Primary Shop 自动创建，`is_primary = true`
- [ ] Entity-Shop 双向关联正确

---

## 测试场景 2: 创建多个 Shop

### 场景描述
Entity 所有者为同一 Entity 创建多个 Shop。

### 测试步骤
```rust
// 1. 已有 Entity 1 和 Primary Shop 1

// 2. 创建第二个 Shop
EntityShop::create_shop(
    origin,
    entity_id: 1,
    name: "Branch Store",
    shop_type: ShopType::PhysicalStore,
    member_mode: MemberMode::Independent,
    initial_fund: 1000,
);

// 3. 验证
assert!(EntityShop::shops(2).is_some());
assert_eq!(EntityShop::shop_entity(2), Some(1));

let entity = EntityRegistry::entities(1).unwrap();
assert!(entity.shop_ids.contains(&2));
```

### 验收标准
- [ ] 新 Shop 正确创建
- [ ] Entity 的 `shop_ids` 更新
- [ ] `ShopEntity` 反向索引正确

---

## 测试场景 3: MemberMode::Inherit

### 场景描述
Shop 使用继承模式，会员数据存储在 Entity 级别。

### 测试步骤
```rust
// 1. 创建使用 Inherit 模式的 Shop
let shop_id = 1; // MemberMode::Inherit

// 2. 注册会员
EntityMember::register_member(origin, shop_id, None);

// 3. 验证会员存储在 Entity 级别
let entity_id = EntityShop::shop_entity(shop_id).unwrap();
assert!(EntityMember::entity_members(entity_id, &account).is_some());
```

### 验收标准
- [ ] 会员数据正确存储在 Entity 级别
- [ ] 该会员在所有 Inherit 模式的 Shop 可见

---

## 测试场景 4: MemberMode::Independent

### 场景描述
Shop 使用独立模式，会员数据存储在 Shop 级别。

### 测试步骤
```rust
// 1. 创建使用 Independent 模式的 Shop
EntityShop::create_shop(
    origin,
    entity_id: 1,
    name: "Independent Branch",
    shop_type: ShopType::OnlineStore,
    member_mode: MemberMode::Independent,
    initial_fund: 1000,
);
let shop_id = 2;

// 2. 注册会员
EntityMember::register_member(origin, shop_id, None);

// 3. 验证会员存储在 Shop 级别
assert!(EntityMember::shop_members(shop_id, &account).is_some());
```

### 验收标准
- [ ] 会员数据正确存储在 Shop 级别
- [ ] 该会员仅在该 Shop 可见

---

## 测试场景 5: Shop 暂停/关闭

### 场景描述
暂停或关闭 Shop 对业务的影响。

### 测试步骤
```rust
// 1. 暂停 Shop
EntityShop::pause_shop(origin, shop_id);

// 2. 验证 Shop 状态
let shop = EntityShop::shops(shop_id).unwrap();
assert_eq!(shop.status, ShopOperatingStatus::Paused);

// 3. 验证业务影响
assert!(!EntityShop::is_shop_active(shop_id));
// 商品上架应失败
assert_noop!(
    EntityService::create_product(origin, shop_id, ...),
    Error::ShopNotActive
);

// 4. 恢复 Shop
EntityShop::resume_shop(origin, shop_id);
assert!(EntityShop::is_shop_active(shop_id));
```

### 验收标准
- [ ] Shop 暂停后，新订单/商品操作被阻止
- [ ] Shop 恢复后，业务正常
- [ ] Primary Shop 不能关闭

---

## 测试场景 6: 跨 Shop 返佣

### 场景描述
用户在 Shop B 消费，返佣给在 Shop A 注册的推荐人。

### 前提条件
- Entity 1 有两个 Shop: A (Inherit) 和 B (Inherit)
- 用户 Alice 在 Shop A 注册，推荐人是 Bob

### 测试步骤
```rust
// 1. Alice 在 Shop B 下单
let order_id = EntityTransaction::place_order(
    alice_origin,
    shop_id: shop_b,
    product_id,
    quantity,
);

// 2. 订单完成后验证返佣
// Bob 应收到返佣（因为 Inherit 模式共享会员体系）
let bob_commission = EntityMember::entity_members(entity_id, &bob)
    .unwrap()
    .pending_commission;
assert!(bob_commission > 0);
```

### 验收标准
- [ ] 跨 Shop 返佣计算正确
- [ ] 返佣记录关联正确的 Shop

---

## 测试场景 7: Shop 积分系统

### 场景描述
Shop 独立积分系统测试。

### 测试步骤
```rust
// 1. 启用 Shop 积分
EntityShop::enable_points(
    origin,
    shop_id,
    name: "Loyalty Points",
    symbol: "LP",
    reward_rate: 100,    // 1%
    exchange_rate: 1000, // 1000 积分 = 1 USDT
    transferable: true,
);

// 2. 用户消费获得积分
// (订单完成后自动发放)

// 3. 验证积分余额
let points = EntityShop::shop_points_balance(shop_id, &user);
assert!(points > 0);

// 4. 积分兑换（如果可转让）
EntityShop::transfer_points(origin, shop_id, &recipient, amount);
```

### 验收标准
- [ ] 积分配置正确保存
- [ ] 消费后积分正确发放
- [ ] 积分转账功能正常（如果启用）

---

## 测试场景 8: Entity 统计汇总

### 场景描述
Entity 级别统计从多个 Shop 汇总。

### 测试步骤
```rust
// 1. Entity 有多个 Shop
// Shop 1: total_sales = 1000, total_orders = 10
// Shop 2: total_sales = 2000, total_orders = 20

// 2. 验证 Entity 汇总统计
let entity = EntityRegistry::entities(entity_id).unwrap();
assert_eq!(entity.total_sales, 3000);
assert_eq!(entity.total_orders, 30);
```

### 验收标准
- [ ] Entity 销售额 = 所有 Shop 销售额之和
- [ ] Entity 订单数 = 所有 Shop 订单数之和

---

## 运行测试

### 单元测试
```bash
cargo test -p pallet-entity-shop
cargo test -p pallet-entity-registry
```

### 集成测试
```bash
# 运行 E2E 测试（需要启动本地节点）
./scripts/run_integration_tests.sh
```

---

## 待完成任务

1. **业务模块 ShopProvider 适配**
   - [ ] service: 使用 `ShopProvider::shop_owner` 替代 `EntityProvider::shop_owner`
   - [ ] transaction: 同上
   - [ ] review: 同上
   - [ ] market: 同上

2. **测试覆盖率目标**
   - [ ] pallet-entity-shop: >80%
   - [ ] pallet-entity-registry 更新部分: >80%

3. **Runtime 集成**
   - [ ] 完成所有业务模块的 ShopProvider 适配
   - [ ] Runtime 编译通过
   - [ ] 本地节点启动测试
