# 前端启动转圈问题诊断

## 问题描述
应用启动后一直显示加载动画（转圈），无法进入主界面。

## 可能的原因

### 1. WalletAuthGate 初始化卡住
`WalletAuthGate` 组件在 `_layout.tsx` 中包裹了整个应用，如果初始化失败或超时，会一直显示加载动画。

### 2. SecureStore 访问问题
在某些设备或模拟器上，`expo-secure-store` 可能无法正常工作。

### 3. 路由导航问题
初始化完成后，路由跳转可能失败。

## 已实施的修复

### 1. 添加超时保护
在 `WalletAuthGate.tsx` 中添加了 10 秒超时检测：
```typescript
const timeout = setTimeout(() => {
  console.error('[WalletAuthGate] Initialization timeout!');
  setInitError('初始化超时，请重启应用');
}, 10000);
```

### 2. 添加错误显示
如果初始化失败，会显示错误信息而不是一直转圈：
```typescript
if (initError) {
  return (
    <View style={styles.loadingContainer}>
      <Text style={styles.errorText}>⚠️ {initError}</Text>
      <Text style={styles.errorHint}>请检查控制台日志</Text>
    </View>
  );
}
```

### 3. 优化初始化流程
将账户列表加载改为异步后台加载，不阻塞主初始化流程：
```typescript
// 延迟加载账户列表，不阻塞初始化
if (hasWallet) {
  setTimeout(() => {
    get().loadAllAccounts().catch(err => {
      console.error('[Wallet] Failed to load accounts:', err);
    });
  }, 100);
}
```

## 调试步骤

### 1. 查看 Metro 日志
```bash
cd frontend
npx expo start --clear
```

查看控制台输出，特别是：
- `[WalletAuthGate] Starting initialization...`
- `[Wallet] Starting initialization...`
- `[Wallet] Initialized successfully`

### 2. 使用调试页面
访问 `exp://192.168.1.28:8082/debug` 查看详细状态。

或在应用中手动导航：
```typescript
import { useRouter } from 'expo-router';
const router = useRouter();
router.push('/debug');
```

### 3. 检查设备日志

#### Android
```bash
adb logcat | grep -i "expo\|react"
```

#### iOS
在 Xcode 中查看设备日志。

### 4. 清除应用数据
如果是 SecureStore 数据损坏：

#### Android
```bash
adb shell pm clear host.exp.exponent  # Expo Go
# 或
adb shell pm clear com.yourapp.package  # 开发构建
```

#### iOS
在设备上长按应用图标 → 删除应用 → 重新安装

## 常见问题

### Q: 一直显示"正在初始化..."
**A:** 检查 Metro 日志，看是否有错误。可能是：
- SecureStore 权限问题
- 加密库初始化失败
- 路由配置错误

### Q: 显示"初始化超时"
**A:** 说明初始化过程超过 10 秒。可能原因：
- 设备性能问题
- SecureStore 响应慢
- 网络请求卡住（检查是否有不必要的网络调用）

### Q: 显示错误信息
**A:** 根据错误信息排查：
- "初始化失败: ..." - 查看具体错误原因
- 检查 `wallet.store.ts` 中的 `initialize()` 方法

## 快速修复建议

### 方案 1: 跳过 WalletAuthGate（临时）
在 `app/_layout.tsx` 中临时移除 `WalletAuthGate`：

```typescript
export default function RootLayout() {
  return (
    <View style={styles.container}>
      <StatusBar style="light" />
      {/* 临时注释掉 */}
      {/* <WalletAuthGate> */}
        <View style={styles.content}>
          <Slot />
        </View>
      {/* </WalletAuthGate> */}
    </View>
  );
}
```

### 方案 2: 使用简化的初始化
创建一个最小化的 `WalletAuthGate`：

```typescript
export function WalletAuthGate({ children }: WalletAuthGateProps) {
  const [ready, setReady] = React.useState(false);

  useEffect(() => {
    // 最简单的初始化
    setTimeout(() => setReady(true), 1000);
  }, []);

  if (!ready) {
    return <Text>Loading...</Text>;
  }

  return <>{children}</>;
}
```

### 方案 3: 检查 expo-secure-store
测试 SecureStore 是否正常工作：

```typescript
import * as SecureStore from 'expo-secure-store';

async function testSecureStore() {
  try {
    await SecureStore.setItemAsync('test', 'value');
    const value = await SecureStore.getItemAsync('test');
    console.log('SecureStore works:', value === 'value');
    await SecureStore.deleteItemAsync('test');
  } catch (error) {
    console.error('SecureStore error:', error);
  }
}
```

## 下一步

1. 重新启动应用，查看是否显示错误信息
2. 如果仍然转圈，访问 `/debug` 页面查看详细状态
3. 检查 Metro 控制台日志
4. 如果问题持续，尝试快速修复方案 1 或 2

## 相关文件

- `frontend/app/_layout.tsx` - 根布局
- `frontend/src/features/wallet/WalletAuthGate.tsx` - 认证门卫
- `frontend/src/stores/wallet.store.ts` - 钱包状态
- `frontend/src/lib/keystore.native.ts` - 密钥存储
- `frontend/app/debug.tsx` - 调试页面
