# 节点侧 AppCode 配置 - 使用说明

## 概述

已实现节点启动时通过环境变量或命令行参数配置 Almanac AppCode,避免在源代码中硬编码敏感信息。

## 代码改动

### 1. node/src/cli.rs
添加了 `almanac_appcode` 命令行参数:
```rust
#[arg(long, env = "ALMANAC_APPCODE")]
pub almanac_appcode: Option<String>,
```

### 2. node/src/service.rs
在 `new_full()` 函数中:
- 添加 `almanac_appcode: Option<String>` 参数
- 在 OCW 启动时将 AppCode 写入本地存储
- 添加详细的日志输出

### 3. node/src/command.rs
在主节点启动逻辑中传递 `cli.almanac_appcode.clone()`

## 使用方式

### 方式一: 环境变量 (推荐开发环境)

```bash
# 设置环境变量
export ALMANAC_APPCODE="your_aliyun_appcode_here"

# 启动节点
./target/release/stardust-node --dev

# 或者临时设置
ALMANAC_APPCODE="your_appcode" ./target/release/stardust-node --dev
```

### 方式二: 命令行参数

```bash
./target/release/stardust-node \
  --dev \
  --almanac-appcode "your_aliyun_appcode_here"
```

### 方式三: .env 文件 (配合 systemd 或 Docker)

创建 `.env` 文件:
```bash
# .env
ALMANAC_APPCODE=your_aliyun_appcode_here
```

在 systemd service 中使用:
```ini
[Unit]
Description=Stardust Node
After=network.target

[Service]
Type=simple
User=stardust
Group=stardust
WorkingDirectory=/var/lib/stardust
EnvironmentFile=/etc/stardust/.env
ExecStart=/usr/local/bin/stardust-node --chain=production
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### 方式四: Docker 环境

```bash
# Dockerfile 无需改动,运行时注入
docker run -d \
  -e ALMANAC_APPCODE="your_appcode" \
  --name stardust-node \
  stardust-node:latest

# docker-compose.yml
version: '3'
services:
  node:
    image: stardust-node:latest
    environment:
      - ALMANAC_APPCODE=${ALMANAC_APPCODE}
    env_file:
      - .env
```

## 验证配置

### 查看启动日志

成功配置时会看到:
```
✅ Almanac AppCode configured (length: XX bytes)
```

未配置时会看到警告:
```
⚠️ ALMANAC_APPCODE not set, Almanac OCW will not work.
   Set via --almanac-appcode CLI argument or ALMANAC_APPCODE environment variable
```

### 测试完整流程

```bash
# 1. 构建节点
cargo build --release

# 2. 设置 AppCode (使用测试值)
export ALMANAC_APPCODE="test_appcode_12345"

# 3. 启动节点并查看日志
./target/release/stardust-node --dev 2>&1 | grep -i almanac

# 预期输出:
# ✅ Almanac AppCode configured (length: 18 bytes)
```

## OCW 中读取 AppCode

在 `pallet-almanac` 的 Off-chain Worker 中,通过以下方式读取:

```rust
impl<T: Config> Pallet<T> {
    fn get_appcode() -> Result<Vec<u8>, &'static str> {
        // 从 OCW 本地存储读取
        sp_io::offchain::local_storage_get(
            sp_core::offchain::StorageKind::PERSISTENT,
            b"almanac::appcode",
        )
        .ok_or("AppCode not configured")
    }

    fn fetch_almanac_from_api(year: u32, month: u8, day: u8) -> Result<AlmanacInfo, &'static str> {
        // 获取 AppCode
        let appcode = Self::get_appcode()?;
        let appcode_str = sp_std::str::from_utf8(&appcode)
            .map_err(|_| "Invalid AppCode UTF-8")?;

        // 构造 HTTP 请求
        let url = "https://jmhlysjjr.market.alicloudapi.com/holiday/list";
        let body = format!("year={}", year);

        let request = http::Request::post(url, vec![body.as_bytes()])
            .add_header("Authorization", &format!("APPCODE {}", appcode_str))
            .add_header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .deadline(sp_io::offchain::timestamp().add(Duration::from_millis(10000)));

        // ... 发送请求
    }
}
```

## 安全注意事项

### ✅ 良好实践

1. **不要提交到 Git**
   ```bash
   # .gitignore
   .env
   *.env
   secrets.*
   ```

2. **限制文件权限**
   ```bash
   chmod 600 /etc/stardust/.env
   chown stardust:stardust /etc/stardust/.env
   ```

3. **环境隔离**
   - 开发环境: 使用测试 AppCode
   - 生产环境: 使用正式 AppCode

4. **日志脱敏**
   - 不在日志中打印完整 AppCode
   - 只显示长度或前几位

### ⚠️ 避免的做法

1. ❌ 不要硬编码在源代码中
2. ❌ 不要提交到版本控制
3. ❌ 不要在进程标题中显示
4. ❌ 不要在公共日志中打印

## 常见问题

### Q1: AppCode 在哪里存储?
A: 存储在节点的 OCW 本地数据库中,路径通常是 `{base_path}/offchains/`

### Q2: 如何更换 AppCode?
A: 重启节点时使用新的环境变量或参数即可,会覆盖旧值

### Q3: 多个节点如何共享 AppCode?
A:
- 方案一: 每个节点独立配置环境变量
- 方案二: 使用配置管理工具(如 Ansible)统一分发
- 方案三: 使用链上加密存储(高级方案,见 APPCODE_SECURITY.md)

### Q4: AppCode 丢失怎么办?
A: OCW 会报错 "AppCode not configured",不会影响节点运行,只是黄历功能不可用

### Q5: 如何验证 AppCode 是否生效?
A:
1. 查看启动日志中的 "✅ Almanac AppCode configured"
2. 在 pallet-almanac 的 OCW 中尝试调用 API
3. 检查 offchain 存储: `{base_path}/offchains/*/almanac::appcode`

## 下一步

1. ✅ 实现 pallet-almanac 的 OCW 逻辑,读取并使用 AppCode
2. ✅ 添加 AppCode 有效性验证(可选)
3. ✅ 实现链上加密存储方案(生产环境,见 APPCODE_SECURITY.md)
4. ✅ 添加 AppCode 轮换机制
5. ✅ 实现监控告警(AppCode 过期、API 调用失败等)

## 示例脚本

### 开发环境快速启动

```bash
#!/bin/bash
# dev-start.sh

# 设置开发环境 AppCode
export ALMANAC_APPCODE="dev_test_appcode_12345"

# 清理开发链状态
./target/release/stardust-node purge-chain --dev -y

# 启动开发链
./target/release/stardust-node \
  --dev \
  --tmp \
  --offchain-worker=Always \
  --enable-offchain-indexing=true
```

### 生产环境启动脚本

```bash
#!/bin/bash
# prod-start.sh

# 从加密文件读取 AppCode
if [ -f /etc/stardust/secrets.encrypted ]; then
  export ALMANAC_APPCODE=$(decrypt-tool /etc/stardust/secrets.encrypted)
fi

# 验证 AppCode 已设置
if [ -z "$ALMANAC_APPCODE" ]; then
  echo "Error: ALMANAC_APPCODE not set"
  exit 1
fi

# 启动节点
exec /usr/local/bin/stardust-node \
  --chain=production \
  --base-path=/var/lib/stardust \
  --offchain-worker=Always \
  --enable-offchain-indexing=true \
  --prometheus-external
```

## 相关文档

- [APPCODE_SECURITY.md](./APPCODE_SECURITY.md) - 详细的安全方案设计
- [DESIGN.md](./DESIGN.md) - pallet-almanac 完整设计方案
