# 服务提供者个人主页扩展方案

## 1. 概述

### 1.1 背景

当前 `pallet-divination-market` 已实现基础的服务提供者功能，包括：
- 提供者注册与基本信息（name, bio, avatar_cid）
- 服务套餐管理
- 订单与评价系统
- 等级晋升机制

但缺少一个**完整的个人主页展示系统**，用户难以全面了解服务提供者的专业背景和服务能力。

### 1.2 目标

设计并实现服务提供者个人主页功能，包括：
1. **丰富的个人资料** - 展示专业背景、从业经验、资质证书
2. **技能标签系统** - 直观展示擅长领域和占卜类型
3. **作品集展示** - 精选案例和解读样本
4. **数据统计面板** - 服务数据可视化
5. **用户评价聚合** - 评价分类展示

---

## 2. 数据结构设计

### 2.1 新增类型定义 (`types.rs`)

```rust
/// 服务提供者详细资料
///
/// 用于个人主页展示的扩展信息
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxDetailLen, MaxCidLen))]
pub struct ProviderProfile<BlockNumber, MaxDetailLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 详细自我介绍 IPFS CID（支持富文本/Markdown）
    pub introduction_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 从业年限
    pub experience_years: u8,

    /// 师承/学习背景
    pub background: Option<BoundedVec<u8, MaxDetailLen>>,

    /// 服务理念/座右铭
    pub motto: Option<BoundedVec<u8, ConstU32<256>>>,

    /// 擅长问题类型描述
    pub expertise_description: Option<BoundedVec<u8, MaxDetailLen>>,

    /// 工作时间说明（如：每日 9:00-21:00）
    pub working_hours: Option<BoundedVec<u8, ConstU32<128>>>,

    /// 平均响应时间（分钟）
    pub avg_response_time: Option<u32>,

    /// 是否接受预约
    pub accepts_appointment: bool,

    /// 联系方式（可选，IPFS CID 加密存储）
    pub contact_info_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 个人主页背景图 IPFS CID
    pub banner_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 资料最后更新时间
    pub updated_at: BlockNumber,
}

/// 资质证书
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxNameLen, MaxCidLen))]
pub struct Certificate<BlockNumber, MaxNameLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 证书 ID
    pub id: u32,

    /// 证书名称
    pub name: BoundedVec<u8, MaxNameLen>,

    /// 证书类型
    pub cert_type: CertificateType,

    /// 颁发机构
    pub issuer: Option<BoundedVec<u8, MaxNameLen>>,

    /// 证书图片 IPFS CID
    pub image_cid: BoundedVec<u8, MaxCidLen>,

    /// 颁发时间（区块号或时间戳）
    pub issued_at: Option<BlockNumber>,

    /// 是否已验证（管理员验证）
    pub is_verified: bool,

    /// 上传时间
    pub uploaded_at: BlockNumber,
}

/// 证书类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum CertificateType {
    /// 学历证书
    #[default]
    Education = 0,
    /// 专业资格证书
    Professional = 1,
    /// 行业协会认证
    Association = 2,
    /// 师承证明
    Apprenticeship = 3,
    /// 获奖证书
    Award = 4,
    /// 其他
    Other = 5,
}

/// 作品集/案例展示
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxTitleLen, MaxCidLen))]
pub struct PortfolioItem<BlockNumber, MaxTitleLen: Get<u32>, MaxCidLen: Get<u32>> {
    /// 作品 ID
    pub id: u32,

    /// 作品标题
    pub title: BoundedVec<u8, MaxTitleLen>,

    /// 占卜类型
    pub divination_type: DivinationType,

    /// 案例类型
    pub case_type: PortfolioCaseType,

    /// 案例内容 IPFS CID（脱敏后的解读案例）
    pub content_cid: BoundedVec<u8, MaxCidLen>,

    /// 封面图片 IPFS CID
    pub cover_cid: Option<BoundedVec<u8, MaxCidLen>>,

    /// 是否精选（置顶展示）
    pub is_featured: bool,

    /// 浏览次数
    pub view_count: u32,

    /// 点赞次数
    pub like_count: u32,

    /// 发布时间
    pub published_at: BlockNumber,
}

/// 案例类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum PortfolioCaseType {
    /// 经典解读案例
    #[default]
    ClassicCase = 0,
    /// 教学文章
    Tutorial = 1,
    /// 理论研究
    Research = 2,
    /// 心得分享
    Sharing = 3,
}

/// 技能标签
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
#[scale_info(skip_type_params(MaxLabelLen))]
pub struct SkillTag<MaxLabelLen: Get<u32>> {
    /// 标签名称
    pub label: BoundedVec<u8, MaxLabelLen>,

    /// 标签类型
    pub tag_type: SkillTagType,

    /// 熟练程度（1-5）
    pub proficiency: u8,
}

/// 技能标签类型
#[derive(Clone, Copy, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub enum SkillTagType {
    /// 占卜类型相关
    #[default]
    DivinationType = 0,
    /// 擅长领域
    Specialty = 1,
    /// 服务特色
    ServiceFeature = 2,
    /// 自定义标签
    Custom = 3,
}

/// 提供者统计摘要（用于主页展示）
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct ProviderStatsSummary<Balance: Default> {
    /// 总服务人次
    pub total_customers: u32,

    /// 本月订单数
    pub monthly_orders: u32,

    /// 本周订单数
    pub weekly_orders: u32,

    /// 回头客比例（基点，10000 = 100%）
    pub repeat_customer_rate: u16,

    /// 平均解读时长（分钟）
    pub avg_interpretation_time: u32,

    /// 各评分维度平均分（* 100）
    pub avg_accuracy_rating: u16,
    pub avg_attitude_rating: u16,
    pub avg_response_rating: u16,

    /// 5星好评率（基点）
    pub five_star_rate: u16,

    /// 悬赏被采纳次数
    pub bounty_adoptions: u32,

    /// 悬赏获奖总金额
    pub bounty_earnings: Balance,
}

/// 评价标签统计
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
pub struct ReviewTagStats {
    /// "解读准确" 次数
    pub accurate_count: u32,
    /// "态度友好" 次数
    pub friendly_count: u32,
    /// "回复及时" 次数
    pub quick_response_count: u32,
    /// "专业深入" 次数
    pub professional_count: u32,
    /// "耐心解答" 次数
    pub patient_count: u32,
    /// "物超所值" 次数
    pub value_for_money_count: u32,
}
```

### 2.2 新增存储项 (`lib.rs`)

```rust
/// 提供者详细资料
#[pallet::storage]
#[pallet::getter(fn provider_profiles)]
pub type ProviderProfiles<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    ProviderProfile<BlockNumberFor<T>, T::MaxDescriptionLength, T::MaxCidLength>,
>;

/// 提供者资质证书（提供者 -> 证书ID -> 证书）
#[pallet::storage]
#[pallet::getter(fn certificates)]
pub type Certificates<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    u32,
    Certificate<BlockNumberFor<T>, T::MaxNameLength, T::MaxCidLength>,
>;

/// 提供者下一个证书 ID
#[pallet::storage]
#[pallet::getter(fn next_certificate_id)]
pub type NextCertificateId<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

/// 提供者作品集（提供者 -> 作品ID -> 作品）
#[pallet::storage]
#[pallet::getter(fn portfolios)]
pub type Portfolios<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    Blake2_128Concat,
    u32,
    PortfolioItem<BlockNumberFor<T>, T::MaxNameLength, T::MaxCidLength>,
>;

/// 提供者下一个作品 ID
#[pallet::storage]
#[pallet::getter(fn next_portfolio_id)]
pub type NextPortfolioId<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

/// 提供者技能标签
#[pallet::storage]
#[pallet::getter(fn skill_tags)]
pub type SkillTags<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    BoundedVec<SkillTag<ConstU32<32>>, ConstU32<20>>,
    ValueQuery,
>;

/// 提供者统计摘要（链下计算，定期更新）
#[pallet::storage]
#[pallet::getter(fn provider_stats_summary)]
pub type ProviderStatsSummary<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,
    ProviderStatsSummary<BalanceOf<T>>,
>;

/// 提供者评价标签统计
#[pallet::storage]
#[pallet::getter(fn review_tag_stats)]
pub type ReviewTagStatistics<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, ReviewTagStats, ValueQuery>;

/// 作品点赞记录（作品ID -> 用户 -> 是否点赞）
#[pallet::storage]
#[pallet::getter(fn portfolio_likes)]
pub type PortfolioLikes<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    (T::AccountId, u32), // (provider, portfolio_id)
    Blake2_128Concat,
    T::AccountId,        // liker
    bool,
    ValueQuery,
>;
```

---

## 3. 外部调用函数设计

### 3.1 个人资料管理

```rust
/// 更新提供者详细资料
///
/// # 参数
/// - `introduction_cid`: 详细自我介绍 IPFS CID
/// - `experience_years`: 从业年限
/// - `background`: 师承/学习背景
/// - `motto`: 服务理念/座右铭
/// - `expertise_description`: 擅长问题类型描述
/// - `working_hours`: 工作时间说明
/// - `avg_response_time`: 平均响应时间（分钟）
/// - `accepts_appointment`: 是否接受预约
/// - `banner_cid`: 主页背景图 CID
#[pallet::call_index(26)]
#[pallet::weight(Weight::from_parts(40_000_000, 0))]
pub fn update_profile(
    origin: OriginFor<T>,
    introduction_cid: Option<Vec<u8>>,
    experience_years: Option<u8>,
    background: Option<Vec<u8>>,
    motto: Option<Vec<u8>>,
    expertise_description: Option<Vec<u8>>,
    working_hours: Option<Vec<u8>>,
    avg_response_time: Option<u32>,
    accepts_appointment: Option<bool>,
    banner_cid: Option<Vec<u8>>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 验证是注册的提供者
    ensure!(
        Providers::<T>::contains_key(&who),
        Error::<T>::ProviderNotFound
    );

    let current_block = <frame_system::Pallet<T>>::block_number();

    ProviderProfiles::<T>::try_mutate(&who, |maybe_profile| {
        let profile = maybe_profile.get_or_insert_with(|| ProviderProfile {
            introduction_cid: None,
            experience_years: 0,
            background: None,
            motto: None,
            expertise_description: None,
            working_hours: None,
            avg_response_time: None,
            accepts_appointment: false,
            contact_info_cid: None,
            banner_cid: None,
            updated_at: current_block,
        });

        if let Some(cid) = introduction_cid {
            profile.introduction_cid = Some(
                BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
            );
        }
        if let Some(years) = experience_years {
            profile.experience_years = years;
        }
        if let Some(bg) = background {
            profile.background = Some(
                BoundedVec::try_from(bg).map_err(|_| Error::<T>::DescriptionTooLong)?
            );
        }
        if let Some(m) = motto {
            profile.motto = Some(
                BoundedVec::try_from(m).map_err(|_| Error::<T>::DescriptionTooLong)?
            );
        }
        if let Some(exp) = expertise_description {
            profile.expertise_description = Some(
                BoundedVec::try_from(exp).map_err(|_| Error::<T>::DescriptionTooLong)?
            );
        }
        if let Some(wh) = working_hours {
            profile.working_hours = Some(
                BoundedVec::try_from(wh).map_err(|_| Error::<T>::DescriptionTooLong)?
            );
        }
        if let Some(time) = avg_response_time {
            profile.avg_response_time = Some(time);
        }
        if let Some(accepts) = accepts_appointment {
            profile.accepts_appointment = accepts;
        }
        if let Some(cid) = banner_cid {
            profile.banner_cid = Some(
                BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
            );
        }

        profile.updated_at = current_block;

        Ok::<_, DispatchError>(())
    })?;

    Self::deposit_event(Event::ProfileUpdated { provider: who });

    Ok(())
}
```

### 3.2 资质证书管理

```rust
/// 添加资质证书
#[pallet::call_index(27)]
#[pallet::weight(Weight::from_parts(35_000_000, 0))]
pub fn add_certificate(
    origin: OriginFor<T>,
    name: Vec<u8>,
    cert_type: CertificateType,
    issuer: Option<Vec<u8>>,
    image_cid: Vec<u8>,
    issued_at: Option<BlockNumberFor<T>>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    ensure!(
        Providers::<T>::contains_key(&who),
        Error::<T>::ProviderNotFound
    );

    let cert_id = NextCertificateId::<T>::get(&who);
    ensure!(cert_id < T::MaxCertificatesPerProvider::get(), Error::<T>::TooManyCertificates);

    let name_bounded = BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
    let image_cid_bounded = BoundedVec::try_from(image_cid).map_err(|_| Error::<T>::CidTooLong)?;
    let issuer_bounded = issuer
        .map(|i| BoundedVec::try_from(i).map_err(|_| Error::<T>::NameTooLong))
        .transpose()?;

    let certificate = Certificate {
        id: cert_id,
        name: name_bounded,
        cert_type,
        issuer: issuer_bounded,
        image_cid: image_cid_bounded,
        issued_at,
        is_verified: false,
        uploaded_at: <frame_system::Pallet<T>>::block_number(),
    };

    Certificates::<T>::insert(&who, cert_id, certificate);
    NextCertificateId::<T>::insert(&who, cert_id.saturating_add(1));

    Self::deposit_event(Event::CertificateAdded {
        provider: who,
        certificate_id: cert_id,
    });

    Ok(())
}

/// 删除资质证书
#[pallet::call_index(28)]
#[pallet::weight(Weight::from_parts(20_000_000, 0))]
pub fn remove_certificate(
    origin: OriginFor<T>,
    certificate_id: u32,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    ensure!(
        Certificates::<T>::contains_key(&who, certificate_id),
        Error::<T>::CertificateNotFound
    );

    Certificates::<T>::remove(&who, certificate_id);

    Self::deposit_event(Event::CertificateRemoved {
        provider: who,
        certificate_id,
    });

    Ok(())
}

/// 验证资质证书（治理权限）
#[pallet::call_index(29)]
#[pallet::weight(Weight::from_parts(25_000_000, 0))]
pub fn verify_certificate(
    origin: OriginFor<T>,
    provider: T::AccountId,
    certificate_id: u32,
    is_verified: bool,
) -> DispatchResult {
    T::GovernanceOrigin::ensure_origin(origin)?;

    Certificates::<T>::try_mutate(&provider, certificate_id, |maybe_cert| {
        let cert = maybe_cert.as_mut().ok_or(Error::<T>::CertificateNotFound)?;
        cert.is_verified = is_verified;
        Ok::<_, DispatchError>(())
    })?;

    Self::deposit_event(Event::CertificateVerified {
        provider,
        certificate_id,
        is_verified,
    });

    Ok(())
}
```

### 3.3 作品集管理

```rust
/// 发布作品/案例
#[pallet::call_index(30)]
#[pallet::weight(Weight::from_parts(40_000_000, 0))]
pub fn publish_portfolio(
    origin: OriginFor<T>,
    title: Vec<u8>,
    divination_type: DivinationType,
    case_type: PortfolioCaseType,
    content_cid: Vec<u8>,
    cover_cid: Option<Vec<u8>>,
    is_featured: bool,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    ensure!(
        Providers::<T>::contains_key(&who),
        Error::<T>::ProviderNotFound
    );

    let portfolio_id = NextPortfolioId::<T>::get(&who);
    ensure!(portfolio_id < T::MaxPortfoliosPerProvider::get(), Error::<T>::TooManyPortfolios);

    let title_bounded = BoundedVec::try_from(title).map_err(|_| Error::<T>::NameTooLong)?;
    let content_cid_bounded = BoundedVec::try_from(content_cid).map_err(|_| Error::<T>::CidTooLong)?;
    let cover_cid_bounded = cover_cid
        .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
        .transpose()?;

    let portfolio = PortfolioItem {
        id: portfolio_id,
        title: title_bounded,
        divination_type,
        case_type,
        content_cid: content_cid_bounded,
        cover_cid: cover_cid_bounded,
        is_featured,
        view_count: 0,
        like_count: 0,
        published_at: <frame_system::Pallet<T>>::block_number(),
    };

    Portfolios::<T>::insert(&who, portfolio_id, portfolio);
    NextPortfolioId::<T>::insert(&who, portfolio_id.saturating_add(1));

    Self::deposit_event(Event::PortfolioPublished {
        provider: who,
        portfolio_id,
        divination_type,
    });

    Ok(())
}

/// 更新作品
#[pallet::call_index(31)]
#[pallet::weight(Weight::from_parts(30_000_000, 0))]
pub fn update_portfolio(
    origin: OriginFor<T>,
    portfolio_id: u32,
    title: Option<Vec<u8>>,
    content_cid: Option<Vec<u8>>,
    cover_cid: Option<Vec<u8>>,
    is_featured: Option<bool>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    Portfolios::<T>::try_mutate(&who, portfolio_id, |maybe_portfolio| {
        let portfolio = maybe_portfolio.as_mut().ok_or(Error::<T>::PortfolioNotFound)?;

        if let Some(t) = title {
            portfolio.title = BoundedVec::try_from(t).map_err(|_| Error::<T>::NameTooLong)?;
        }
        if let Some(cid) = content_cid {
            portfolio.content_cid = BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?;
        }
        if let Some(cid) = cover_cid {
            portfolio.cover_cid = Some(
                BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
            );
        }
        if let Some(f) = is_featured {
            portfolio.is_featured = f;
        }

        Ok::<_, DispatchError>(())
    })?;

    Self::deposit_event(Event::PortfolioUpdated {
        provider: who,
        portfolio_id,
    });

    Ok(())
}

/// 删除作品
#[pallet::call_index(32)]
#[pallet::weight(Weight::from_parts(20_000_000, 0))]
pub fn remove_portfolio(
    origin: OriginFor<T>,
    portfolio_id: u32,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    ensure!(
        Portfolios::<T>::contains_key(&who, portfolio_id),
        Error::<T>::PortfolioNotFound
    );

    Portfolios::<T>::remove(&who, portfolio_id);

    Self::deposit_event(Event::PortfolioRemoved {
        provider: who,
        portfolio_id,
    });

    Ok(())
}

/// 点赞作品
#[pallet::call_index(33)]
#[pallet::weight(Weight::from_parts(25_000_000, 0))]
pub fn like_portfolio(
    origin: OriginFor<T>,
    provider: T::AccountId,
    portfolio_id: u32,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 验证作品存在
    ensure!(
        Portfolios::<T>::contains_key(&provider, portfolio_id),
        Error::<T>::PortfolioNotFound
    );

    // 检查是否已点赞
    let key = (provider.clone(), portfolio_id);
    ensure!(
        !PortfolioLikes::<T>::get(&key, &who),
        Error::<T>::AlreadyLiked
    );

    // 记录点赞
    PortfolioLikes::<T>::insert(&key, &who, true);

    // 更新点赞数
    Portfolios::<T>::mutate(&provider, portfolio_id, |maybe_portfolio| {
        if let Some(p) = maybe_portfolio {
            p.like_count += 1;
        }
    });

    Self::deposit_event(Event::PortfolioLiked {
        provider,
        portfolio_id,
        liker: who,
    });

    Ok(())
}
```

### 3.4 技能标签管理

```rust
/// 设置技能标签
#[pallet::call_index(34)]
#[pallet::weight(Weight::from_parts(30_000_000, 0))]
pub fn set_skill_tags(
    origin: OriginFor<T>,
    tags: Vec<(Vec<u8>, SkillTagType, u8)>, // (label, type, proficiency)
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    ensure!(
        Providers::<T>::contains_key(&who),
        Error::<T>::ProviderNotFound
    );

    let mut skill_tags: BoundedVec<SkillTag<ConstU32<32>>, ConstU32<20>> = BoundedVec::new();

    for (label, tag_type, proficiency) in tags {
        ensure!(proficiency >= 1 && proficiency <= 5, Error::<T>::InvalidRating);

        let label_bounded = BoundedVec::try_from(label)
            .map_err(|_| Error::<T>::NameTooLong)?;

        skill_tags.try_push(SkillTag {
            label: label_bounded,
            tag_type,
            proficiency,
        }).map_err(|_| Error::<T>::TooManyTags)?;
    }

    SkillTags::<T>::insert(&who, skill_tags);

    Self::deposit_event(Event::SkillTagsUpdated { provider: who });

    Ok(())
}
```

---

## 4. 新增事件

```rust
/// 个人资料已更新
ProfileUpdated { provider: T::AccountId },

/// 资质证书已添加
CertificateAdded {
    provider: T::AccountId,
    certificate_id: u32,
},

/// 资质证书已删除
CertificateRemoved {
    provider: T::AccountId,
    certificate_id: u32,
},

/// 资质证书验证状态已更新
CertificateVerified {
    provider: T::AccountId,
    certificate_id: u32,
    is_verified: bool,
},

/// 作品已发布
PortfolioPublished {
    provider: T::AccountId,
    portfolio_id: u32,
    divination_type: DivinationType,
},

/// 作品已更新
PortfolioUpdated {
    provider: T::AccountId,
    portfolio_id: u32,
},

/// 作品已删除
PortfolioRemoved {
    provider: T::AccountId,
    portfolio_id: u32,
},

/// 作品被点赞
PortfolioLiked {
    provider: T::AccountId,
    portfolio_id: u32,
    liker: T::AccountId,
},

/// 技能标签已更新
SkillTagsUpdated { provider: T::AccountId },
```

---

## 5. 新增错误类型

```rust
/// 资质证书不存在
CertificateNotFound,
/// 证书数量已达上限
TooManyCertificates,
/// 作品不存在
PortfolioNotFound,
/// 作品数量已达上限
TooManyPortfolios,
/// 已点赞
AlreadyLiked,
/// 标签数量过多
TooManyTags,
```

---

## 6. 新增配置常量

```rust
/// 每个提供者最大证书数
#[pallet::constant]
type MaxCertificatesPerProvider: Get<u32>;

/// 每个提供者最大作品数
#[pallet::constant]
type MaxPortfoliosPerProvider: Get<u32>;
```

**推荐默认值：**
- `MaxCertificatesPerProvider`: 10
- `MaxPortfoliosPerProvider`: 50

---

## 7. 前端个人主页设计

### 7.1 页面结构

```
┌─────────────────────────────────────────────────────────────┐
│                    [背景横幅图片]                              │
├─────────────────────────────────────────────────────────────┤
│  [头像]  昵称 · 等级徽章 · 认证标识                            │
│          ⭐ 4.8 (328评价) | 完成 1,234 单                     │
│          「座右铭/服务理念」                                   │
├─────────────────────────────────────────────────────────────┤
│  [技能标签云]                                                 │
│  #八字命理 #婚姻感情 #事业财运 #梅花易数 ...                    │
├─────────────────────────────────────────────────────────────┤
│  📊 数据面板                                                  │
│  ┌──────────┬──────────┬──────────┬──────────┐              │
│  │ 服务人次  │ 本月订单  │ 好评率   │ 响应时间  │              │
│  │  1,234   │   45     │  98.5%  │  30分钟  │              │
│  └──────────┴──────────┴──────────┴──────────┘              │
├─────────────────────────────────────────────────────────────┤
│  📋 服务套餐 [查看全部]                                        │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │ 八字详批        │  │ 梅花快占        │                   │
│  │ ¥199 | 已售 234 │  │ ¥59 | 已售 567  │                   │
│  └─────────────────┘  └─────────────────┘                   │
├─────────────────────────────────────────────────────────────┤
│  📜 资质证书 [已认证 ✓]                                        │
│  ┌─────┐ ┌─────┐ ┌─────┐                                    │
│  │证书1│ │证书2│ │证书3│                                    │
│  └─────┘ └─────┘ └─────┘                                    │
├─────────────────────────────────────────────────────────────┤
│  📚 精选案例                                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 案例标题: 八字看婚姻走向                              │    │
│  │ 案例摘要: 此命局...                                   │    │
│  │ 👁 1,234 | 👍 89                                     │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│  💬 用户评价                                                  │
│  评价标签: 解读准确(156) 态度友好(143) 回复及时(98)            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ 用户A: ⭐⭐⭐⭐⭐ 大师解读非常准确...                  │    │
│  │ 用户B: ⭐⭐⭐⭐⭐ 服务态度很好...                      │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│  📖 个人介绍                                                  │
│  从业 15 年，师从 XXX 大师...                                 │
│  擅长: 婚姻感情、事业财运...                                   │
│  工作时间: 每日 9:00-21:00                                    │
├─────────────────────────────────────────────────────────────┤
│              [立即咨询] [预约服务]                             │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 前端组件清单

| 组件名 | 功能 | 数据来源 |
|--------|------|----------|
| `ProviderHeader` | 头部信息展示 | Provider + Profile |
| `SkillTagCloud` | 技能标签云 | SkillTags |
| `StatsPanel` | 数据统计面板 | ProviderStatsSummary |
| `ServicePackageList` | 服务套餐列表 | Packages |
| `CertificateGallery` | 资质证书展示 | Certificates |
| `PortfolioSection` | 作品集展示 | Portfolios |
| `ReviewSection` | 评价展示 | Reviews + ReviewTagStats |
| `ProfileIntro` | 个人介绍 | Profile |
| `ActionButtons` | 操作按钮 | - |

---

## 8. 数据查询接口（RPC/Subsquid）

### 8.1 链上查询

```rust
// Runtime API 扩展
pub trait DivinationMarketApi<AccountId> {
    /// 获取提供者完整主页数据
    fn get_provider_profile_full(provider: AccountId) -> Option<FullProviderProfile>;

    /// 获取提供者所有证书
    fn get_provider_certificates(provider: AccountId) -> Vec<Certificate>;

    /// 获取提供者所有作品
    fn get_provider_portfolios(provider: AccountId) -> Vec<PortfolioItem>;

    /// 获取提供者统计摘要
    fn get_provider_stats(provider: AccountId) -> Option<ProviderStatsSummary>;
}
```

### 8.2 Subsquid 查询（推荐）

```graphql
type ProviderProfile @entity {
  id: ID!
  provider: Provider!
  introductionCid: String
  experienceYears: Int
  background: String
  motto: String
  expertiseDescription: String
  workingHours: String
  avgResponseTime: Int
  acceptsAppointment: Boolean
  bannerCid: String
  updatedAt: BigInt
}

type Certificate @entity {
  id: ID!
  provider: Provider!
  name: String!
  certType: CertificateType!
  issuer: String
  imageCid: String!
  issuedAt: BigInt
  isVerified: Boolean!
  uploadedAt: BigInt!
}

type PortfolioItem @entity {
  id: ID!
  provider: Provider!
  title: String!
  divinationType: DivinationType!
  caseType: PortfolioCaseType!
  contentCid: String!
  coverCid: String
  isFeatured: Boolean!
  viewCount: Int!
  likeCount: Int!
  publishedAt: BigInt!
}

# 聚合查询
query GetProviderFullProfile($providerId: ID!) {
  provider(id: $providerId) {
    id
    name
    bio
    avatarCid
    tier
    status
    totalOrders
    completedOrders
    averageRating

    profile {
      introductionCid
      experienceYears
      motto
      workingHours
    }

    certificates(orderBy: uploadedAt_DESC) {
      id
      name
      certType
      isVerified
      imageCid
    }

    portfolios(orderBy: publishedAt_DESC, first: 10) {
      id
      title
      divinationType
      coverCid
      likeCount
      viewCount
    }

    packages(where: { isActive_eq: true }) {
      id
      name
      price
      divinationType
      salesCount
    }

    reviews(orderBy: createdAt_DESC, first: 20) {
      overallRating
      contentCid
      isAnonymous
      createdAt
    }
  }
}
```

---

## 9. 实施计划

### 阶段一：基础数据结构（1-2天）
1. 在 `types.rs` 添加新类型定义
2. 在 `lib.rs` 添加存储项
3. 添加配置常量

### 阶段二：核心功能实现（2-3天）
1. 实现 `update_profile` 函数
2. 实现资质证书管理函数
3. 实现作品集管理函数
4. 实现技能标签管理函数

### 阶段三：事件与错误（0.5天）
1. 添加新事件
2. 添加新错误类型

### 阶段四：测试（1-2天）
1. 编写单元测试
2. 集成测试

### 阶段五：前端开发（3-5天）
1. 设计个人主页 UI
2. 实现各组件
3. 对接 Subsquid 查询

### 阶段六：Subsquid 适配（1-2天）
1. 更新 schema
2. 添加事件处理器
3. 测试查询

---

## 10. 注意事项

1. **隐私保护**：联系方式等敏感信息使用 IPFS 加密存储
2. **内容审核**：作品集内容需要脱敏处理，避免泄露客户隐私
3. **存储优化**：大量文本内容存储在 IPFS，链上只存 CID
4. **性能考虑**：复杂查询通过 Subsquid 处理，减轻链上压力
5. **向后兼容**：新功能为可选扩展，不影响现有提供者

---

## 11. 总结

本方案通过扩展现有 `Provider` 结构，增加：
- 详细个人资料（`ProviderProfile`）
- 资质证书系统（`Certificate`）
- 作品集展示（`PortfolioItem`）
- 技能标签（`SkillTag`）
- 统计摘要（`ProviderStatsSummary`）

实现一个功能完善的服务提供者个人主页系统，帮助用户全面了解服务提供者，提升平台信任度和转化率。
