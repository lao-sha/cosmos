# 玄学占卜 NFT 模块 (pallet-divination-nft)

通用的占卜结果 NFT 模块，支持将各类玄学占卜结果（梅花易数、八字、塔罗等）铸造为链上 NFT，并提供完整的交易市场功能。

## 概述

本模块实现了：

- **NFT 铸造**：基于占卜结果自动判定稀有度
- **交易市场**：定价挂单、议价出价、安全交易
- **收藏展示**：个人收藏集、公开展示
- **版税机制**：创作者在每次转售时获得版税

## 架构设计

通过 `DivinationProvider` trait 与各玄学核心模块解耦：

```text
┌─────────────────────────────────────────────────────────┐
│                   pallet-divination-nft                 │
│    (通用 NFT 铸造、交易、收藏功能)                        │
└──────────────────────────┬──────────────────────────────┘
                           │ DivinationProvider trait
                           ▼
┌─────────────────────────────────────────────────────────┐
│              Runtime: CombinedDivinationProvider        │
└───────┬─────────────────────────────────────────────────┘
        │
        ├──► pallet-meihua (梅花易数)
        ├──► pallet-bazi (八字排盘)
        ├──► pallet-tarot (塔罗牌)
        └──► ... (其他玄学系统)
```

## 核心功能

### 1. NFT 铸造

```rust
mint_nft(
    divination_type: DivinationType,  // 占卜类型
    result_id: u64,                   // 占卜结果 ID
    name: Vec<u8>,                    // NFT 名称
    image_cid: Vec<u8>,               // 图片 IPFS CID
    description_cid: Option<Vec<u8>>, // 描述 CID
    animation_cid: Option<Vec<u8>>,   // 动画 CID
    royalty_rate: u16,                // 版税比例（万分比）
)
```

铸造规则：
- 仅占卜结果创建者可铸造
- 每个占卜结果只能铸造一次
- 自动计算稀有度并收取对应费用

### 2. NFT 交易

```rust
// 转移 NFT
transfer_nft(nft_id: u64, to: AccountId)

// 销毁 NFT
burn_nft(nft_id: u64)

// 挂单出售
list_nft(nft_id: u64, price: Balance, expires_in: Option<BlockNumber>)

// 取消挂单
cancel_listing(nft_id: u64)

// 购买挂单 NFT
buy_nft(nft_id: u64)
```

### 3. 议价系统

```rust
// 提交出价
make_offer(nft_id: u64, amount: Balance)

// 取消出价
cancel_offer(offer_id: u64)

// 接受出价
accept_offer(offer_id: u64)
```

### 4. 收藏集管理

```rust
// 创建收藏集
create_collection(name: Vec<u8>, description_cid: Option<Vec<u8>>)

// 添加 NFT 到收藏集
add_to_collection(nft_id: u64, collection_id: u32)

// 从收藏集移除
remove_from_collection(nft_id: u64, collection_id: u32)

// 删除收藏集
delete_collection(collection_id: u32)
```

## 稀有度系统

### 稀有度等级

| 等级 | 分数范围 | 铸造费用倍率 | 供应上限 |
|------|---------|-------------|---------|
| Common | 0-100 | 1.0x | 无限 |
| Rare | 101-200 | 1.5x | 10,000 |
| Epic | 201-400 | 2.5x | 1,000 |
| Legendary | 401+ | 5.0x | 100 |

### 稀有度计算

稀有度由各占卜模块通过 `DivinationProvider::rarity_data()` 提供：

```rust
// 梅花易数示例
let rarity_input = RarityInput {
    primary_score: if is_pure_gua { 80 } else { 30 },  // 纯卦高分
    secondary_score: dong_yao_score,                    // 动爻分数
    is_special_date: is_festival,                       // 节日加成
    is_special_combination: is_pure_gua,                // 特殊组合
    custom_factors: [0, 0, 0, 0],
};
```

## 费用机制

### 铸造费用

```
铸造费用 = 基础费用 × 稀有度倍率
```

### 交易费用分配

```
卖家收入 = 成交价 - 平台手续费 - 版税
平台手续费 = 成交价 × 平台费率（默认 2.5%）
版税 = 成交价 × 版税比例（创作者设定，最高 10%）
```

## 数据结构

### DivinationNft - NFT 结构

```rust
pub struct DivinationNft<AccountId, Balance, BlockNumber, MaxCid, MaxName> {
    pub id: u64,
    pub divination_type: DivinationType,
    pub result_id: u64,
    pub owner: AccountId,
    pub creator: AccountId,
    pub rarity: Rarity,
    pub status: NftStatus,
    pub metadata: NftMetadata<MaxCid, MaxName>,
    pub minted_at: BlockNumber,
    pub mint_fee: Balance,
    pub royalty_rate: u16,
    pub transfer_count: u32,
}
```

### NftStatus - NFT 状态

```rust
pub enum NftStatus {
    Normal,   // 正常
    Listed,   // 挂单中
    Burned,   // 已销毁
}
```

### Listing - 挂单信息

```rust
pub struct Listing<AccountId, Balance, BlockNumber> {
    pub nft_id: u64,
    pub seller: AccountId,
    pub price: Balance,
    pub listed_at: BlockNumber,
    pub expires_at: Option<BlockNumber>,
}
```

### Offer - 出价信息

```rust
pub struct Offer<AccountId, Balance, BlockNumber> {
    pub id: u64,
    pub nft_id: u64,
    pub bidder: AccountId,
    pub amount: Balance,
    pub offered_at: BlockNumber,
    pub expires_at: BlockNumber,
    pub is_active: bool,
}
```

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// 占卜结果查询接口
    type DivinationProvider: DivinationProvider<Self::AccountId>;
    
    /// 基础铸造费用
    type BaseMintFee: Get<Balance>;
    
    /// 平台交易手续费率（万分比）
    type PlatformFeeRate: Get<u16>;
    
    /// 最大版税比例（万分比）
    type MaxRoyaltyRate: Get<u16>;
    
    /// 出价有效期（区块数）
    type OfferValidityPeriod: Get<BlockNumber>;
    
    /// 每用户最大收藏集数量
    type MaxCollectionsPerUser: Get<u32>;
    
    /// 每收藏集最大 NFT 数量
    type MaxNftsPerCollection: Get<u32>;
}
```

## 事件

```rust
NftMinted { nft_id, divination_type, result_id, owner, rarity, mint_fee }
NftTransferred { nft_id, from, to }
NftBurned { nft_id, owner }
NftListed { nft_id, seller, price }
ListingCancelled { nft_id }
NftSold { nft_id, seller, buyer, price, royalty, platform_fee }
OfferMade { offer_id, nft_id, bidder, amount }
OfferCancelled { offer_id }
OfferAccepted { offer_id, nft_id, seller, buyer, amount }
CollectionCreated { collection_id, creator }
```

## 统计数据

模块自动维护全局和按类型的统计数据：

```rust
pub struct NftStats<Balance> {
    pub total_minted: u64,
    pub total_burned: u64,
    pub total_trades: u64,
    pub total_volume: Balance,
    pub active_listings: u64,
    pub common_count: u64,
    pub rare_count: u64,
    pub epic_count: u64,
    pub legendary_count: u64,
}
```

## 依赖

```toml
[dependencies]
pallet-divination-nft = { path = "../nft", default-features = false }
pallet-divination-common = { path = "../common", default-features = false }
```

## License

MIT
