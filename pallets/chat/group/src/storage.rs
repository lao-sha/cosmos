/// Stardust智能群聊系统 - 智能存储引擎
///
/// 实现四层存储架构和智能存储策略

use crate::types::*;
use codec::{Decode, Encode};
use frame_support::{
    pallet_prelude::*,
    traits::UnixTime,
};
use sp_std::{vec::Vec, convert::TryInto};

/// 智能存储引擎
pub struct SmartStorageEngine<T: frame_system::Config> {
    _phantom: sp_std::marker::PhantomData<T>,
}

impl<T: frame_system::Config> SmartStorageEngine<T> {
    /// 创建新的存储引擎实例
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }

    /// 智能存储决策
    pub fn determine_storage_strategy(
        content: &[u8],
        encryption_mode: EncryptionMode,
        message_type: MessageType,
        urgency: MessageUrgency,
        retention_policy: RetentionPolicy,
        group_size: u32,
    ) -> StorageStrategy<T> {
        let content_analysis = Self::analyze_content_characteristics(content);

        let storage_tier = match (content_analysis.size_category, encryption_mode, message_type, urgency) {
            // 小消息 + 透明模式 + 普通优先级 -> 链上存储
            (ContentSizeCategory::Small, EncryptionMode::Transparent, _, MessageUrgency::Normal) => {
                StorageTier::OnChain
            },

            // 大文件 + 任意模式 -> IPFS存储
            (ContentSizeCategory::Large, _, MessageType::File, _) => {
                StorageTier::IPFS
            },

            // 媒体文件 -> IPFS存储
            (_, _, MessageType::Image | MessageType::Video | MessageType::Audio, _) => {
                StorageTier::IPFS
            },

            // 临时消息 + 任意模式 + 高优先级 -> 临时存储
            (_, _, MessageType::Temporary | MessageType::Ephemeral, _) => {
                StorageTier::Temporary
            },

            // 中等消息 + 加密模式 -> 混合存储
            (ContentSizeCategory::Medium, EncryptionMode::Business | EncryptionMode::Military, _, _) => {
                StorageTier::Hybrid
            },

            // 紧急消息 -> 链上存储（最快访问）
            (_, _, _, MessageUrgency::Emergency) => {
                StorageTier::OnChain
            },

            // 默认：根据内容大小决定
            (ContentSizeCategory::Small, _, _, _) => StorageTier::OnChain,
            (ContentSizeCategory::Medium, _, _, _) => StorageTier::Hybrid,
            (ContentSizeCategory::Large, _, _, _) => StorageTier::IPFS,
        };

        StorageStrategy {
            primary_tier: storage_tier,
            backup_tier: Self::determine_backup_tier(storage_tier, encryption_mode),
            replication_factor: Self::calculate_replication_factor(encryption_mode, group_size),
            ttl_seconds: Self::calculate_ttl(message_type, encryption_mode, retention_policy),
            compression_enabled: content_analysis.compression_beneficial,
            auto_migration_enabled: Self::should_enable_auto_migration(message_type, encryption_mode),
        }
    }

    /// 分析内容特性
    fn analyze_content_characteristics(content: &[u8]) -> ContentAnalysis {
        let size = content.len();

        let size_category = match size {
            0..=256 => ContentSizeCategory::Small,
            257..=2048 => ContentSizeCategory::Medium,
            _ => ContentSizeCategory::Large,
        };

        // 分析内容类型
        let content_type = Self::detect_content_type(content);

        // 评估压缩效果
        let compression_beneficial = Self::estimate_compression_benefit(content, content_type);

        // 计算内容熵
        let entropy = Self::calculate_entropy(content);

        ContentAnalysis {
            size,
            size_category,
            content_type,
            entropy,
            compression_beneficial,
            estimated_compression_ratio: Self::estimate_compression_ratio(content, content_type),
        }
    }

    /// 检测内容类型
    fn detect_content_type(content: &[u8]) -> ContentType {
        if content.is_empty() {
            return ContentType::Empty;
        }

        // 检查常见文件头
        match content.get(0..4) {
            Some([0xFF, 0xD8, 0xFF, _]) => ContentType::JPEG,
            Some([0x89, 0x50, 0x4E, 0x47]) => ContentType::PNG,
            Some([0x47, 0x49, 0x46, 0x38]) => ContentType::GIF,
            Some([0x50, 0x4B, 0x03, 0x04]) => ContentType::Archive, // ZIP/DOCX/etc
            Some([0x25, 0x50, 0x44, 0x46]) => ContentType::PDF,
            _ => {
                // 检查是否为文本内容
                if Self::is_text_content(content) {
                    if Self::is_json_content(content) {
                        ContentType::JSON
                    } else if Self::is_code_content(content) {
                        ContentType::Code
                    } else {
                        ContentType::Text
                    }
                } else {
                    ContentType::Binary
                }
            }
        }
    }

    /// 检查是否为文本内容
    fn is_text_content(content: &[u8]) -> bool {
        // 检查UTF-8有效性和可打印字符比例
        if let Ok(text) = sp_std::str::from_utf8(content) {
            let printable_chars = text.chars()
                .filter(|&c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .count();
            let total_chars = text.chars().count();

            total_chars > 0 && (printable_chars as f32 / total_chars as f32) > 0.8
        } else {
            false
        }
    }

    /// 检查是否为JSON内容
    fn is_json_content(content: &[u8]) -> bool {
        if let Ok(text) = sp_std::str::from_utf8(content) {
            let trimmed = text.trim();
            (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
            (trimmed.starts_with('[') && trimmed.ends_with(']'))
        } else {
            false
        }
    }

    /// 检查是否为代码内容
    fn is_code_content(content: &[u8]) -> bool {
        if let Ok(text) = sp_std::str::from_utf8(content) {
            let code_keywords = ["function", "class", "import", "export", "const", "let", "var", "fn", "pub", "struct"];
            let code_symbols = ["->", "=>", "::", "&&", "||"];

            let has_keywords = code_keywords.iter().any(|&keyword| text.contains(keyword));
            let has_symbols = code_symbols.iter().any(|&symbol| text.contains(symbol));

            has_keywords || has_symbols
        } else {
            false
        }
    }

    /// 估计压缩效果
    fn estimate_compression_benefit(content: &[u8], content_type: ContentType) -> bool {
        match content_type {
            ContentType::Text | ContentType::JSON | ContentType::Code => {
                // 文本内容通常可以压缩
                content.len() > 100 // 只有足够大的内容才值得压缩
            },
            ContentType::Binary => {
                // 分析重复字节的比例
                Self::calculate_repetition_ratio(content) > 0.3
            },
            ContentType::JPEG | ContentType::PNG | ContentType::GIF => {
                // 图像文件已经压缩过
                false
            },
            ContentType::Archive => {
                // 归档文件已经压缩过
                false
            },
            ContentType::PDF => {
                // PDF文件通常已经优化
                false
            },
            ContentType::Empty => false,
        }
    }

    /// 计算重复字节比例
    fn calculate_repetition_ratio(content: &[u8]) -> f32 {
        if content.len() <= 1 {
            return 0.0;
        }

        let mut byte_counts = [0u32; 256];
        for &byte in content {
            byte_counts[byte as usize] += 1;
        }

        let max_count = byte_counts.iter().max().unwrap_or(&0);
        *max_count as f32 / content.len() as f32
    }

    /// 估计压缩比例
    fn estimate_compression_ratio(content: &[u8], content_type: ContentType) -> f32 {
        match content_type {
            ContentType::Text => 0.3, // 文本通常可以压缩到30%
            ContentType::JSON => 0.2, // JSON压缩效果更好
            ContentType::Code => 0.4, // 代码压缩效果一般
            ContentType::Binary => {
                let repetition = Self::calculate_repetition_ratio(content);
                1.0 - repetition * 0.7 // 根据重复率估计
            },
            _ => 1.0, // 其他类型不压缩
        }
    }

    /// 计算内容熵
    fn calculate_entropy(content: &[u8]) -> f32 {
        if content.is_empty() {
            return 0.0;
        }

        let mut byte_counts = [0u32; 256];
        for &byte in content {
            byte_counts[byte as usize] += 1;
        }

        let length = content.len() as f32;
        let mut entropy = 0.0f32;

        for count in byte_counts.iter() {
            if *count > 0 {
                let probability = *count as f32 / length;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// 确定备份存储层
    fn determine_backup_tier(primary_tier: StorageTier, encryption_mode: EncryptionMode) -> Option<StorageTier> {
        match (primary_tier, encryption_mode) {
            (StorageTier::OnChain, EncryptionMode::Military) => Some(StorageTier::IPFS),
            (StorageTier::OnChain, _) => Some(StorageTier::Hybrid),
            (StorageTier::IPFS, _) => Some(StorageTier::Hybrid),
            (StorageTier::Hybrid, _) => None, // 混合存储已经有冗余
            (StorageTier::Temporary, _) => None, // 临时存储不需要备份
        }
    }

    /// 计算复制因子
    fn calculate_replication_factor(encryption_mode: EncryptionMode, group_size: u32) -> u32 {
        let base_factor = match encryption_mode {
            EncryptionMode::Military => 5, // 军用级：5份复制
            EncryptionMode::Business => 3, // 商用级：3份复制
            EncryptionMode::Selective => 2, // 选择性：2份复制
            EncryptionMode::Transparent => 2, // 透明：2份复制
        };

        // 根据群组大小调整
        if group_size > 100 {
            base_factor + 1
        } else if group_size < 10 {
            base_factor.max(2) // 最少2份
        } else {
            base_factor
        }
    }

    /// 计算生存时间
    fn calculate_ttl(
        message_type: MessageType,
        encryption_mode: EncryptionMode,
        retention_policy: RetentionPolicy,
    ) -> Option<u64> {
        // 首先根据消息类型确定基础TTL
        let base_ttl = match message_type {
            MessageType::Temporary => Some(3600), // 临时消息：1小时
            MessageType::Ephemeral => Some(86400), // 阅后即焚：1天
            MessageType::System => Some(2592000), // 系统消息：1个月
            _ => None, // 普通消息：根据其他因素决定
        };

        if base_ttl.is_some() {
            return base_ttl;
        }

        // 根据保留策略决定
        match retention_policy {
            RetentionPolicy::Permanent => None,
            RetentionPolicy::Days(days) => Some(days as u64 * 86400),
            RetentionPolicy::Months(months) => Some(months as u64 * 2592000),
            RetentionPolicy::Years(years) => Some(years as u64 * 31536000),
            RetentionPolicy::UntilGroupDisbanded => None,
        }
    }

    /// 是否启用自动迁移
    fn should_enable_auto_migration(message_type: MessageType, encryption_mode: EncryptionMode) -> bool {
        match message_type {
            MessageType::Temporary | MessageType::Ephemeral => false, // 临时消息不迁移
            MessageType::System => false, // 系统消息保持原地
            _ => match encryption_mode {
                EncryptionMode::Military => true, // 军用级启用智能迁移
                EncryptionMode::Business => true, // 商用级启用智能迁移
                EncryptionMode::Selective => true, // 选择性启用智能迁移
                EncryptionMode::Transparent => true, // 透明模式启用智能迁移
            }
        }
    }

    /// 执行存储策略
    pub fn execute_storage_strategy(
        message_meta: &GroupMessageMeta<T>,
        strategy: &StorageStrategy<T>,
    ) -> Result<StorageExecutionResult, StorageError> {
        let mut result = StorageExecutionResult {
            primary_storage_success: false,
            backup_storage_success: false,
            compression_applied: false,
            compression_ratio: 1.0,
            storage_locations: Vec::new(),
            total_storage_used: 0,
        };

        let content = &message_meta.content;

        // 应用压缩（如果启用）
        let (final_content, compression_ratio) = if strategy.compression_enabled {
            let compressed = Self::compress_content(content, message_meta.message_type)?;
            let ratio = compressed.len() as f32 / content.len() as f32;
            result.compression_applied = true;
            result.compression_ratio = ratio;
            (compressed, ratio)
        } else {
            (content.clone().into(), 1.0)
        };

        // 执行主存储
        match Self::store_in_tier(message_meta, &final_content, strategy.primary_tier) {
            Ok(location_info) => {
                result.primary_storage_success = true;
                result.storage_locations.push(location_info.clone());
                result.total_storage_used += location_info.size_used;
            },
            Err(e) => return Err(e),
        }

        // 执行备份存储（如果配置）
        if let Some(backup_tier) = &strategy.backup_tier {
            match Self::store_in_tier(message_meta, &final_content, *backup_tier) {
                Ok(location_info) => {
                    result.backup_storage_success = true;
                    result.storage_locations.push(location_info.clone());
                    result.total_storage_used += location_info.size_used;
                },
                Err(_) => {
                    // 备份存储失败不是致命错误
                }
            }
        }

        Ok(result)
    }

    /// 在指定存储层存储内容
    fn store_in_tier(
        message_meta: &GroupMessageMeta<T>,
        content: &Vec<u8>,
        tier: StorageTier,
    ) -> Result<StorageLocationInfo, StorageError> {
        match tier {
            StorageTier::OnChain => Self::store_on_chain(message_meta, content),
            StorageTier::IPFS => Self::store_on_ipfs(message_meta, content),
            StorageTier::Hybrid => Self::store_hybrid(message_meta, content),
            StorageTier::Temporary => Self::store_temporary(message_meta, content),
        }
    }

    /// 链上存储实现
    fn store_on_chain(
        message_meta: &GroupMessageMeta<T>,
        content: &Vec<u8>,
    ) -> Result<StorageLocationInfo, StorageError> {
        // 链上存储通过 pallet 存储映射实现
        Ok(StorageLocationInfo {
            tier: StorageTier::OnChain,
            location_id: format!("onchain:{}", message_meta.id),
            size_used: content.len() as u64,
            access_time_ms: 50, // 链上访问很快
            retrieval_cost: StorageCost::Low,
        })
    }

    /// IPFS存储实现
    fn store_on_ipfs(
        message_meta: &GroupMessageMeta<T>,
        content: &Vec<u8>,
    ) -> Result<StorageLocationInfo, StorageError> {
        // 这里应该调用 pallet-stardust-ipfs 进行实际存储
        // 目前使用模拟实现

        let content_hash = sp_io::hashing::blake2_256(content);
        let ipfs_cid = format!("Qm{}", hex::encode(&content_hash[0..16]));

        Ok(StorageLocationInfo {
            tier: StorageTier::IPFS,
            location_id: ipfs_cid,
            size_used: content.len() as u64,
            access_time_ms: 500, // IPFS访问中等速度
            retrieval_cost: StorageCost::Medium,
        })
    }

    /// 混合存储实现
    fn store_hybrid(
        message_meta: &GroupMessageMeta<T>,
        content: &Vec<u8>,
    ) -> Result<StorageLocationInfo, StorageError> {
        // 元数据存储在链上，内容存储在IPFS
        if content.len() <= 256 {
            // 小内容直接链上存储
            Self::store_on_chain(message_meta, content)
        } else {
            // 大内容IPFS存储
            Self::store_on_ipfs(message_meta, content)
        }
    }

    /// 临时存储实现
    fn store_temporary(
        message_meta: &GroupMessageMeta<T>,
        content: &Vec<u8>,
    ) -> Result<StorageLocationInfo, StorageError> {
        // 临时存储只在内存中，不持久化
        Ok(StorageLocationInfo {
            tier: StorageTier::Temporary,
            location_id: format!("temp:{}", message_meta.id),
            size_used: content.len() as u64,
            access_time_ms: 10, // 内存访问最快
            retrieval_cost: StorageCost::Free,
        })
    }

    /// 压缩内容
    fn compress_content(content: &[u8], message_type: MessageType) -> Result<Vec<u8>, StorageError> {
        match message_type {
            MessageType::Text | MessageType::System => {
                Self::compress_text(content)
            },
            MessageType::File => {
                // 根据文件类型选择压缩算法
                Self::compress_generic(content)
            },
            _ => {
                // 其他类型不压缩
                Ok(content.to_vec())
            }
        }
    }

    /// 文本压缩（简化的LZ77算法）
    fn compress_text(content: &[u8]) -> Result<Vec<u8>, StorageError> {
        if content.len() < 16 {
            return Ok(content.to_vec()); // 太小的内容不压缩
        }

        let mut compressed = Vec::new();
        let mut i = 0;

        while i < content.len() {
            let mut best_length = 0;
            let mut best_distance = 0;

            // 查找最长匹配
            let start = if i >= 255 { i - 255 } else { 0 };
            for j in start..i {
                let mut length = 0;
                while i + length < content.len() &&
                      j + length < i &&
                      content[j + length] == content[i + length] {
                    length += 1;
                }

                if length > best_length && length >= 3 {
                    best_length = length;
                    best_distance = i - j;
                }
            }

            if best_length >= 3 {
                // 编码匹配
                compressed.push(0xFF); // 标识符
                compressed.push(best_distance as u8);
                compressed.push(best_length as u8);
                i += best_length;
            } else {
                // 编码单个字符
                compressed.push(content[i]);
                i += 1;
            }
        }

        // 只有压缩效果显著时才返回压缩结果
        if compressed.len() < content.len() * 9 / 10 {
            Ok(compressed)
        } else {
            Ok(content.to_vec())
        }
    }

    /// 通用压缩
    fn compress_generic(content: &[u8]) -> Result<Vec<u8>, StorageError> {
        // 简化的重复字节压缩
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < content.len() {
            let current_byte = content[i];
            let mut count = 1;

            // 计算连续相同字节的数量
            while i + count < content.len() && content[i + count] == current_byte && count < 255 {
                count += 1;
            }

            if count >= 3 {
                // 编码重复序列
                compressed.push(0xFE); // 重复标识符
                compressed.push(count as u8);
                compressed.push(current_byte);
                i += count;
            } else {
                // 直接存储字节
                for _ in 0..count {
                    compressed.push(current_byte);
                }
                i += count;
            }
        }

        Ok(compressed)
    }

    /// 解压缩内容
    pub fn decompress_content(
        compressed: &[u8],
        message_type: MessageType,
    ) -> Result<Vec<u8>, StorageError> {
        match message_type {
            MessageType::Text | MessageType::System => {
                Self::decompress_text(compressed)
            },
            MessageType::File => {
                Self::decompress_generic(compressed)
            },
            _ => Ok(compressed.to_vec()),
        }
    }

    /// 文本解压缩
    fn decompress_text(compressed: &[u8]) -> Result<Vec<u8>, StorageError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < compressed.len() {
            if compressed[i] == 0xFF && i + 2 < compressed.len() {
                // 解码匹配
                let distance = compressed[i + 1] as usize;
                let length = compressed[i + 2] as usize;

                if distance <= decompressed.len() {
                    let start = decompressed.len() - distance;
                    for j in 0..length {
                        if start + (j % distance) < decompressed.len() {
                            let byte = decompressed[start + (j % distance)];
                            decompressed.push(byte);
                        }
                    }
                }

                i += 3;
            } else {
                // 直接复制字符
                decompressed.push(compressed[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// 通用解压缩
    fn decompress_generic(compressed: &[u8]) -> Result<Vec<u8>, StorageError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < compressed.len() {
            if compressed[i] == 0xFE && i + 2 < compressed.len() {
                // 解码重复序列
                let count = compressed[i + 1] as usize;
                let byte = compressed[i + 2];

                for _ in 0..count {
                    decompressed.push(byte);
                }

                i += 3;
            } else {
                decompressed.push(compressed[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// 智能存储迁移
    pub fn execute_intelligent_migration(
        current_storage_info: &StorageLocationInfo,
        access_pattern: &AccessPattern,
        group_settings: &GroupStorageSettings,
    ) -> Result<Option<StorageTier>, StorageError> {
        let current_time = 1234567890u64; // 应该从 TimeProvider 获取

        // 分析是否需要迁移
        let migration_decision = Self::analyze_migration_need(
            current_storage_info,
            access_pattern,
            group_settings,
            current_time,
        );

        match migration_decision {
            MigrationDecision::NoMigration => Ok(None),
            MigrationDecision::MigrateTo(target_tier) => {
                // 验证迁移的合理性
                if Self::validate_migration(current_storage_info.tier, target_tier) {
                    Ok(Some(target_tier))
                } else {
                    Ok(None)
                }
            },
        }
    }

    /// 分析迁移需求
    fn analyze_migration_need(
        storage_info: &StorageLocationInfo,
        access_pattern: &AccessPattern,
        settings: &GroupStorageSettings,
        current_time: u64,
    ) -> MigrationDecision {
        let age_seconds = current_time.saturating_sub(access_pattern.last_access_time);
        let access_frequency = if age_seconds > 0 {
            access_pattern.total_accesses as f64 / age_seconds as f64
        } else {
            0.0
        };

        match storage_info.tier {
            StorageTier::OnChain => {
                // 链上数据如果很少访问，可以迁移到IPFS
                if age_seconds > settings.cold_data_threshold &&
                   access_frequency < settings.low_access_frequency {
                    MigrationDecision::MigrateTo(StorageTier::IPFS)
                } else {
                    MigrationDecision::NoMigration
                }
            },

            StorageTier::IPFS => {
                // IPFS数据如果频繁访问，可以迁移到链上
                if access_frequency > settings.high_access_frequency {
                    MigrationDecision::MigrateTo(StorageTier::OnChain)
                }
                // 如果长期不访问，迁移到临时存储
                else if age_seconds > settings.archive_threshold &&
                        access_frequency < settings.archive_frequency {
                    MigrationDecision::MigrateTo(StorageTier::Temporary)
                } else {
                    MigrationDecision::NoMigration
                }
            },

            StorageTier::Hybrid => {
                // 混合存储根据访问模式调整
                if access_frequency > settings.high_access_frequency {
                    MigrationDecision::MigrateTo(StorageTier::OnChain)
                } else if access_frequency < settings.low_access_frequency {
                    MigrationDecision::MigrateTo(StorageTier::IPFS)
                } else {
                    MigrationDecision::NoMigration
                }
            },

            StorageTier::Temporary => {
                // 临时存储如果频繁访问，升级到链上
                if access_frequency > settings.high_access_frequency {
                    MigrationDecision::MigrateTo(StorageTier::OnChain)
                } else {
                    MigrationDecision::NoMigration
                }
            },
        }
    }

    /// 验证迁移合理性
    fn validate_migration(from_tier: StorageTier, to_tier: StorageTier) -> bool {
        match (from_tier, to_tier) {
            // 禁止的迁移路径
            (StorageTier::Temporary, StorageTier::IPFS) => false, // 临时不能直接到IPFS
            (StorageTier::OnChain, StorageTier::Temporary) => false, // 链上不能直接到临时

            // 允许的迁移路径
            _ => true,
        }
    }

    /// 计算存储成本
    pub fn calculate_storage_cost(
        strategy: &StorageStrategy<T>,
        content_size: u64,
        duration_seconds: u64,
    ) -> StorageCostEstimate {
        let primary_cost = Self::calculate_tier_cost(
            strategy.primary_tier,
            content_size,
            duration_seconds,
            strategy.replication_factor,
        );

        let backup_cost = if let Some(backup_tier) = &strategy.backup_tier {
            Self::calculate_tier_cost(
                *backup_tier,
                content_size,
                duration_seconds,
                strategy.replication_factor,
            )
        } else {
            TierCost::default()
        };

        StorageCostEstimate {
            primary_cost,
            backup_cost,
            total_cost: primary_cost.total + backup_cost.total,
            cost_per_gb_month: Self::calculate_cost_per_gb_month(&primary_cost, &backup_cost),
        }
    }

    /// 计算单个存储层的成本
    fn calculate_tier_cost(
        tier: StorageTier,
        size_bytes: u64,
        duration_seconds: u64,
        replication_factor: u32,
    ) -> TierCost {
        let base_cost_per_gb_month = match tier {
            StorageTier::OnChain => 100.0, // 链上存储成本最高
            StorageTier::IPFS => 5.0,      // IPFS中等成本
            StorageTier::Hybrid => 20.0,   // 混合存储中高成本
            StorageTier::Temporary => 0.1, // 临时存储成本最低
        };

        let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let duration_months = duration_seconds as f64 / (30.0 * 24.0 * 3600.0);
        let storage_cost = base_cost_per_gb_month * size_gb * duration_months;

        let bandwidth_cost = match tier {
            StorageTier::OnChain => 0.0,  // 链上无带宽成本
            StorageTier::IPFS => 0.1,     // IPFS少量带宽成本
            StorageTier::Hybrid => 0.05,  // 混合存储少量带宽成本
            StorageTier::Temporary => 0.0, // 临时存储无带宽成本
        };

        let replication_multiplier = replication_factor as f64;

        TierCost {
            storage: storage_cost * replication_multiplier,
            bandwidth: bandwidth_cost * size_gb,
            operations: 0.01, // 固定操作成本
            total: (storage_cost + bandwidth_cost + 0.01) * replication_multiplier,
        }
    }

    /// 计算每GB每月成本
    fn calculate_cost_per_gb_month(primary: &TierCost, backup: &TierCost) -> f64 {
        primary.total + backup.total
    }
}

/// 内容分析结果
#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub size: usize,
    pub size_category: ContentSizeCategory,
    pub content_type: ContentType,
    pub entropy: f32,
    pub compression_beneficial: bool,
    pub estimated_compression_ratio: f32,
}

/// 内容大小分类
#[derive(Debug, Clone, PartialEq)]
pub enum ContentSizeCategory {
    Small,  // 0-256 bytes
    Medium, // 257-2048 bytes
    Large,  // >2048 bytes
}

/// 内容类型
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Empty,
    Text,
    JSON,
    Code,
    Binary,
    JPEG,
    PNG,
    GIF,
    PDF,
    Archive,
}

/// 消息紧急程度
#[derive(Debug, Clone, PartialEq)]
pub enum MessageUrgency {
    Emergency, // 紧急消息
    High,      // 高优先级
    Normal,    // 普通优先级
    Low,       // 低优先级
}

/// 保留策略
#[derive(Debug, Clone, PartialEq)]
pub enum RetentionPolicy {
    Permanent,              // 永久保存
    Days(u32),             // 保存指定天数
    Months(u32),           // 保存指定月数
    Years(u32),            // 保存指定年数
    UntilGroupDisbanded,   // 保存到群组解散
}

/// 存储执行结果
#[derive(Debug, Clone)]
pub struct StorageExecutionResult {
    pub primary_storage_success: bool,
    pub backup_storage_success: bool,
    pub compression_applied: bool,
    pub compression_ratio: f32,
    pub storage_locations: Vec<StorageLocationInfo>,
    pub total_storage_used: u64,
}

/// 存储位置信息
#[derive(Debug, Clone)]
pub struct StorageLocationInfo {
    pub tier: StorageTier,
    pub location_id: String,
    pub size_used: u64,
    pub access_time_ms: u32,
    pub retrieval_cost: StorageCost,
}

/// 存储成本类型
#[derive(Debug, Clone, PartialEq)]
pub enum StorageCost {
    Free,
    Low,
    Medium,
    High,
}

/// 访问模式
#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub total_accesses: u32,
    pub last_access_time: u64,
    pub access_frequency: f64,
    pub access_times: Vec<u64>,
}

/// 群组存储设置
#[derive(Debug, Clone)]
pub struct GroupStorageSettings {
    pub cold_data_threshold: u64,     // 冷数据阈值（秒）
    pub archive_threshold: u64,       // 归档阈值（秒）
    pub low_access_frequency: f64,    // 低访问频率阈值
    pub high_access_frequency: f64,   // 高访问频率阈值
    pub archive_frequency: f64,       // 归档频率阈值
}

impl Default for GroupStorageSettings {
    fn default() -> Self {
        Self {
            cold_data_threshold: 2592000,    // 30天
            archive_threshold: 7776000,      // 90天
            low_access_frequency: 0.001,     // 每秒0.001次
            high_access_frequency: 0.1,      // 每秒0.1次
            archive_frequency: 0.0001,       // 每秒0.0001次
        }
    }
}

/// 迁移决策
#[derive(Debug, Clone)]
pub enum MigrationDecision {
    NoMigration,
    MigrateTo(StorageTier),
}

/// 存储成本估算
#[derive(Debug, Clone)]
pub struct StorageCostEstimate {
    pub primary_cost: TierCost,
    pub backup_cost: TierCost,
    pub total_cost: f64,
    pub cost_per_gb_month: f64,
}

/// 单个层级成本
#[derive(Debug, Clone, Default)]
pub struct TierCost {
    pub storage: f64,   // 存储成本
    pub bandwidth: f64, // 带宽成本
    pub operations: f64, // 操作成本
    pub total: f64,     // 总成本
}

/// 存储错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    TierNotAvailable,
    InsufficientSpace,
    CompressionFailed,
    EncryptionRequired,
    NetworkError,
    PermissionDenied,
    InvalidContent,
    MigrationFailed,
}

// 临时的 hex 编码函数（因为在 no_std 环境中）
mod hex {
    use sp_std::vec::Vec;

    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}