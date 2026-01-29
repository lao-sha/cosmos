#!/bin/bash

# Cosmos 项目重命名脚本
# 将 Stardust 项目重命名为 Cosmos，代币 DUST 重命名为 COS
#
# 使用方法:
#   chmod +x scripts/rename-to-cosmos.sh
#   ./scripts/rename-to-cosmos.sh
#
# 注意: 执行前请确保已创建备份分支！

set -e

echo "=========================================="
echo "Cosmos 项目重命名脚本"
echo "=========================================="
echo ""
echo "⚠️  警告: 此脚本将修改大量文件！"
echo "⚠️  请确保已创建备份分支！"
echo ""
read -p "是否继续？(yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "已取消操作。"
    exit 0
fi

echo ""
echo "开始批量替换..."
echo ""

# 定义排除路径
EXCLUDE_PATHS="-not -path '*/node_modules/*' -not -path '*/target/*' -not -path '*/.git/*' -not -path '*/Cargo.lock' -not -path '*/package-lock.json'"

# 1. 项目名称替换: Stardust → Cosmos
echo "[1/7] 替换项目名称: Stardust → Cosmos"
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.md" -o -name "*.toml" -o -name "*.json" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/Stardust/Cosmos/g' {} + 2>/dev/null || true
echo "✓ 完成"

# 2. 代币符号替换: DUST → COS (大写)
echo "[2/7] 替换代币符号: DUST → COS"
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.md" -o -name "*.toml" -o -name "*.json" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/DUST/COS/g' {} + 2>/dev/null || true
echo "✓ 完成"

# 3. 小写键名替换: 'stardust_' → 'cosmos_'
echo "[3/7] 替换存储键名: 'stardust_' → 'cosmos_'"
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i "s/'stardust_/'cosmos_/g" {} + 2>/dev/null || true
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/"stardust_/"cosmos_/g' {} + 2>/dev/null || true
echo "✓ 完成"

# 4. 小写项目名替换: stardust → cosmos (用于变量名、函数名等)
echo "[4/7] 替换小写项目名: stardust → cosmos"
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/stardust_dev/cosmos_dev/g' {} + 2>/dev/null || true
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/stardust_local/cosmos_local/g' {} + 2>/dev/null || true
echo "✓ 完成"

# 5. Pallet 名称替换: pallet-stardust-ipfs → pallet-cosmos-ipfs
echo "[5/7] 替换 Pallet 名称: pallet-stardust-ipfs → pallet-cosmos-ipfs"
find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/pallet-stardust-ipfs/pallet-cosmos-ipfs/g' {} + 2>/dev/null || true
find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.toml" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/StardustIpfs/CosmosIpfs/g' {} + 2>/dev/null || true
echo "✓ 完成"

# 6. 邮箱域名替换: @stardust.app → @cosmos.app
echo "[6/7] 替换邮箱域名: @stardust.app → @cosmos.app"
find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.md" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/@stardust\.app/@cosmos.app/g' {} + 2>/dev/null || true
find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.md" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/support@stardust\.app/support@cosmos.app/g' {} + 2>/dev/null || true
find . -type f \( -name "*.ts" -o -name "*.tsx" -o -name "*.md" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/privacy@stardust\.app/privacy@cosmos.app/g' {} + 2>/dev/null || true
echo "✓ 完成"

# 7. 特殊替换: dust → cos (用于变量名，需要小心)
echo "[7/7] 替换变量名: dust → cos (仅限注释和字符串)"
# 只替换注释和字符串中的 dust_amount, dust_qty 等
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/dust_amount/cos_amount/g' {} + 2>/dev/null || true
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/dust_qty/cos_qty/g' {} + 2>/dev/null || true
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" \) \
  $EXCLUDE_PATHS \
  -exec sed -i 's/get_dust_to_usd_rate/get_cos_to_usd_rate/g' {} + 2>/dev/null || true
echo "✓ 完成"

echo ""
echo "=========================================="
echo "批量替换完成！"
echo "=========================================="
echo ""
echo "⚠️  重要提示:"
echo "1. 请检查关键文件是否正确修改:"
echo "   - runtime/src/lib.rs (spec_name, impl_name)"
echo "   - node/src/chain_spec.rs (tokenSymbol, chain names)"
echo "   - frontend/src/lib/wallet.ts (存储键名)"
echo ""
echo "2. 运行编译测试:"
echo "   cargo build"
echo "   cd frontend && npm run build"
echo ""
echo "3. 检查 Git 修改:"
echo "   git status"
echo "   git diff"
echo ""
echo "4. 如有问题，可以从备份分支恢复:"
echo "   git checkout backup-before-cosmos-rename"
echo ""

