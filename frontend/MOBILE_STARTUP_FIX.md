# React Native 启动问题修复

## 问题描述

应用在 Android 模拟器中一直显示加载转圈，无法正常启动。

## 根本原因

代码中使用了 `@polkadot/extension-dapp` 包，这是**浏览器扩展 API**，在 React Native 环境中无法使用，导致：

1. 模块加载时抛出异常
2. 应用卡在启动画面
3. Metro bundler 无法连接

### 问题代码位置

```typescript
// frontend/src/lib/api.ts (旧版本)
import { web3Enable, web3Accounts, web3FromAddress } from '@polkadot/extension-dapp';
```

## 解决方案

### 1. 创建平台特定的 API 实现

- `api.web.ts` - Web 平台（使用浏览器扩展）
- `api.native.ts` - React Native 平台（不使用浏览器扩展）
- `api.ts` - 统一入口，根据平台自动选择

### 2. 文件结构

```
frontend/src/lib/
├── api.ts           # 统一入口（根据 Platform.OS 选择）
├── api.web.ts       # Web 版本（浏览器扩展）
├── api.native.ts    # React Native 版本（无扩展依赖）
└── signer.native.ts # React Native 签名器
```

### 3. 关键修改

#### api.native.ts
```typescript
// 移除浏览器扩展依赖
// import { web3Enable, web3Accounts, web3FromAddress } from '@polkadot/extension-dapp';

// 只保留 API 连接功能
import { ApiPromise, WsProvider } from '@polkadot/api';
```

#### api.ts (统一入口)
```typescript
import { Platform } from 'react-native';

if (Platform.OS === 'web') {
  module.exports = require('./api.native'); // 暂时都用 native 版本
} else {
  module.exports = require('./api.native');
}
```

## 账户管理方案

### React Native
- 使用 `signer.native.ts` 管理密钥对
- 通过 `wallet.store.ts` 管理账户状态
- 密钥存储在 `expo-secure-store` 中

### Web (未来)
- 使用 `@polkadot/extension-dapp` 连接浏览器扩展
- 通过 Polkadot.js 扩展管理账户

## 验证修复

### 1. 清除缓存
```bash
cd frontend
rm -rf .expo node_modules/.cache
```

### 2. 重新启动
```bash
npx expo start --clear --android
```

### 3. 检查日志
应该能看到：
- Metro bundler 开始编译
- 应用成功连接
- 不再有模块加载错误

## 相关文件

- `frontend/src/lib/api.ts` - API 统一入口
- `frontend/src/lib/api.native.ts` - React Native 实现
- `frontend/src/lib/api.web.ts` - Web 实现（原 api.ts）
- `frontend/src/lib/signer.native.ts` - React Native 签名器
- `frontend/src/stores/wallet.store.ts` - 钱包状态管理

## 注意事项

1. **不要在 React Native 代码中导入浏览器扩展相关的包**
   - `@polkadot/extension-dapp`
   - `@polkadot/extension-inject`
   
2. **使用平台特定的实现**
   - React Native: `signer.native.ts`
   - Web: `@polkadot/extension-dapp`

3. **账户管理统一通过 wallet store**
   - 不要直接调用 `getCurrentAccount()` 等扩展 API
   - 使用 `useWalletStore()` 获取当前账户

## 测试清单

- [ ] 应用能正常启动
- [ ] 不再显示加载转圈
- [ ] Metro bundler 正常编译
- [ ] 钱包功能正常（创建/解锁）
- [ ] 链上交易正常（签名/发送）

## 参考

- [Polkadot.js API 文档](https://polkadot.js.org/docs/api/)
- [React Native Platform 模块](https://reactnative.dev/docs/platform)
- [Expo Secure Store](https://docs.expo.dev/versions/latest/sdk/securestore/)
