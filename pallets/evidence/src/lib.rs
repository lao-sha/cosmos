#![cfg_attr(not(feature = "std"), no_std)]
// 函数级中文注释：允许未使用的导入（trait方法调用）
#![allow(unused_imports)]

extern crate alloc;

pub use pallet::*;
use sp_core::Get;

// 函数级中文注释：导入log用于记录自动pin失败的警告
extern crate log;
// 函数级中文注释：导入pallet_memo_ipfs用于IpfsPinner trait
extern crate pallet_stardust_ipfs;
use pallet_stardust_ipfs::IpfsPinner;

// 函数级中文注释：权重模块导入，提供 WeightInfo 接口用于基于输入规模计算交易权重。
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod private_content;
pub mod weights;
// L-4修复：CID加密验证模块
pub mod cid_validator;

#[allow(deprecated)]
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::{
        private_content::{EncryptedKeyBundles, UserPublicKey},
        weights::WeightInfo,
    };
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;
    use frame_support::{pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_core::blake2_256;
    use sp_core::H256;
    use sp_runtime::traits::{Saturating, AtLeast32BitUnsigned};
    // 导入共享媒体工具库
    use media_utils::{
        HashHelper, IpfsHelper, MediaError
    };
    // 导入CID验证器trait
    use crate::cid_validator::{CidValidator, DefaultCidValidator};

    /// Phase 1.5优化：证据内容类型枚举
    /// 
    /// 函数级中文注释：标识证据的内容类型
    /// - 用于前端渲染和验证
    /// - 支持单一类型和混合类型
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Debug)]
    pub enum ContentType {
        /// 图片证据（单张或多张）
        Image,
        /// 视频证据（单个或多个）
        Video,
        /// 文档证据（单个或多个）
        Document,
        /// 混合类型（图片+视频+文档）
        Mixed,
        /// 纯文本描述
        Text,
    }

    /// Phase 1.5优化：共享证据记录结构（CID化版本）
    /// 
    /// 函数级详细中文注释：
    /// **核心优化**：
    /// - 旧版：链上存储所有CID数组（imgs, vids, docs）
    /// - 新版：链上只存储单一content_cid，实际内容存IPFS
    /// 
    /// **存储成本对比**：
    /// - 旧版：840字节（10张图片）
    /// - 新版：214字节（仅元数据+CID引用）
    /// - **降低74.5%** ⭐
    /// 
    /// **IPFS内容格式**（JSON）：
    /// ```json
    /// {
    ///   "version": "1.0",
    ///   "evidence_id": 123,
    ///   "domain": 2,
    ///   "target_id": 456,
    ///   "content": {
    ///     "images": ["QmXxx1", "QmXxx2", ...],
    ///     "videos": ["QmYyy1", ...],
    ///     "documents": ["QmZzz1", ...],
    ///     "memo": "可选文字说明"
    ///   },
    ///   "metadata": {
    ///     "created_at": 1234567890,
    ///     "owner": "5GrwvaEF...",
    ///     "encryption": {
    ///       "enabled": true,
    ///       "scheme": "aes256-gcm",
    ///       "key_bundles": {...}
    ///     }
    ///   }
    /// }
    /// ```
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(MaxContentCidLen, MaxSchemeLen))]
    pub struct Evidence<
        AccountId,
        BlockNumber,
        MaxContentCidLen: Get<u32>,
        MaxSchemeLen: Get<u32>,
    > {
        /// 证据唯一ID
        pub id: u64,
        /// 所属域（1=Grave, 2=Deceased, etc.）
        pub domain: u8,
        /// 目标ID（如deceased_id）
        pub target_id: u64,
        /// 证据所有者
        pub owner: AccountId,
        
        /// Phase 1.5优化：核心字段 - IPFS内容CID
        /// - 指向IPFS上的JSON文件
        /// - 包含所有图片/视频/文档的CID数组
        /// - 链上只存64字节CID引用
        pub content_cid: BoundedVec<u8, MaxContentCidLen>,
        
        /// Phase 1.5优化：内容类型标识
        /// - 便于前端快速识别和渲染
        /// - 无需下载IPFS内容即可知道类型
        pub content_type: ContentType,
        
        /// 创建时间（区块号）
        pub created_at: BlockNumber,
        
        /// Phase 1.5优化：加密标识
        /// - true: content_cid指向的内容已加密
        /// - false: 公开内容
        pub is_encrypted: bool,
        
        /// Phase 1.5优化：加密方案描述（可选）
        /// - 例如："aes256-gcm", "xchacha20-poly1305"
        /// - 用于解密时选择正确的算法
        pub encryption_scheme: Option<BoundedVec<u8, MaxSchemeLen>>,
        
        /// 证据承诺（commit），例如 H(ns || subject_id || cid_enc || salt || ver)
        pub commit: Option<H256>,
        
        /// 命名空间（8字节），用于授权与分域检索
        pub ns: Option<[u8; 8]>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + TypeInfo + core::fmt::Debug {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        // Phase 1.5优化：新的泛型参数（CID化版本）
        /// 内容CID最大长度（IPFS CID，建议64字节）
        #[pallet::constant]
        type MaxContentCidLen: Get<u32>;
        /// 加密方案描述最大长度（建议32字节）
        #[pallet::constant]
        type MaxSchemeLen: Get<u32>;
        
        // 旧版泛型参数（保留以向后兼容旧API）
        #[pallet::constant]
        type MaxCidLen: Get<u32>;
        #[pallet::constant]
        type MaxImg: Get<u32>;
        #[pallet::constant]
        type MaxVid: Get<u32>;
        #[pallet::constant]
        type MaxDoc: Get<u32>;
        #[pallet::constant]
        type MaxMemoLen: Get<u32>;
        #[pallet::constant]
        type MaxAuthorizedUsers: Get<u32>;
        #[pallet::constant]
        type MaxKeyLen: Get<u32>;
        #[pallet::constant]
        type EvidenceNsBytes: Get<[u8; 8]>;
        type Authorizer: EvidenceAuthorizer<Self::AccountId>;
        #[pallet::constant]
        type MaxPerSubjectTarget: Get<u32>;
        #[pallet::constant]
        type MaxPerSubjectNs: Get<u32>;
        #[pallet::constant]
        type WindowBlocks: Get<BlockNumberFor<Self>>;
        #[pallet::constant]
        type MaxPerWindow: Get<u32>;
        #[pallet::constant]
        type EnableGlobalCidDedup: Get<bool>;
        #[pallet::constant]
        type MaxListLen: Get<u32>;
        type WeightInfo: WeightInfo;
        type FamilyVerifier: FamilyRelationVerifier<Self::AccountId>;
        
        // ============= IPFS自动Pin相关配置 =============
        /// 函数级详细中文注释：IPFS自动pin提供者，供证据CID自动固定
        /// 
        /// 集成目标：
        /// - imgs: 图片证据CID列表自动pin
        /// - vids: 视频证据CID列表自动pin
        /// - docs: 文档证据CID列表自动pin
        /// 
        /// 使用场景：
        /// - commit: 提交证据时自动pin所有CID
        /// 
        /// 注意：
        /// - 证据通常关联到deceased_id（通过target_id）
        /// - 由Runtime注入实现（pallet_stardust_ipfs::Pallet<Runtime>）
        type IpfsPinner: pallet_stardust_ipfs::IpfsPinner<Self::AccountId, Self::Balance>;
        
        /// 函数级中文注释：余额类型（用于IPFS存储费用支付）
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        
        /// 函数级中文注释：默认IPFS存储单价（每副本每月）
        #[pallet::constant]
        type DefaultStoragePrice: Get<Self::Balance>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type NextEvidenceId<T: Config> = StorageValue<_, u64, ValueQuery>;
    #[pallet::storage]
    pub type Evidences<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        Evidence<T::AccountId, BlockNumberFor<T>, T::MaxContentCidLen, T::MaxSchemeLen>,
        OptionQuery,
    >;
    #[pallet::storage]
    pub type EvidenceByTarget<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, (u8, u64), Blake2_128Concat, u64, (), OptionQuery>;

    /// 新增：按命名空间+主体键值引用证据 id（便于按 ns/subject_id 聚合）
    #[pallet::storage]
    pub type EvidenceByNs<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ([u8; 8], u64),
        Blake2_128Concat,
        u64,
        (),
        OptionQuery,
    >;

    /// 新增：承诺哈希到 EvidenceId 的唯一索引，防止重复提交
    #[pallet::storage]
    pub type CommitIndex<T: Config> = StorageMap<_, Blake2_128Concat, H256, u64, OptionQuery>;

    /// 函数级中文注释：Plain 模式全局 CID 去重索引（可选）。
    /// - key 为 blake2_256(cid)；value 为 EvidenceId（首次出现的记录）。
    #[pallet::storage]
    pub type CidHashIndex<T: Config> = StorageMap<_, Blake2_128Concat, H256, u64, OptionQuery>;

    /// 函数级中文注释：每主体（domain,target）下的证据提交计数（链接操作不计数）。
    #[pallet::storage]
    pub type EvidenceCountByTarget<T: Config> =
        StorageMap<_, Blake2_128Concat, (u8, u64), u32, ValueQuery>;

    /// 函数级中文注释：每主体（ns,subject_id）下的证据提交计数（commit_hash 路径）。
    #[pallet::storage]
    pub type EvidenceCountByNs<T: Config> =
        StorageMap<_, Blake2_128Concat, ([u8; 8], u64), u32, ValueQuery>;

    /// 函数级中文注释：账户限频窗口存储（窗口起点与计数）。
    #[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo, MaxEncodedLen, Default)]
    pub struct WindowInfo<BlockNumber> {
        pub window_start: BlockNumber,
        pub count: u32,
    }
    #[pallet::storage]
    pub type AccountWindows<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, WindowInfo<BlockNumberFor<T>>, ValueQuery>;

    // === 私密内容存储 ===

    /// 私密内容序列号
    #[pallet::storage]
    pub type NextPrivateContentId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 私密内容主存储
    #[pallet::storage]
    pub type PrivateContents<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, private_content::PrivateContent<T>, OptionQuery>;

    /// 按CID索引私密内容（支持去重和快速查找）
    #[pallet::storage]
    pub type PrivateContentByCid<T: Config> =
        StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxCidLen>, u64, OptionQuery>;

    /// 按主体索引私密内容
    #[pallet::storage]
    pub type PrivateContentBySubject<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ([u8; 8], u64), // (ns, subject_id)
        Blake2_128Concat,
        u64, // content_id
        (),
        OptionQuery,
    >;

    /// 用户公钥存储
    #[pallet::storage]
    pub type UserPublicKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserPublicKey<T>, OptionQuery>;

    /// 密钥轮换历史
    #[pallet::storage]
    pub type KeyRotationHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64, // content_id
        Blake2_128Concat,
        u32, // rotation_round
        private_content::KeyRotationRecord<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        EvidenceCommitted {
            id: u64,
            domain: u8,
            target_id: u64,
            owner: T::AccountId,
        },
        EvidenceLinked {
            domain: u8,
            target_id: u64,
            id: u64,
        },
        EvidenceUnlinked {
            domain: u8,
            target_id: u64,
            id: u64,
        },
        /// 新增：V2 事件，按命名空间与主体提交/链接
        EvidenceCommittedV2 {
            id: u64,
            ns: [u8; 8],
            subject_id: u64,
            owner: T::AccountId,
        },
        EvidenceLinkedV2 {
            ns: [u8; 8],
            subject_id: u64,
            id: u64,
        },
        EvidenceUnlinkedV2 {
            ns: [u8; 8],
            subject_id: u64,
            id: u64,
        },
        /// 函数级中文注释：因限频或配额被限制（便于前端提示）。
        EvidenceThrottled(T::AccountId, u8 /*reason_code: 1=RateLimited,2=Quota*/),
        /// 函数级中文注释：达到主体配额上限。
        EvidenceQuotaReached(
            u8,  /*0=target,1=ns*/
            u64, /*subject_id or target_id*/
        ),

        // === 私密内容事件 ===
        /// 私密内容已存储
        PrivateContentStored {
            content_id: u64,
            ns: [u8; 8],
            subject_id: u64,
            cid: BoundedVec<u8, T::MaxCidLen>,
            creator: T::AccountId,
        },

        /// 访问权限已授予
        AccessGranted {
            content_id: u64,
            user: T::AccountId,
            granted_by: T::AccountId,
        },

        /// 访问权限已撤销
        AccessRevoked {
            content_id: u64,
            user: T::AccountId,
            revoked_by: T::AccountId,
        },

        /// 密钥已轮换
        KeysRotated {
            content_id: u64,
            rotation_round: u32,
            rotated_by: T::AccountId,
        },

        /// 用户公钥已注册
        PublicKeyRegistered {
            user: T::AccountId,
            key_type: u8,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 权限不足（命名空间或账户不被授权）
        NotAuthorized,
        /// 未找到目标对象
        NotFound,

        // === 私密内容错误 ===
        /// 私密内容未找到
        PrivateContentNotFound,
        /// 用户公钥未注册
        PublicKeyNotRegistered,
        /// 无权访问此内容
        AccessDenied,
        /// CID已存在（去重检查）
        CidAlreadyExists,
        /// 授权用户数量过多
        TooManyAuthorizedUsers,
        /// 无效的加密密钥格式
        InvalidEncryptedKey,
        /// 家庭关系验证失败
        FamilyVerificationFailed,
        /// 密钥类型不支持
        UnsupportedKeyType,
        /// 图片数量超过上限
        TooManyImages,
        /// 视频数量超过上限
        TooManyVideos,
        /// 文档数量超过上限
        TooManyDocs,
        /// CID 长度或格式非法（非可见 ASCII 或为空）
        InvalidCidFormat,
        /// 发现重复的 CID 输入
        DuplicateCid,
        /// 提交的承诺已存在（防重）
        CommitAlreadyExists,
        /// 证据命名空间与当前操作命名空间不匹配
        NamespaceMismatch,
        /// 账号在窗口内达到提交上限
        RateLimited,
        /// 该主体已达到最大证据条数
        TooManyForSubject,
        /// 全局 CID 去重命中（Plain 模式）
        DuplicateCidGlobal,
    }

    #[allow(deprecated)]
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 函数级中文注释：提交证据，生成 EvidenceId 并落库；仅授权账户可提交。
        #[pallet::call_index(0)]
        #[allow(deprecated)]
        #[pallet::weight(T::WeightInfo::commit(imgs.len() as u32, vids.len() as u32, docs.len() as u32))]
        pub fn commit(
            origin: OriginFor<T>,
            domain: u8,
            target_id: u64,
            imgs: Vec<BoundedVec<u8, T::MaxCidLen>>,
            vids: Vec<BoundedVec<u8, T::MaxCidLen>>,
            docs: Vec<BoundedVec<u8, T::MaxCidLen>>,
            _memo: Option<BoundedVec<u8, T::MaxMemoLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // Authorizer 鉴权（通过适配器，解耦到 runtime）
            let ns = T::EvidenceNsBytes::get();
            ensure!(
                <T as Config>::Authorizer::is_authorized(ns, &who),
                Error::<T>::NotAuthorized
            );
            // 限频与配额
            let now = <frame_system::Pallet<T>>::block_number();
            Self::touch_window(&who, now)?;
            let cnt = EvidenceCountByTarget::<T>::get((domain, target_id));
            ensure!(
                cnt < T::MaxPerSubjectTarget::get(),
                Error::<T>::TooManyForSubject
            );
            // 校验 CID（长度/格式/重复）与数量上限
            Self::validate_cid_vec(&imgs)?;
            Self::validate_cid_vec(&vids)?;
            Self::validate_cid_vec(&docs)?;
            // 可选全局去重
            Self::ensure_global_cid_unique([&imgs, &vids, &docs])?;
            
            let id = NextEvidenceId::<T>::mutate(|n| {
                let id = *n;
                *n = n.saturating_add(1);
                id
            });
            
            // TODO: Phase 1.5 完整实施 - 将 imgs/vids/docs 打包为JSON上传IPFS，返回content_cid
            // 临时方案：使用第一个img的CID作为content_cid（需要类型转换）
            let temp_vec: Vec<u8> = if !imgs.is_empty() {
                imgs[0].clone().into_inner()
            } else if !vids.is_empty() {
                vids[0].clone().into_inner()
            } else if !docs.is_empty() {
                docs[0].clone().into_inner()
            } else {
                b"QmPlaceholder".to_vec()
            };
            let content_cid: BoundedVec<u8, T::MaxContentCidLen> = temp_vec.try_into()
                .map_err(|_| Error::<T>::InvalidCidFormat)?;
            
            let ev = Evidence {
                id,
                domain,
                target_id,
                owner: who.clone(),
                content_cid,
                content_type: ContentType::Mixed, // 临时使用Mixed类型
                created_at: now,
                is_encrypted: false, // 临时假设不加密
                encryption_scheme: None,
                commit: None,
                ns: Some(ns),
            };
            Evidences::<T>::insert(id, &ev);
            EvidenceByTarget::<T>::insert((domain, target_id), id, ());
            // 计数 + 去重索引落库
            EvidenceCountByTarget::<T>::insert((domain, target_id), cnt.saturating_add(1));
            
            // TODO: Phase 1.5 完整实施 - 从 content_cid 指向的JSON解析出所有CID进行去重和pin
            // 临时方案：对当前的content_cid进行去重和pin
            if T::EnableGlobalCidDedup::get() {
                let h = H256::from(blake2_256(&ev.content_cid.clone().into_inner()));
                if CidHashIndex::<T>::get(h).is_none() {
                    CidHashIndex::<T>::insert(h, id);
                }
            }

            // 函数级详细中文注释：自动pin证据CID到IPFS
            // TODO: Phase 1.5 完整实施 - pin content_cid及其包含的所有媒体CID
            // 临时方案：只pin content_cid本身
            let deceased_id_u64 = target_id;
            let cid_vec: Vec<u8> = ev.content_cid.clone().into_inner();
            if let Err(e) = T::IpfsPinner::pin_cid_for_deceased(
                who.clone(),
                deceased_id_u64,
                cid_vec,
                None, // 使用默认Standard层级（3副本）
            ) {
                log::warn!(
                    target: "evidence",
                    "Auto-pin content cid failed for evidence {:?}: {:?}",
                    id,
                    e
                );
            }
            
            // 只读方法移至模块外部以避免 non_local_definitions 警告在 -D warnings 下被提升为错误。
            Self::deposit_event(Event::EvidenceCommitted {
                id,
                domain,
                target_id,
                owner: who,
            });
            Ok(())
        }

        /// 函数级中文注释（V2）：仅登记承诺哈希（不在链上存储任何明文/可逆 CID）。
        /// - ns：8 字节命名空间（如 b"kyc_____"、b"otc_ord_"）。
        /// - subject_id：业务主体 id（如订单号、账户短码等）。
        /// - commit：承诺哈希（例如 blake2b256(ns||subject_id||cid_enc||salt||ver)）。
        #[pallet::call_index(1)]
        #[allow(deprecated)]
        #[pallet::weight(T::WeightInfo::commit_hash())]
        pub fn commit_hash(
            origin: OriginFor<T>,
            ns: [u8; 8],
            subject_id: u64,
            commit: H256,
            memo: Option<BoundedVec<u8, T::MaxMemoLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                <T as Config>::Authorizer::is_authorized(ns, &who),
                Error::<T>::NotAuthorized
            );
            // 防重：承诺哈希唯一
            ensure!(
                CommitIndex::<T>::get(commit).is_none(),
                Error::<T>::CommitAlreadyExists
            );
            // 限频与配额
            let now = <frame_system::Pallet<T>>::block_number();
            Self::touch_window(&who, now)?;
            let cnt = EvidenceCountByNs::<T>::get((ns, subject_id));
            ensure!(
                cnt < T::MaxPerSubjectNs::get(),
                Error::<T>::TooManyForSubject
            );
            let id = NextEvidenceId::<T>::mutate(|n| {
                let id = *n;
                *n = n.saturating_add(1);
                id
            });
            // TODO: Phase 1.5 完整实施 - 从memo或其他来源获取content_cid
            // 临时方案：转换memo为content_cid类型
            let temp_vec2: Vec<u8> = if let Some(ref m) = memo {
                m.clone().into_inner()
            } else {
                b"QmPlaceholder2".to_vec()
            };
            let content_cid: BoundedVec<u8, T::MaxContentCidLen> = temp_vec2.try_into()
                .map_err(|_| Error::<T>::InvalidCidFormat)?;
            
            let ev = Evidence {
                id,
                domain: 0,
                target_id: subject_id,
                owner: who.clone(),
                content_cid,
                content_type: ContentType::Mixed,
                created_at: now,
                is_encrypted: false,
                encryption_scheme: None,
                commit: Some(commit),
                ns: Some(ns),
            };
            Evidences::<T>::insert(id, &ev);
            EvidenceByNs::<T>::insert((ns, subject_id), id, ());
            CommitIndex::<T>::insert(commit, id);
            EvidenceCountByNs::<T>::insert((ns, subject_id), cnt.saturating_add(1));
            Self::deposit_event(Event::EvidenceCommittedV2 {
                id,
                ns,
                subject_id,
                owner: who,
            });
            Ok(())
        }

        /// 函数级中文注释：为目标链接已存在的证据（允许复用）；仅授权账户可调用。
        #[pallet::call_index(2)]
        #[allow(deprecated)]
        #[pallet::weight(T::WeightInfo::link())]
        pub fn link(origin: OriginFor<T>, domain: u8, target_id: u64, id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let ev = Evidences::<T>::get(id).ok_or(Error::<T>::NotFound)?;
            let ev_ns = ev.ns.ok_or(Error::<T>::NamespaceMismatch)?;
            ensure!(
                <T as Config>::Authorizer::is_authorized(ev_ns, &who),
                Error::<T>::NotAuthorized
            );
            EvidenceByTarget::<T>::insert((domain, target_id), id, ());
            Self::deposit_event(Event::EvidenceLinked {
                domain,
                target_id,
                id,
            });
            Ok(())
        }

        /// 函数级中文注释（V2）：按命名空间与主体链接既有证据 id（仅保存引用，不触碰明文）。
        #[pallet::call_index(3)]
        #[allow(deprecated)]
        #[pallet::weight(T::WeightInfo::link_by_ns())]
        pub fn link_by_ns(
            origin: OriginFor<T>,
            ns: [u8; 8],
            subject_id: u64,
            id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                <T as Config>::Authorizer::is_authorized(ns, &who),
                Error::<T>::NotAuthorized
            );
            let ev = Evidences::<T>::get(id).ok_or(Error::<T>::NotFound)?;
            let ev_ns = ev.ns.ok_or(Error::<T>::NamespaceMismatch)?;
            ensure!(ev_ns == ns, Error::<T>::NamespaceMismatch);
            EvidenceByNs::<T>::insert((ns, subject_id), id, ());
            Self::deposit_event(Event::EvidenceLinkedV2 { ns, subject_id, id });
            Ok(())
        }

        /// 函数级中文注释：取消目标与证据的链接；仅授权账户可调用。
        #[pallet::call_index(4)]
        #[allow(deprecated)]
        #[pallet::weight(T::WeightInfo::unlink())]
        pub fn unlink(origin: OriginFor<T>, domain: u8, target_id: u64, id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let ev = Evidences::<T>::get(id).ok_or(Error::<T>::NotFound)?;
            let ev_ns = ev.ns.ok_or(Error::<T>::NamespaceMismatch)?;
            ensure!(
                <T as Config>::Authorizer::is_authorized(ev_ns, &who),
                Error::<T>::NotAuthorized
            );
            EvidenceByTarget::<T>::remove((domain, target_id), id);
            Self::deposit_event(Event::EvidenceUnlinked {
                domain,
                target_id,
                id,
            });
            Ok(())
        }

        /// 函数级中文注释（V2）：按命名空间与主体取消链接。
        #[pallet::call_index(5)]
        #[allow(deprecated)]
        #[pallet::weight(T::WeightInfo::unlink_by_ns())]
        pub fn unlink_by_ns(
            origin: OriginFor<T>,
            ns: [u8; 8],
            subject_id: u64,
            id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                <T as Config>::Authorizer::is_authorized(ns, &who),
                Error::<T>::NotAuthorized
            );
            let ev = Evidences::<T>::get(id).ok_or(Error::<T>::NotFound)?;
            let ev_ns = ev.ns.ok_or(Error::<T>::NamespaceMismatch)?;
            ensure!(ev_ns == ns, Error::<T>::NamespaceMismatch);
            EvidenceByNs::<T>::remove((ns, subject_id), id);
            Self::deposit_event(Event::EvidenceUnlinkedV2 { ns, subject_id, id });
            Ok(())
        }

        // ===== 私密内容管理 Extrinsics =====

        /// 注册用户公钥（用于加密密钥包）
        #[pallet::call_index(6)]
        #[pallet::weight(10_000)] // TODO: 使用WeightInfo
        pub fn register_public_key(
            origin: OriginFor<T>,
            key_data: BoundedVec<u8, T::MaxKeyLen>,
            key_type: u8, // 1=RSA-2048, 2=Ed25519, 3=ECDSA-P256
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证密钥类型
            ensure!(
                key_type >= 1 && key_type <= 3,
                Error::<T>::UnsupportedKeyType
            );

            // 验证密钥长度
            match key_type {
                1 => ensure!(
                    key_data.len() >= 270 && key_data.len() <= 512,
                    Error::<T>::InvalidEncryptedKey
                ), // RSA-2048 DER
                2 => ensure!(key_data.len() == 32, Error::<T>::InvalidEncryptedKey), // Ed25519
                3 => ensure!(
                    key_data.len() == 33 || key_data.len() == 65,
                    Error::<T>::InvalidEncryptedKey
                ), // ECDSA
                _ => return Err(Error::<T>::UnsupportedKeyType.into()),
            }

            let now = <frame_system::Pallet<T>>::block_number();

            let public_key = UserPublicKey::<T> {
                key_data,
                key_type,
                registered_at: now,
            };

            UserPublicKeys::<T>::insert(&who, &public_key);

            Self::deposit_event(Event::PublicKeyRegistered {
                user: who,
                key_type,
            });

            Ok(())
        }

        /// 存储私密内容
        #[pallet::call_index(7)]
        #[pallet::weight(10_000)] // TODO: 使用WeightInfo
        pub fn store_private_content(
            origin: OriginFor<T>,
            ns: [u8; 8],
            subject_id: u64,
            cid: BoundedVec<u8, T::MaxCidLen>,
            content_hash: H256,
            encryption_method: u8,
            access_policy: private_content::AccessPolicy<T>,
            encrypted_keys: EncryptedKeyBundles<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 权限检查
            ensure!(
                <T as Config>::Authorizer::is_authorized(ns, &who),
                Error::<T>::NotAuthorized
            );

            // 私密内容必须使用加密CID验证
            let cid_bytes: &[u8] = cid.as_slice();
            ensure!(
                crate::cid_validator::DefaultCidValidator::is_encrypted(cid_bytes),
                Error::<T>::InvalidCidFormat
            );

            // CID格式验证（使用IPFS规范）
            let cid_str = core::str::from_utf8(cid_bytes)
                .map_err(|_| Error::<T>::InvalidCidFormat)?;
            IpfsHelper::validate_cid(cid_str)
                .map_err(|_| Error::<T>::InvalidCidFormat)?;

            // CID 去重检查
            ensure!(
                PrivateContentByCid::<T>::get(&cid).is_none(),
                Error::<T>::CidAlreadyExists
            );

            // 验证创建者是否有加密密钥
            ensure!(
                encrypted_keys.iter().any(|(user, _)| user == &who),
                Error::<T>::InvalidEncryptedKey
            );

            // 验证所有用户都已注册公钥
            for (user, _) in encrypted_keys.iter() {
                ensure!(
                    UserPublicKeys::<T>::contains_key(user),
                    Error::<T>::PublicKeyNotRegistered
                );
            }

            // 家庭成员访问策略验证
            if let private_content::AccessPolicy::FamilyMembers(deceased_id) = &access_policy {
                ensure!(
                    T::FamilyVerifier::is_authorized_for_deceased(&who, *deceased_id),
                    Error::<T>::FamilyVerificationFailed
                );
            }

            let content_id = NextPrivateContentId::<T>::mutate(|id| {
                let current = *id;
                *id = id.saturating_add(1);
                current
            });

            let now = <frame_system::Pallet<T>>::block_number();

            let content = private_content::PrivateContent {
                id: content_id,
                ns,
                subject_id,
                cid: cid.clone(),
                content_hash,
                encryption_method,
                creator: who.clone(),
                access_policy,
                encrypted_keys,
                created_at: now,
                updated_at: now,
            };

            // 存储
            PrivateContents::<T>::insert(content_id, &content);
            PrivateContentByCid::<T>::insert(&cid, content_id);
            PrivateContentBySubject::<T>::insert((ns, subject_id), content_id, ());

            Self::deposit_event(Event::PrivateContentStored {
                content_id,
                ns,
                subject_id,
                cid,
                creator: who,
            });

            Ok(())
        }

        /// 授予用户访问权限
        #[pallet::call_index(8)]
        #[pallet::weight(10_000)] // TODO: 使用WeightInfo
        pub fn grant_access(
            origin: OriginFor<T>,
            content_id: u64,
            user: T::AccountId,
            encrypted_key: BoundedVec<u8, ConstU32<512>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证用户已注册公钥
            ensure!(
                UserPublicKeys::<T>::contains_key(&user),
                Error::<T>::PublicKeyNotRegistered
            );

            PrivateContents::<T>::try_mutate(content_id, |maybe_content| -> DispatchResult {
                let content = maybe_content
                    .as_mut()
                    .ok_or(Error::<T>::PrivateContentNotFound)?;

                // 权限检查：仅创建者可授予权限
                ensure!(content.creator == who, Error::<T>::AccessDenied);

                // 检查是否已授权
                let key_vec: Vec<u8> = encrypted_key.into();
                let bounded_key = BoundedVec::try_from(key_vec.clone())
                    .map_err(|_| Error::<T>::InvalidEncryptedKey)?;
                let mut found = false;
                for (existing_user, existing_key) in content.encrypted_keys.iter_mut() {
                    if existing_user == &user {
                        *existing_key = bounded_key.clone();
                        found = true;
                        break;
                    }
                }

                if !found {
                    content
                        .encrypted_keys
                        .try_push((user.clone(), bounded_key))
                        .map_err(|_| Error::<T>::TooManyAuthorizedUsers)?;
                }

                content.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::AccessGranted {
                    content_id,
                    user,
                    granted_by: who,
                });

                Ok(())
            })
        }

        /// 撤销用户访问权限
        #[pallet::call_index(9)]
        #[pallet::weight(10_000)] // TODO: 使用WeightInfo
        pub fn revoke_access(
            origin: OriginFor<T>,
            content_id: u64,
            user: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivateContents::<T>::try_mutate(content_id, |maybe_content| -> DispatchResult {
                let content = maybe_content
                    .as_mut()
                    .ok_or(Error::<T>::PrivateContentNotFound)?;

                // 权限检查
                ensure!(content.creator == who, Error::<T>::AccessDenied);
                ensure!(user != who, Error::<T>::AccessDenied); // 不能撤销自己的权限

                // 移除用户
                content
                    .encrypted_keys
                    .retain(|(existing_user, _)| existing_user != &user);
                content.updated_at = <frame_system::Pallet<T>>::block_number();

                Self::deposit_event(Event::AccessRevoked {
                    content_id,
                    user,
                    revoked_by: who,
                });

                Ok(())
            })
        }

        /// 轮换内容加密密钥
        #[pallet::call_index(10)]
        #[pallet::weight(10_000)] // TODO: 使用WeightInfo
        pub fn rotate_content_keys(
            origin: OriginFor<T>,
            content_id: u64,
            new_content_hash: H256, // 重新加密后的内容哈希
            new_encrypted_keys: BoundedVec<
                (T::AccountId, BoundedVec<u8, ConstU32<512>>),
                T::MaxAuthorizedUsers,
            >,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivateContents::<T>::try_mutate(content_id, |maybe_content| -> DispatchResult {
                let content = maybe_content
                    .as_mut()
                    .ok_or(Error::<T>::PrivateContentNotFound)?;

                // 权限检查
                ensure!(content.creator == who, Error::<T>::AccessDenied);

                // 验证所有用户都已注册公钥
                for (user, _) in new_encrypted_keys.iter() {
                    ensure!(
                        UserPublicKeys::<T>::contains_key(user),
                        Error::<T>::PublicKeyNotRegistered
                    );
                }

                // 更新内容
                content.content_hash = new_content_hash;
                let converted = new_encrypted_keys
                    .into_iter()
                    .map(|(u, k)| {
                        let key_vec: Vec<u8> = k.into();
                        BoundedVec::try_from(key_vec)
                            .map(|bk| (u, bk))
                            .map_err(|_| Error::<T>::InvalidEncryptedKey)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let bounded_converted = BoundedVec::try_from(converted)
                    .map_err(|_| Error::<T>::TooManyAuthorizedUsers)?;
                content.encrypted_keys = bounded_converted;
                content.updated_at = <frame_system::Pallet<T>>::block_number();

                // 记录轮换历史
                let rotation_round = KeyRotationHistory::<T>::iter_prefix(content_id)
                    .map(|(round, _)| round)
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1);

                let rotation_record = private_content::KeyRotationRecord {
                    content_id,
                    rotation_round,
                    rotated_at: content.updated_at,
                    rotated_by: who.clone(),
                };

                KeyRotationHistory::<T>::insert(content_id, rotation_round, &rotation_record);

                Self::deposit_event(Event::KeysRotated {
                    content_id,
                    rotation_round,
                    rotated_by: who,
                });

                Ok(())
            })
        }

        // 只读接口应放置在 inherent impl 中，而非 extrinsics 块。
    }

    /// 承诺哈希和验证工具函数
    impl<T: Config> Pallet<T> {
        /// 计算 Evidence 承诺哈希
        ///
        /// 使用 stardust-media-common 的 HashHelper 计算标准格式的承诺哈希:
        /// H(ns || subject_id || cid || salt || version)
        ///
        /// # 参数
        /// - `ns`: 8字节命名空间
        /// - `subject_id`: 主体ID
        /// - `cid`: IPFS CID数据
        /// - `salt`: 盐值
        /// - `version`: 版本号（通常为1）
        ///
        /// # 返回
        /// - 计算得到的 H256 承诺哈希
        pub fn compute_evidence_commitment(
            ns: &[u8; 8],
            subject_id: u64,
            cid: &[u8],
            salt: &[u8],
            version: u32,
        ) -> H256 {
            HashHelper::evidence_commitment(ns, subject_id, cid, salt, version)
        }

        /// 验证承诺哈希是否正确
        ///
        /// # 参数
        /// - `ns`: 8字节命名空间
        /// - `subject_id`: 主体ID
        /// - `cid`: IPFS CID数据
        /// - `salt`: 盐值
        /// - `version`: 版本号
        /// - `expected_commit`: 期望的承诺哈希
        ///
        /// # 返回
        /// - `true`: 验证通过
        /// - `false`: 验证失败
        pub fn verify_evidence_commitment(
            ns: &[u8; 8],
            subject_id: u64,
            cid: &[u8],
            salt: &[u8],
            version: u32,
            expected_commit: &H256,
        ) -> bool {
            let computed = Self::compute_evidence_commitment(ns, subject_id, cid, salt, version);
            &computed == expected_commit
        }

        /// 验证 CID 格式（单个）
        ///
        /// 使用 stardust-media-common 的 IpfsHelper 验证单个 CID 格式。
        pub fn validate_single_cid(cid: &[u8]) -> Result<(), Error<T>> {
            let cid_str = core::str::from_utf8(cid)
                .map_err(|_| Error::<T>::InvalidCidFormat)?;

            IpfsHelper::validate_cid(cid_str)
                .map_err(|_| Error::<T>::InvalidCidFormat)
        }

        /// 验证内容完整性
        ///
        /// 使用 stardust-media-common 的 IpfsHelper 验证内容与 CID 的对应关系。
        /// 注意：此函数仅在有实际内容数据时使用。
        pub fn verify_content_integrity(content_data: &[u8], cid: &str) -> bool {
            IpfsHelper::verify_content(content_data, cid)
        }
    }

    /// 授权适配接口：由 runtime 实现并桥接到 `pallet-authorizer`，以保持低耦合。
    pub trait EvidenceAuthorizer<AccountId> {
        /// 校验某账户是否在给定命名空间下被授权提交/链接证据
        fn is_authorized(ns: [u8; 8], who: &AccountId) -> bool;
    }

    /// 家庭关系验证接口
    pub trait FamilyRelationVerifier<AccountId> {
        /// 验证用户是否为指定逝者的家庭成员
        fn is_family_member(user: &AccountId, deceased_id: u64) -> bool;
        /// 验证用户是否为逝者的授权管理员
        fn is_authorized_for_deceased(user: &AccountId, deceased_id: u64) -> bool;
    }

    /// 只读查询 trait 占位：供其他 pallet 低耦合读取证据（可在 runtime 或外部实现）。
    pub trait EvidenceProvider<AccountId> {
        /// 返回指定 ID 的证据；本 Pallet 不提供默认实现，避免类型推断问题。
        fn get(_id: u64) -> Option<()>;
    }

    /// 私密内容查询接口 (供其他 pallet 使用)
    pub trait PrivateContentProvider<AccountId> {
        /// 检查用户是否可以访问指定的私密内容
        fn can_access(content_id: u64, user: &AccountId) -> bool;
        /// 获取用户的解密密钥
        fn get_decryption_key(content_id: u64, user: &AccountId) -> Option<Vec<u8>>;
    }

    impl<T: Config> Pallet<T> {
        // ===== 私密内容查询方法 =====

        /// 检查用户是否有访问特定私密内容的权限
        pub fn can_access_private_content(content_id: u64, user: &T::AccountId) -> bool {
            if let Some(content) = PrivateContents::<T>::get(content_id) {
                // 检查是否是创建者
                if &content.creator == user {
                    return true;
                }

                // 检查访问策略
                match &content.access_policy {
                    private_content::AccessPolicy::OwnerOnly => false,
                    private_content::AccessPolicy::SharedWith(users) => {
                        users.iter().any(|u| u == user)
                    }
                    private_content::AccessPolicy::FamilyMembers(deceased_id) => {
                        T::FamilyVerifier::is_family_member(user, *deceased_id)
                    }
                    private_content::AccessPolicy::TimeboxedAccess { users, expires_at } => {
                        let now = <frame_system::Pallet<T>>::block_number();
                        now <= *expires_at && users.iter().any(|u| u == user)
                    }
                    private_content::AccessPolicy::GovernanceControlled => {
                        // TODO: 实现治理权限检查
                        false
                    }
                    private_content::AccessPolicy::RoleBased(_role) => {
                        // TODO: 实现基于角色的权限检查
                        false
                    }
                }
            } else {
                false
            }
        }

        /// 获取用户的加密密钥包
        pub fn get_encrypted_key_for_user(
            content_id: u64,
            user: &T::AccountId,
        ) -> Option<BoundedVec<u8, T::MaxKeyLen>> {
            if let Some(content) = PrivateContents::<T>::get(content_id) {
                if Self::can_access_private_content(content_id, user) {
                    content
                        .encrypted_keys
                        .iter()
                        .find(|(u, _)| u == user)
                        .map(|(_, key)| key.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }

        /// 通过CID查找私密内容
        pub fn get_private_content_by_cid(
            cid: &BoundedVec<u8, T::MaxCidLen>,
        ) -> Option<private_content::PrivateContent<T>> {
            if let Some(content_id) = PrivateContentByCid::<T>::get(cid) {
                PrivateContents::<T>::get(content_id)
            } else {
                None
            }
        }

        /// 获取主体下的所有私密内容ID
        pub fn get_private_content_ids_by_subject(ns: [u8; 8], subject_id: u64) -> Vec<u64> {
            PrivateContentBySubject::<T>::iter_prefix((ns, subject_id))
                .map(|(content_id, _)| content_id)
                .collect()
        }

        /// 函数级中文注释：限频检查并计数。
        /// - 进入窗口：超过 WindowBlocks 自动滚动窗口并清零计数；严格小于最大次数方可提交。
        fn touch_window(who: &T::AccountId, now: BlockNumberFor<T>) -> Result<(), Error<T>> {
            AccountWindows::<T>::mutate(who, |w| {
                let wb = T::WindowBlocks::get();
                if now.saturating_sub(w.window_start) >= wb {
                    w.window_start = now;
                    w.count = 0;
                }
            });
            let info = AccountWindows::<T>::get(who);
            ensure!(info.count < T::MaxPerWindow::get(), Error::<T>::RateLimited);
            AccountWindows::<T>::mutate(who, |w| {
                w.count = w.count.saturating_add(1);
            });
            Ok(())
        }

        /// 函数级中文注释：校验一组 CID 的格式与去重要求。
        /// 规则：每个 CID 必须非空、符合IPFS格式规范；组内不得重复。
        /// 使用 stardust-media-common 的 IpfsHelper 进行规范验证。
        fn validate_cid_vec(list: &Vec<BoundedVec<u8, T::MaxCidLen>>) -> Result<(), Error<T>> {
            let mut set: BTreeSet<Vec<u8>> = BTreeSet::new();
            for cid in list.iter() {
                if cid.is_empty() {
                    return Err(Error::<T>::InvalidCidFormat);
                }

                // 转换为字符串进行IPFS规范验证
                let cid_str = core::str::from_utf8(cid.as_slice())
                    .map_err(|_| Error::<T>::InvalidCidFormat)?;

                // 使用 stardust-media-common 的 IpfsHelper 进行规范验证
                IpfsHelper::validate_cid(cid_str)
                    .map_err(|_| Error::<T>::InvalidCidFormat)?;

                // 检查重复
                let v: Vec<u8> = cid.clone().into_inner();
                if !set.insert(v) {
                    return Err(Error::<T>::DuplicateCid);
                }
            }
            Ok(())
        }

        /// 函数级中文注释：可选的全局 CID 去重检查（Plain 模式）。
        /// - EnableGlobalCidDedup=true 时，逐个 CID 计算 blake2_256 并查重；首次出现时在提交成功后写入索引。
        fn ensure_global_cid_unique(
            list_groups: [&Vec<BoundedVec<u8, T::MaxCidLen>>; 3],
        ) -> Result<(), Error<T>> {
            if !T::EnableGlobalCidDedup::get() {
                return Ok(());
            }
            for list in list_groups.into_iter() {
                for cid in list.iter() {
                    let h = H256::from(blake2_256(&cid.clone().into_inner()));
                    if CidHashIndex::<T>::get(h).is_some() {
                        return Err(Error::<T>::DuplicateCidGlobal);
                    }
                }
            }
            Ok(())
        }
    }
}

// ===== 只读方法（模块外部，避免 non_local_definitions）=====
impl<T: pallet::Config> Pallet<T> {
    /// 函数级中文注释：只读-按 (domain,target) 分页列出 evidence id（从 start_id 起，最多 MaxListLen 条）。
    pub fn list_ids_by_target(
        domain: u8,
        target_id: u64,
        start_id: u64,
        limit: u32,
    ) -> alloc::vec::Vec<u64> {
        let mut out: alloc::vec::Vec<u64> = alloc::vec::Vec::new();
        let mut cnt: u32 = 0;
        let cap = core::cmp::min(limit, T::MaxListLen::get());
        for id in pallet::EvidenceByTarget::<T>::iter_key_prefix((domain, target_id)) {
            if id < start_id {
                continue;
            }
            out.push(id);
            cnt = cnt.saturating_add(1);
            if cnt >= cap {
                break;
            }
        }
        out
    }

    /// 函数级中文注释：只读-按 (ns,subject_id) 分页列出 evidence id（从 start_id 起，最多 MaxListLen 条）。
    pub fn list_ids_by_ns(
        ns: [u8; 8],
        subject_id: u64,
        start_id: u64,
        limit: u32,
    ) -> alloc::vec::Vec<u64> {
        let mut out: alloc::vec::Vec<u64> = alloc::vec::Vec::new();
        let mut cnt: u32 = 0;
        let cap = core::cmp::min(limit, T::MaxListLen::get());
        for id in pallet::EvidenceByNs::<T>::iter_key_prefix((ns, subject_id)) {
            if id < start_id {
                continue;
            }
            out.push(id);
            cnt = cnt.saturating_add(1);
            if cnt >= cap {
                break;
            }
        }
        out
    }

    /// 函数级中文注释：只读-获取主体证据数量。
    pub fn count_by_target(domain: u8, target_id: u64) -> u32 {
        pallet::EvidenceCountByTarget::<T>::get((domain, target_id))
    }
    pub fn count_by_ns(ns: [u8; 8], subject_id: u64) -> u32 {
        pallet::EvidenceCountByNs::<T>::get((ns, subject_id))
    }
}
