# 购买 DUST 功能 - 快速启动指南

## 🚀 立即开始

### 1. 查看已创建的文件

```bash
# 状态管理
src/stores/trading.store.ts

# 服务层
src/services/trading.service.ts

# 组件
src/features/trading/components/
├── MakerCard.tsx
├── PriceDisplay.tsx
├── AmountInput.tsx
├── CountdownTimer.tsx
└── index.ts

# 页面
app/wallet/buy-dust/
├── index.tsx                    # 购买首页
├── first-purchase.tsx           # 首购页面
├── order.tsx                    # 创建订单
└── [orderId]/
    ├── index.tsx                # 订单详情
    ├── waiting.tsx              # 等待放币
    └── complete.tsx             # 交易完成
```

### 2. 访问页面

启动开发服务器后，访问:

```
http://localhost:8081/wallet/buy-dust
```

### 3. 测试流程

#### 首购流程:
1. 访问 `/wallet/buy-dust`
2. 点击"立即首购"或选择做市商
3. 在首购页面确认信息
4. 点击"创建首购订单"
5. 查看订单详情，复制收款地址
6. 点击"我已付款"
7. 等待做市商确认
8. 查看完成页面

#### 普通订单流程:
1. 访问 `/wallet/buy-dust`
2. 选择做市商
3. 输入购买金额 (20-200 USD)
4. 点击"创建订单"
5. 后续流程同首购

---

## 📦 依赖安装

如果缺少依赖，请安装:

```bash
npm install crypto-js
npm install @types/crypto-js --save-dev
```

---

## 🔧 下一步开发任务

### 高优先级 (本周完成)

#### 1. 集成 Polkadot.js API

在 `src/services/trading.service.ts` 中:

```typescript
// 初始化 API
import { ApiPromise, WsProvider } from '@polkadot/api';

const provider = new WsProvider('ws://localhost:9944');
const api = await ApiPromise.create({ provider });

tradingService.initialize(api);
```

#### 2. 创建支付信息输入表单

创建 `src/features/trading/components/PaymentForm.tsx`:

```typescript
interface PaymentFormProps {
  onSubmit: (data: PaymentData) => void;
}

interface PaymentData {
  realName: string;
  idCard: string;
  phone: string;
  wechatId: string;
}
```

#### 3. 在订单创建时收集用户信息

修改 `first-purchase.tsx` 和 `order.tsx`:

```typescript
const [paymentData, setPaymentData] = useState<PaymentData | null>(null);

// 显示表单
<PaymentForm onSubmit={setPaymentData} />

// 使用真实数据创建订单
const paymentCommit = TradingService.generatePaymentCommit(
  paymentData.realName,
  paymentData.idCard,
  paymentData.phone
);
```

---

## 🎨 UI 调整建议

### 1. 添加加载状态

所有异步操作都应显示加载状态:

```typescript
{loadingMakers ? (
  <ActivityIndicator size="large" color="#B2955D" />
) : (
  // 内容
)}
```

### 2. 添加错误提示

使用 Alert 或 Toast 显示错误:

```typescript
if (makerError) {
  Alert.alert('错误', makerError);
}
```

### 3. 添加空状态

当没有数据时显示友好提示:

```typescript
{makers.length === 0 && (
  <View style={styles.empty}>
    <Text>暂无可用做市商</Text>
  </View>
)}
```

---

## 🐛 调试技巧

### 1. 查看状态

在组件中添加:

```typescript
useEffect(() => {
  console.log('[Debug] Current state:', {
    makers,
    selectedMaker,
    currentOrder,
  });
}, [makers, selectedMaker, currentOrder]);
```

### 2. 模拟数据

在开发阶段，store 中已经提供了模拟数据，可以直接测试 UI。

### 3. 测试订单流程

手动修改订单状态来测试不同页面:

```typescript
// 在 store 中临时修改
set({
  currentOrder: {
    ...currentOrder,
    state: OrderState.Paid, // 测试等待页面
  },
});
```

---

## 📱 移动端适配

### 1. 响应式布局

所有组件都使用 Flexbox，自动适配不同屏幕。

### 2. 触摸优化

使用 `TouchableOpacity` 提供触摸反馈:

```typescript
<TouchableOpacity
  activeOpacity={0.7}
  onPress={handlePress}
>
  {/* 内容 */}
</TouchableOpacity>
```

### 3. 键盘处理

在输入页面使用 `KeyboardAvoidingView`:

```typescript
<KeyboardAvoidingView
  behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
>
  {/* 表单 */}
</KeyboardAvoidingView>
```

---

## 🔐 安全注意事项

### 1. 敏感信息处理

- ✅ 支付信息使用 SHA256 哈希
- ✅ 链上只存储哈希值
- ⚠️ 需要在本地安全存储原始数据

### 2. 交易签名

- ✅ 使用 Polkadot.js 签名
- ✅ 用户确认后才签名
- ⚠️ 需要处理签名失败情况

### 3. 数据验证

- ✅ 前端验证金额范围
- ✅ 验证做市商状态
- ⚠️ 需要验证用户输入格式

---

## 📊 性能优化建议

### 1. 使用 React.memo

对不常变化的组件使用 memo:

```typescript
export const MakerCard = React.memo<MakerCardProps>(({ maker, onPress }) => {
  // ...
});
```

### 2. 使用 useCallback

对回调函数使用 useCallback:

```typescript
const handleSelectMaker = useCallback((makerId: number) => {
  selectMaker(makerId);
}, [selectMaker]);
```

### 3. 分页加载

如果做市商很多，考虑分页:

```typescript
const [page, setPage] = useState(1);
const makersToShow = makers.slice(0, page * 10);
```

---

## 🎯 功能扩展建议

### 1. 订单历史

创建 `/wallet/buy-dust/history` 页面显示历史订单。

### 2. 价格走势图

使用图表库显示 DUST 价格走势。

### 3. 推送通知

订单状态变化时发送推送通知。

### 4. 多语言支持

使用 i18n 支持多语言。

---

## 📞 获取帮助

如有问题，请查看:

1. **设计文档**: `docs/BUY_DUST_DESIGN.md`
2. **完成报告**: `docs/BUY_DUST_FRONTEND_COMPLETE.md`
3. **后端文档**: `../pallets/trading/README.md`

---

**祝开发顺利！** 🎉
