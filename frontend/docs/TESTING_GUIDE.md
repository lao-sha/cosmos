# 购买 DUST 功能 - 完整测试指南

## 🎯 测试目标

验证购买 DUST 功能的完整流程，包括：
- API 连接
- 做市商查询
- 订单创建
- 交易签名
- 订单状态更新

---

## 📋 测试前准备

### 1. 启动区块链节点

```bash
cd ../node
cargo build --release
./target/release/stardust-node --dev --tmp
```

**验证节点启动**:
- 看到 "Idle" 日志
- 端口 9944 监听
- 区块正常生成

### 2. 配置环境变量

创建 `frontend/.env`:

```bash
EXPO_PUBLIC_WS_PROVIDER=ws://localhost:9944
```

### 3. 安装 Polkadot.js 扩展 (Web 测试)

1. 访问 https://polkadot.js.org/extension/
2. 安装浏览器扩展
3. 创建或导入账户
4. 确保账户有足够的 DUST 余额

### 4. 启动前端

```bash
cd frontend
npm install
npm start
```

选择 Web 模式进行测试。

---

## 🧪 测试用例

### 测试用例 1: API 初始化

**步骤**:
1. 打开应用
2. 观察控制台日志

**预期结果**:
```
[API] Initializing Polkadot API...
[API] Provider URL: ws://localhost:9944
[API] Connected to chain: Development
[API] Node version: x.x.x
[Account] Enabling extension...
[Account] Extensions enabled: 1
[Account] Accounts found: 1
```

**验证**:
- ✅ 无错误日志
- ✅ API 连接成功
- ✅ 扩展启用成功
- ✅ 账户加载成功

---

### 测试用例 2: 查询做市商列表

**步骤**:
1. 导航到 `/wallet/buy-dust`
2. 观察做市商列表

**预期结果**:
- 显示做市商列表
- 每个做市商显示：
  - 姓名（脱敏）
  - 评分
  - 溢价
  - 在线状态
  - 微信号

**验证**:
- ✅ 列表加载成功
- ✅ 数据显示正确
- ✅ 无加载错误

**调试**:
```javascript
// 在浏览器控制台
console.log('[Debug] Makers:', useTradingStore.getState().makers);
```

---

### 测试用例 3: 创建首购订单

**步骤**:
1. 在购买首页点击"立即首购"
2. 选择一个做市商
3. 点击"创建首购订单"
4. 填写支付信息表单：
   - 真实姓名: 张三
   - 身份证号: 110101199001011234
   - 手机号: 13812345678
   - 微信号: wechat_test
5. 点击"确认"
6. Polkadot.js 扩展弹出签名请求
7. 点击"Sign the transaction"
8. 等待交易确认

**预期结果**:
```
[Trading] Transaction included in block: 0x...
[Trading] First purchase created, order ID: 1
```

**验证**:
- ✅ 表单验证正确
- ✅ 签名请求弹出
- ✅ 交易成功上链
- ✅ 获取订单 ID
- ✅ 跳转到订单详情页

**可能的错误**:
1. "No account selected"
   - 确保 Polkadot.js 扩展已授权
   - 检查账户是否选中

2. "InsufficientBalance"
   - 账户余额不足
   - 使用开发链的 Alice/Bob 账户

3. "FirstPurchaseAlreadyCompleted"
   - 该账户已完成首购
   - 使用新账户测试

---

### 测试用例 4: 查看订单详情

**步骤**:
1. 创建订单后自动跳转
2. 或手动访问 `/wallet/buy-dust/[orderId]`

**预期结果**:
- 显示订单信息
- 显示倒计时
- 显示收款地址
- 可以复制地址
- 可以标记已付款
- 可以取消订单

**验证**:
- ✅ 订单信息正确
- ✅ 倒计时正常运行
- ✅ 地址可以复制
- ✅ 按钮可点击

---

### 测试用例 5: 标记已付款

**步骤**:
1. 在订单详情页
2. 点击"我已付款"
3. 确认对话框
4. 签名交易
5. 等待确认

**预期结果**:
```
[Trading] Mark paid transaction in block
```

**验证**:
- ✅ 交易成功
- ✅ 自动跳转到等待页面
- ✅ 订单状态更新为 "Paid"

---

### 测试用例 6: 订单状态订阅

**步骤**:
1. 在等待页面
2. 使用做市商账户调用 `releaseDust`
3. 观察页面自动更新

**预期结果**:
- 页面自动跳转到完成页面
- 显示获得的 DUST 数量

**验证**:
- ✅ 订阅正常工作
- ✅ 状态自动更新
- ✅ 页面自动跳转

---

### 测试用例 7: 取消订单

**步骤**:
1. 创建新订单
2. 在订单详情页点击"取消订单"
3. 确认对话框
4. 签名交易

**预期结果**:
```
[Trading] Cancel order transaction in block
```

**验证**:
- ✅ 交易成功
- ✅ 返回上一页
- ✅ 订单状态更新为 "Cancelled"

---

## 🐛 常见问题排查

### 问题 1: API 连接失败

**症状**:
```
Error: Failed to connect to blockchain node
```

**解决方案**:
1. 检查节点是否运行: `lsof -i :9944`
2. 检查环境变量: `echo $EXPO_PUBLIC_WS_PROVIDER`
3. 尝试重启节点
4. 检查防火墙设置

---

### 问题 2: 扩展未授权

**症状**:
```
Error: No extension found
```

**解决方案**:
1. 安装 Polkadot.js 扩展
2. 刷新页面
3. 点击扩展图标授权应用
4. 确保至少有一个账户

---

### 问题 3: 交易失败

**症状**:
```
Error: otcOrder.FirstPurchaseAlreadyCompleted
```

**解决方案**:
1. 查看错误信息
2. 根据错误类型处理：
   - `FirstPurchaseAlreadyCompleted`: 使用新账户
   - `InsufficientBalance`: 充值账户
   - `MakerNotActive`: 选择其他做市商
   - `OrderExpired`: 创建新订单

---

### 问题 4: 订单状态不更新

**症状**:
- 标记已付款后页面不跳转
- 订单状态不变化

**解决方案**:
1. 检查订阅是否正常:
   ```javascript
   console.log('[Debug] Subscription active:', !!unsubscribe);
   ```
2. 手动刷新页面
3. 检查网络连接
4. 查看控制台错误

---

## 📊 测试检查清单

### 基础功能
- [ ] API 初始化成功
- [ ] 扩展连接成功
- [ ] 账户加载成功
- [ ] 做市商列表加载

### 首购流程
- [ ] 显示首购特惠
- [ ] 选择做市商
- [ ] 填写支付信息
- [ ] 表单验证正确
- [ ] 创建订单成功
- [ ] 获取订单 ID

### 订单操作
- [ ] 查看订单详情
- [ ] 复制收款地址
- [ ] 标记已付款
- [ ] 取消订单
- [ ] 订单状态订阅

### 页面跳转
- [ ] 创建后跳转详情
- [ ] 付款后跳转等待
- [ ] 完成后跳转完成页
- [ ] 取消后返回上一页

### 错误处理
- [ ] 网络错误提示
- [ ] 交易失败提示
- [ ] 表单验证提示
- [ ] 余额不足提示

---

## 🎬 测试视频录制

建议录制以下场景:

1. **完整首购流程** (2-3 分钟)
   - 从首页到完成页
   - 包含所有交互

2. **错误处理** (1-2 分钟)
   - 表单验证错误
   - 交易失败
   - 网络断开

3. **订单状态更新** (1 分钟)
   - 实时订阅演示
   - 自动页面跳转

---

## 📝 测试报告模板

```markdown
# 测试报告

**测试日期**: 2026-01-10
**测试人员**: [姓名]
**测试环境**:
- 节点版本: x.x.x
- 前端版本: x.x.x
- 浏览器: Chrome x.x.x

## 测试结果

### 通过的测试
- [x] API 初始化
- [x] 做市商查询
- [x] 创建首购订单
- ...

### 失败的测试
- [ ] 订单状态订阅 - 原因: ...

### 发现的问题
1. 问题描述
   - 重现步骤
   - 预期结果
   - 实际结果
   - 截图/日志

## 建议
- 改进建议 1
- 改进建议 2
```

---

## 🚀 下一步

测试通过后:

1. ✅ 修复发现的问题
2. ✅ 优化用户体验
3. ✅ 添加更多错误提示
4. ✅ 性能优化
5. ✅ 准备上线

---

**祝测试顺利！** 🎉
