# COSMOS Frontend 开发指南

> 版本：1.0 | 更新：2026-02

---

## 一、技术栈概览

| 类别 | 技术 | 版本 |
|------|------|------|
| 框架 | React Native + Expo | 0.81 / 54.0 |
| 路由 | Expo Router | 6.0 |
| 状态 | Zustand + TanStack Query | 5.0 / 5.90 |
| 区块链 | @polkadot/api | 16.5 |
| 类型 | TypeScript | 5.9 |

---

## 二、后端模块 → 前端页面映射

### 2.1 模块架构

```
pallets/
├── entity/        # 实体管理（店铺、会员、代币、治理）
├── trading/       # 交易系统（OTC、Swap、做市商）
├── chat/          # 聊天系统（私聊、群聊）
├── social/        # 社交模块
├── dispute/       # 争议解决（托管、证据、仲裁）
├── storage/       # 存储服务（IPFS）
├── affiliate/     # 联盟计酬
└── referral/      # 推荐关系
```

### 2.2 Entity 模块组

| Pallet | 功能 | 前端路由 | 优先级 |
|--------|------|----------|--------|
| registry | 实体注册 | `/mall/shop/create` | P0 |
| shop | 店铺管理 | `/mall/shop/[id]` | P0 |
| token | 代币发行 | `/mall/token/` | P1 |
| governance | 提案投票 | `/mall/governance/` | P2 |
| member | 会员管理 | `/membership/` | P0 |
| commission | 返佣系统 | `/referral/commission` | P1 |
| market | P2P 市场 | `/market/` | P1 |
| service | 商品服务 | `/mall/products/` | P0 |
| transaction | 订单管理 | `/mall/orders/` | P0 |
| review | 评价系统 | `/mall/reviews/` | P1 |
| kyc | KYC 认证 | `/settings/kyc` | P0 |
| sale | 代币发售 | `/mall/token/sale` | P2 |

### 2.3 Trading 模块组

| Pallet | 功能 | 前端路由 | 优先级 |
|--------|------|----------|--------|
| maker | 做市商 | `/maker/` | P0 |
| otc | OTC 交易 | `/otc/` | P0 |
| swap | 兑换服务 | `/swap/` | P0 |
| credit | 信用系统 | `/profile/credit` | P1 |

### 2.4 Chat 模块组

| Pallet | 功能 | 前端路由 | 优先级 |
|--------|------|----------|--------|
| core | 私聊 | `/chat/` | P0 |
| group | 群聊 | `/chat/group/` | P1 |
| contacts | 联系人 | `/friends/` | P0 |

### 2.5 Dispute 模块组

| Pallet | 功能 | 前端路由 | 优先级 |
|--------|------|----------|--------|
| escrow | 资金托管 | (内部) | - |
| evidence | 证据提交 | `/disputes/evidence` | P1 |
| arbitration | 仲裁流程 | `/disputes/` | P1 |

---

## 三、前端目录结构

```
frontend/
├── app/                    # 页面（文件路由）
│   ├── (tabs)/             # 底部导航
│   ├── wallet/             # 钱包模块
│   ├── otc/                # OTC 交易
│   ├── swap/               # 兑换
│   ├── maker/              # 做市商
│   ├── chat/               # 聊天
│   ├── mall/               # 商城
│   ├── disputes/           # 争议
│   └── settings/           # 设置
│
├── components/             # 公共组件
├── services/               # API 服务
├── stores/                 # Zustand 状态
├── hooks/                  # 自定义 Hooks
└── lib/                    # 工具库
```

---

## 四、核心开发模式

### 4.1 API 连接

```typescript
// services/api.ts
import { ApiPromise, WsProvider } from '@polkadot/api';

const WS_ENDPOINT = 'wss://rpc.cosmos.network';

export async function getApi(): Promise<ApiPromise> {
  const provider = new WsProvider(WS_ENDPOINT);
  return ApiPromise.create({ provider });
}
```

### 4.2 查询链上数据

```typescript
// hooks/useQuery.ts
import { useQuery } from '@tanstack/react-query';
import { getApi } from '@/services/api';

export function useBalance(address: string) {
  return useQuery({
    queryKey: ['balance', address],
    queryFn: async () => {
      const api = await getApi();
      const { data } = await api.query.system.account(address);
      return data.free.toString();
    },
  });
}
```

### 4.3 发送交易

```typescript
// hooks/useTransaction.ts
export function useTransaction() {
  const { keyring } = useWalletStore();

  async function send(pallet: string, method: string, args: any[]) {
    const api = await getApi();
    const tx = api.tx[pallet][method](...args);
    return tx.signAndSend(keyring);
  }

  return { send };
}
```

### 4.4 订阅链上事件

```typescript
// hooks/useSubscription.ts
export function useBlockSubscription(callback: (block: number) => void) {
  useEffect(() => {
    let unsub: () => void;
    
    getApi().then(api => {
      api.rpc.chain.subscribeNewHeads(header => {
        callback(header.number.toNumber());
      }).then(fn => unsub = fn);
    });

    return () => unsub?.();
  }, []);
}
```

---

## 五、Pallet 调用示例

### 5.1 OTC 交易

```typescript
// 创建买单
api.tx.otcOrder.createBuyOrder(makerId, usdtAmount);

// 确认支付
api.tx.otcOrder.confirmPayment(orderId, txHash);

// 释放 COS
api.tx.otcOrder.releaseCos(orderId);
```

### 5.2 Swap 兑换

```typescript
// 创建兑换请求
api.tx.swap.createSwapRequest(makerId, cosAmount);

// 查询做市商列表
api.query.maker.makers.entries();
```

### 5.3 会员管理

```typescript
// 绑定推荐人
api.tx.entityMember.bindReferrer(entityId, referrerCode);

// 查询会员等级
api.query.entityMember.memberLevels(entityId, accountId);
```

### 5.4 聊天

```typescript
// 发送消息
api.tx.chatCore.sendMessage(chatUserId, targetId, encryptedContent, msgType);

// 查询会话
api.query.chatCore.conversations(chatUserId);
```

### 5.5 争议仲裁

```typescript
// 提交证据
api.tx.evidence.commit(domain, bizId, cid, accessMode);

// 发起仲裁
api.tx.arbitration.disputeWithTwoWayDeposit(domain, bizId, deposit);
```

---

## 六、状态管理

### 6.1 钱包状态 (Zustand)

```typescript
// stores/wallet.ts
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface WalletState {
  address: string | null;
  isConnected: boolean;
  connect: (address: string) => void;
  disconnect: () => void;
}

export const useWalletStore = create<WalletState>()(
  persist(
    (set) => ({
      address: null,
      isConnected: false,
      connect: (address) => set({ address, isConnected: true }),
      disconnect: () => set({ address: null, isConnected: false }),
    }),
    { name: 'wallet-storage' }
  )
);
```

### 6.2 链上数据 (TanStack Query)

```typescript
// 配置 QueryClient
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 10_000,      // 10秒内不重新请求
      gcTime: 5 * 60_000,     // 5分钟缓存
      retry: 2,
    },
  },
});
```

---

## 七、安全规范

### 7.1 私钥管理

```typescript
// 使用 SecureStore 存储私钥
import * as SecureStore from 'expo-secure-store';

await SecureStore.setItemAsync('privateKey', encryptedKey);
const key = await SecureStore.getItemAsync('privateKey');
```

### 7.2 生物识别

```typescript
// 交易前验证
import * as LocalAuth from 'expo-local-authentication';

const result = await LocalAuth.authenticateAsync({
  promptMessage: '确认交易',
  fallbackLabel: '使用密码',
});

if (result.success) {
  // 执行交易
}
```

### 7.3 敏感数据

- 私钥：SecureStore 加密存储
- 助记词：不持久化，用后清除
- 交易签名：本地完成，不传服务器

---

## 八、开发流程

### 8.1 新功能开发

1. 确认对应 Pallet 和 Extrinsic
2. 创建 Service 层封装 API 调用
3. 创建 Hook 处理状态和副作用
4. 实现 UI 组件
5. 集成测试

### 8.2 命令

```bash
# 开发
npx expo start

# 构建
npx expo build:ios
npx expo build:android

# 检查
npm run lint
npm run typecheck
```

---

## 九、优先级路线图

### P0 - 核心功能

- [ ] 钱包（创建/导入/转账）
- [ ] OTC 交易
- [ ] Swap 兑换
- [ ] 做市商面板
- [ ] KYC 认证
- [ ] 聊天基础功能
- [ ] 推荐邀请

### P1 - 增强功能

- [ ] 会员等级
- [ ] 返佣系统
- [ ] 代币市场
- [ ] 群聊
- [ ] 评价系统
- [ ] 信用分

### P2 - 高级功能

- [ ] 治理投票
- [ ] 代币发售
- [ ] 争议仲裁
