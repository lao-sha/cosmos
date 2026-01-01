/// Stardust智能群聊系统 - AI智能决策引擎
///
/// 实现内容智能分析和自动化决策系统

use crate::types::*;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use sp_std::{vec::Vec, collections::btree_map::BTreeMap};

/// AI智能决策引擎
pub struct IntelligentDecisionEngine<T: frame_system::Config> {
    _phantom: sp_std::marker::PhantomData<T>,
}

impl<T: frame_system::Config> IntelligentDecisionEngine<T> {
    /// 创建新的AI决策引擎实例
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }

    /// 智能加密模式决策
    pub fn decide_encryption_mode(
        content: &[u8],
        context: &ConversationContext<T>,
        user_behavior: &UserBehaviorData<T>,
        group_analytics: &GroupAnalytics,
    ) -> Result<EncryptionRecommendation, AIError> {
        // 1. 内容敏感性分析
        let sensitivity_analysis = Self::analyze_content_sensitivity(content)?;

        // 2. 场景识别
        let scene_analysis = Self::identify_conversation_scene(context, user_behavior)?;

        // 3. 风险评估
        let risk_assessment = Self::assess_security_risk(
            &sensitivity_analysis,
            &scene_analysis,
            group_analytics,
        )?;

        // 4. 用户偏好学习
        let preference_weight = Self::analyze_user_preference(user_behavior);

        // 5. 综合决策
        let decision = Self::make_encryption_decision(
            sensitivity_analysis,
            scene_analysis,
            risk_assessment,
            preference_weight,
        )?;

        Ok(decision)
    }

    /// 内容敏感性分析
    fn analyze_content_sensitivity(content: &[u8]) -> Result<SensitivityAnalysis, AIError> {
        let content_str = sp_std::str::from_utf8(content)
            .unwrap_or("");

        let mut analysis = SensitivityAnalysis::default();

        // 1. 关键词检测
        analysis.keyword_score = Self::calculate_keyword_sensitivity(content_str);

        // 2. 模式识别
        analysis.pattern_score = Self::analyze_sensitive_patterns(content_str);

        // 3. 个人信息检测
        analysis.pii_score = Self::detect_personal_information(content_str);

        // 4. 财务信息检测
        analysis.financial_score = Self::detect_financial_information(content_str);

        // 5. 技术信息检测
        analysis.technical_score = Self::detect_technical_information(content_str);

        // 6. 综合评分计算
        analysis.overall_score = Self::calculate_overall_sensitivity(
            analysis.keyword_score,
            analysis.pattern_score,
            analysis.pii_score,
            analysis.financial_score,
            analysis.technical_score,
        );

        // 7. 置信度计算
        analysis.confidence = Self::calculate_sensitivity_confidence(&analysis);

        Ok(analysis)
    }

    /// 关键词敏感性计算
    fn calculate_keyword_sensitivity(content: &str) -> f32 {
        let high_sensitivity_keywords = [
            // 英文关键词
            "password", "secret", "private", "confidential", "classified",
            "bank", "card", "ssn", "credit", "account", "pin", "token",
            "api", "key", "certificate", "signature", "wallet",

            // 中文关键词
            "密码", "机密", "私人", "保密", "绝密",
            "银行", "卡号", "身份证", "信用卡", "账户", "密钥",
            "证书", "签名", "钱包", "财务"
        ];

        let medium_sensitivity_keywords = [
            "email", "phone", "address", "name", "id",
            "project", "meeting", "document", "file",
            "邮箱", "电话", "地址", "姓名", "项目", "会议", "文档"
        ];

        let low_sensitivity_keywords = [
            "work", "office", "team", "group", "chat",
            "工作", "办公室", "团队", "群组", "聊天"
        ];

        let content_lower = content.to_lowercase();
        let mut score = 0.0f32;

        // 高敏感度关键词
        for keyword in &high_sensitivity_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                score += 0.3;
            }
        }

        // 中敏感度关键词
        for keyword in &medium_sensitivity_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                score += 0.15;
            }
        }

        // 低敏感度关键词
        for keyword in &low_sensitivity_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                score += 0.05;
            }
        }

        score.min(1.0)
    }

    /// 敏感模式分析
    fn analyze_sensitive_patterns(content: &str) -> f32 {
        let mut score = 0.0f32;

        // 1. 信用卡号模式
        if Self::contains_credit_card_pattern(content) {
            score += 0.4;
        }

        // 2. 身份证号模式
        if Self::contains_id_number_pattern(content) {
            score += 0.4;
        }

        // 3. 邮箱地址模式
        if Self::contains_email_pattern(content) {
            score += 0.2;
        }

        // 4. 电话号码模式
        if Self::contains_phone_pattern(content) {
            score += 0.2;
        }

        // 5. IP地址模式
        if Self::contains_ip_pattern(content) {
            score += 0.1;
        }

        // 6. URL模式
        if Self::contains_url_pattern(content) {
            score += 0.1;
        }

        // 7. 密钥格式模式
        if Self::contains_key_pattern(content) {
            score += 0.5;
        }

        score.min(1.0)
    }

    /// 检测信用卡号模式
    fn contains_credit_card_pattern(content: &str) -> bool {
        // 简化的信用卡号检测（4组4位数字）
        let digit_groups: Vec<&str> = content.split_whitespace()
            .filter(|s| s.chars().all(|c| c.is_ascii_digit()) && s.len() == 4)
            .collect();

        digit_groups.len() >= 4
    }

    /// 检测身份证号模式
    fn contains_id_number_pattern(content: &str) -> bool {
        // 检测18位数字（中国身份证）或其他常见格式
        content.chars()
            .filter(|c| c.is_ascii_digit())
            .count() >= 15
    }

    /// 检测邮箱模式
    fn contains_email_pattern(content: &str) -> bool {
        content.contains('@') && content.contains('.')
    }

    /// 检测电话号码模式
    fn contains_phone_pattern(content: &str) -> bool {
        let digits = content.chars().filter(|c| c.is_ascii_digit()).count();
        digits >= 7 && digits <= 15 &&
        (content.contains('+') || content.contains('-') || content.contains('('))
    }

    /// 检测IP地址模式
    fn contains_ip_pattern(content: &str) -> bool {
        content.matches('.').count() == 3 &&
        content.split('.').all(|part| {
            part.parse::<u32>().map_or(false, |n| n <= 255)
        })
    }

    /// 检测URL模式
    fn contains_url_pattern(content: &str) -> bool {
        content.contains("http://") || content.contains("https://") ||
        content.contains("ftp://") || content.contains("www.")
    }

    /// 检测密钥格式模式
    fn contains_key_pattern(content: &str) -> bool {
        // 检测Base64编码的密钥、JWT token等
        let long_alphanumeric: Vec<&str> = content.split_whitespace()
            .filter(|s| s.len() > 32 && s.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='))
            .collect();

        !long_alphanumeric.is_empty()
    }

    /// 个人信息检测
    fn detect_personal_information(content: &str) -> f32 {
        let pii_indicators = [
            "姓名", "name", "身份证", "identity", "住址", "address",
            "生日", "birthday", "年龄", "age", "性别", "gender"
        ];

        let mut score = 0.0f32;
        let content_lower = content.to_lowercase();

        for indicator in &pii_indicators {
            if content_lower.contains(&indicator.to_lowercase()) {
                score += 0.15;
            }
        }

        // 检测可能的姓名模式（中文姓名）
        if Self::contains_chinese_name_pattern(content) {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// 检测中文姓名模式
    fn contains_chinese_name_pattern(content: &str) -> bool {
        // 简化的中文姓名检测逻辑
        let common_surnames = [
            "王", "李", "张", "刘", "陈", "杨", "黄", "赵", "吴", "周",
            "徐", "孙", "马", "朱", "胡", "林", "郭", "何", "高", "罗"
        ];

        for surname in &common_surnames {
            if content.contains(surname) {
                // 检查后面是否跟着1-2个中文字符
                if let Some(pos) = content.find(surname) {
                    let after_surname = &content[pos + surname.len()..];
                    let chinese_chars: Vec<char> = after_surname.chars()
                        .take(2)
                        .filter(|c| *c as u32 >= 0x4E00 && *c as u32 <= 0x9FFF)
                        .collect();

                    if chinese_chars.len() >= 1 {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// 财务信息检测
    fn detect_financial_information(content: &str) -> f32 {
        let financial_keywords = [
            "银行", "bank", "卡号", "card", "账户", "account",
            "余额", "balance", "转账", "transfer", "支付", "payment",
            "工资", "salary", "收入", "income", "支出", "expense",
            "投资", "investment", "股票", "stock", "基金", "fund"
        ];

        let mut score = 0.0f32;
        let content_lower = content.to_lowercase();

        for keyword in &financial_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                score += 0.2;
            }
        }

        // 检测货币金额
        if Self::contains_currency_amount(content) {
            score += 0.3;
        }

        score.min(1.0)
    }

    /// 检测货币金额
    fn contains_currency_amount(content: &str) -> bool {
        let currency_symbols = ["¥", "$", "€", "£", "元", "美元", "欧元", "英镑"];

        for symbol in &currency_symbols {
            if content.contains(symbol) {
                return true;
            }
        }

        // 检测数字+货币单位的模式
        let amount_patterns = ["万元", "千元", "百万", "亿元"];
        for pattern in &amount_patterns {
            if content.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// 技术信息检测
    fn detect_technical_information(content: &str) -> f32 {
        let technical_keywords = [
            "api", "token", "key", "password", "secret",
            "database", "server", "config", "admin",
            "数据库", "服务器", "配置", "管理员",
            "接口", "令牌", "密钥", "系统"
        ];

        let mut score = 0.0f32;
        let content_lower = content.to_lowercase();

        for keyword in &technical_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                score += 0.15;
            }
        }

        // 检测代码片段
        if Self::contains_code_snippets(content) {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// 检测代码片段
    fn contains_code_snippets(content: &str) -> bool {
        let code_indicators = [
            "function", "class", "import", "export", "const", "let", "var",
            "fn", "pub", "struct", "impl", "trait", "use",
            "def", "class", "import", "from",
            "=>", "->", "::", "&&", "||", "!=", "=="
        ];

        let mut code_score = 0;
        for indicator in &code_indicators {
            if content.contains(indicator) {
                code_score += 1;
            }
        }

        code_score >= 2
    }

    /// 计算综合敏感性评分
    fn calculate_overall_sensitivity(
        keyword_score: f32,
        pattern_score: f32,
        pii_score: f32,
        financial_score: f32,
        technical_score: f32,
    ) -> f32 {
        // 加权平均计算
        let weights = [0.3, 0.3, 0.2, 0.15, 0.05];
        let scores = [keyword_score, pattern_score, pii_score, financial_score, technical_score];

        let weighted_sum: f32 = weights.iter()
            .zip(scores.iter())
            .map(|(w, s)| w * s)
            .sum();

        weighted_sum.min(1.0)
    }

    /// 计算敏感性分析置信度
    fn calculate_sensitivity_confidence(analysis: &SensitivityAnalysis) -> f32 {
        let scores = [
            analysis.keyword_score,
            analysis.pattern_score,
            analysis.pii_score,
            analysis.financial_score,
            analysis.technical_score,
        ];

        // 如果多个维度都有较高分数，置信度更高
        let high_scores = scores.iter().filter(|&&s| s > 0.3).count();
        let medium_scores = scores.iter().filter(|&&s| s > 0.1 && s <= 0.3).count();

        let confidence = match (high_scores, medium_scores) {
            (3.., _) => 0.9,      // 多个高分维度
            (2, _) => 0.8,        // 两个高分维度
            (1, 2..) => 0.7,      // 一个高分加多个中等
            (1, 1) => 0.6,        // 一个高分一个中等
            (0, 3..) => 0.5,      // 多个中等分数
            _ => 0.3,             // 分数都很低
        };

        confidence
    }

    /// 场景识别分析
    fn identify_conversation_scene(
        context: &ConversationContext<T>,
        user_behavior: &UserBehaviorData<T>,
    ) -> Result<SceneAnalysis, AIError> {
        let mut scene_analysis = SceneAnalysis::default();

        // 1. 时间模式分析
        scene_analysis.time_pattern = Self::analyze_time_pattern(context);

        // 2. 参与者关系分析
        scene_analysis.participant_relationship = Self::analyze_participant_relationship(context);

        // 3. 群组类型推断
        scene_analysis.group_type = Self::infer_group_type(context, user_behavior);

        // 4. 交流频率分析
        scene_analysis.communication_frequency = Self::analyze_communication_frequency(context);

        // 5. 主题一致性分析
        scene_analysis.topic_consistency = Self::analyze_topic_consistency(context);

        // 6. 综合场景分类
        scene_analysis.primary_scene = Self::classify_primary_scene(&scene_analysis);

        // 7. 置信度评估
        scene_analysis.confidence = Self::calculate_scene_confidence(&scene_analysis);

        Ok(scene_analysis)
    }

    /// 时间模式分析
    fn analyze_time_pattern(context: &ConversationContext<T>) -> TimePattern {
        let current_hour = context.current_time % (24 * 3600) / 3600;

        match current_hour {
            9..=17 => TimePattern::BusinessHours,
            18..=22 => TimePattern::EveningHours,
            23..=24 | 0..=6 => TimePattern::LateNight,
            _ => TimePattern::Other,
        }
    }

    /// 参与者关系分析
    fn analyze_participant_relationship(context: &ConversationContext<T>) -> ParticipantRelationship {
        match context.participant_count {
            2 => ParticipantRelationship::OneOnOne,
            3..=10 => ParticipantRelationship::SmallGroup,
            11..=50 => ParticipantRelationship::MediumGroup,
            _ => ParticipantRelationship::LargeGroup,
        }
    }

    /// 群组类型推断
    fn infer_group_type(
        context: &ConversationContext<T>,
        _user_behavior: &UserBehaviorData<T>,
    ) -> GroupType {
        // 根据群组大小和活动模式推断
        match (context.participant_count, context.message_frequency) {
            (2..=5, freq) if freq > 0.1 => GroupType::Family,
            (5..=20, freq) if freq > 0.05 => GroupType::WorkTeam,
            (20..=100, _) => GroupType::Company,
            (_, freq) if freq < 0.01 => GroupType::Community,
            _ => GroupType::Social,
        }
    }

    /// 交流频率分析
    fn analyze_communication_frequency(context: &ConversationContext<T>) -> CommunicationFrequency {
        match context.message_frequency {
            freq if freq > 0.1 => CommunicationFrequency::High,
            freq if freq > 0.01 => CommunicationFrequency::Medium,
            _ => CommunicationFrequency::Low,
        }
    }

    /// 主题一致性分析
    fn analyze_topic_consistency(_context: &ConversationContext<T>) -> TopicConsistency {
        // 简化实现，实际应该分析消息内容的主题相关性
        TopicConsistency::Medium
    }

    /// 分类主要场景
    fn classify_primary_scene(analysis: &SceneAnalysis) -> ConversationScene {
        match (analysis.time_pattern, analysis.group_type, analysis.communication_frequency) {
            (TimePattern::BusinessHours, GroupType::WorkTeam, CommunicationFrequency::High) => {
                ConversationScene::BusinessMeeting
            },
            (_, GroupType::Family, _) => ConversationScene::FamilyChat,
            (TimePattern::EveningHours | TimePattern::LateNight, GroupType::Social, _) => {
                ConversationScene::SocialChat
            },
            (TimePattern::BusinessHours, GroupType::Company, _) => {
                ConversationScene::WorkDiscussion
            },
            (_, _, CommunicationFrequency::High) => ConversationScene::UrgentCommunication,
            _ => ConversationScene::CasualChat,
        }
    }

    /// 计算场景分析置信度
    fn calculate_scene_confidence(analysis: &SceneAnalysis) -> f32 {
        let mut confidence = 0.5f32;

        // 根据各项指标的明确性调整置信度
        match analysis.time_pattern {
            TimePattern::BusinessHours => confidence += 0.1,
            _ => confidence += 0.05,
        }

        match analysis.communication_frequency {
            CommunicationFrequency::High | CommunicationFrequency::Low => confidence += 0.1,
            _ => confidence += 0.05,
        }

        match analysis.topic_consistency {
            TopicConsistency::High => confidence += 0.1,
            TopicConsistency::Medium => confidence += 0.05,
            _ => {},
        }

        confidence.min(1.0)
    }

    /// 安全风险评估
    fn assess_security_risk(
        sensitivity: &SensitivityAnalysis,
        scene: &SceneAnalysis,
        _group_analytics: &GroupAnalytics,
    ) -> Result<RiskAssessment, AIError> {
        let mut risk_assessment = RiskAssessment::default();

        // 1. 基于敏感性的风险
        risk_assessment.content_risk = Self::calculate_content_risk(sensitivity);

        // 2. 基于场景的风险
        risk_assessment.context_risk = Self::calculate_context_risk(scene);

        // 3. 传输风险评估
        risk_assessment.transmission_risk = Self::calculate_transmission_risk(scene);

        // 4. 存储风险评估
        risk_assessment.storage_risk = Self::calculate_storage_risk(sensitivity, scene);

        // 5. 综合风险评分
        risk_assessment.overall_risk = Self::calculate_overall_risk(
            risk_assessment.content_risk,
            risk_assessment.context_risk,
            risk_assessment.transmission_risk,
            risk_assessment.storage_risk,
        );

        // 6. 风险等级分类
        risk_assessment.risk_level = Self::classify_risk_level(risk_assessment.overall_risk);

        Ok(risk_assessment)
    }

    /// 计算内容风险
    fn calculate_content_risk(sensitivity: &SensitivityAnalysis) -> f32 {
        sensitivity.overall_score
    }

    /// 计算上下文风险
    fn calculate_context_risk(scene: &SceneAnalysis) -> f32 {
        match scene.primary_scene {
            ConversationScene::BusinessMeeting => 0.8,
            ConversationScene::WorkDiscussion => 0.6,
            ConversationScene::UrgentCommunication => 0.7,
            ConversationScene::FamilyChat => 0.3,
            ConversationScene::SocialChat => 0.2,
            ConversationScene::CasualChat => 0.1,
        }
    }

    /// 计算传输风险
    fn calculate_transmission_risk(scene: &SceneAnalysis) -> f32 {
        match scene.participant_relationship {
            ParticipantRelationship::OneOnOne => 0.2,
            ParticipantRelationship::SmallGroup => 0.3,
            ParticipantRelationship::MediumGroup => 0.5,
            ParticipantRelationship::LargeGroup => 0.7,
        }
    }

    /// 计算存储风险
    fn calculate_storage_risk(sensitivity: &SensitivityAnalysis, scene: &SceneAnalysis) -> f32 {
        let base_risk = sensitivity.overall_score * 0.5;

        let context_multiplier = match scene.primary_scene {
            ConversationScene::BusinessMeeting | ConversationScene::WorkDiscussion => 1.2,
            _ => 1.0,
        };

        (base_risk * context_multiplier).min(1.0)
    }

    /// 计算综合风险
    fn calculate_overall_risk(
        content_risk: f32,
        context_risk: f32,
        transmission_risk: f32,
        storage_risk: f32,
    ) -> f32 {
        // 加权平均
        let weights = [0.4, 0.25, 0.2, 0.15];
        let risks = [content_risk, context_risk, transmission_risk, storage_risk];

        weights.iter()
            .zip(risks.iter())
            .map(|(w, r)| w * r)
            .sum::<f32>()
            .min(1.0)
    }

    /// 分类风险等级
    fn classify_risk_level(overall_risk: f32) -> RiskLevel {
        match overall_risk {
            r if r >= 0.8 => RiskLevel::Critical,
            r if r >= 0.6 => RiskLevel::High,
            r if r >= 0.4 => RiskLevel::Medium,
            r if r >= 0.2 => RiskLevel::Low,
            _ => RiskLevel::Minimal,
        }
    }

    /// 用户偏好分析
    fn analyze_user_preference(user_behavior: &UserBehaviorData<T>) -> UserPreferenceWeight {
        UserPreferenceWeight {
            security_preference: user_behavior.security_preference,
            convenience_preference: 1.0 - user_behavior.security_preference,
            cost_sensitivity: 0.5, // 简化实现
            response_time_importance: match user_behavior.response_time_preference {
                ResponseTimePreference::Fast => 0.8,
                ResponseTimePreference::Balanced => 0.5,
                ResponseTimePreference::Secure => 0.2,
            },
        }
    }

    /// 最终加密决策
    fn make_encryption_decision(
        sensitivity: SensitivityAnalysis,
        scene: SceneAnalysis,
        risk: RiskAssessment,
        preference: UserPreferenceWeight,
    ) -> Result<EncryptionRecommendation, AIError> {
        // 计算各加密模式的适合度评分
        let military_score = Self::calculate_military_mode_score(&sensitivity, &scene, &risk, &preference);
        let business_score = Self::calculate_business_mode_score(&sensitivity, &scene, &risk, &preference);
        let selective_score = Self::calculate_selective_mode_score(&sensitivity, &scene, &risk, &preference);
        let transparent_score = Self::calculate_transparent_mode_score(&sensitivity, &scene, &risk, &preference);

        // 找出最高评分的模式
        let scores = [
            (EncryptionMode::Military, military_score),
            (EncryptionMode::Business, business_score),
            (EncryptionMode::Selective, selective_score),
            (EncryptionMode::Transparent, transparent_score),
        ];

        let (recommended_mode, best_score) = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(sp_std::cmp::Ordering::Equal))
            .ok_or(AIError::DecisionFailed)?;

        // 生成替代选项
        let mut alternatives: Vec<EncryptionMode> = scores.iter()
            .filter(|(mode, score)| *mode != *recommended_mode && *score > 0.3)
            .map(|(mode, _)| *mode)
            .collect();
        alternatives.truncate(3);

        // 生成决策理由
        let reasoning = Self::generate_decision_reasoning(
            *recommended_mode,
            &sensitivity,
            &scene,
            &risk,
        );

        Ok(EncryptionRecommendation {
            recommended_mode: *recommended_mode,
            confidence: *best_score,
            reasoning,
            alternatives: alternatives.try_into().unwrap_or_default(),
            sensitivity_analysis: sensitivity,
            scene_analysis: scene,
            risk_assessment: risk,
        })
    }

    /// 计算军用级模式评分
    fn calculate_military_mode_score(
        sensitivity: &SensitivityAnalysis,
        scene: &SceneAnalysis,
        risk: &RiskAssessment,
        preference: &UserPreferenceWeight,
    ) -> f32 {
        let mut score = 0.0f32;

        // 高敏感内容适合军用级
        score += sensitivity.overall_score * 0.4;

        // 高风险场景适合军用级
        if risk.risk_level == RiskLevel::Critical || risk.risk_level == RiskLevel::High {
            score += 0.3;
        }

        // 商务场景适合军用级
        match scene.primary_scene {
            ConversationScene::BusinessMeeting | ConversationScene::WorkDiscussion => {
                score += 0.2;
            },
            _ => {},
        }

        // 用户安全偏好
        score += preference.security_preference * 0.3;

        // 对响应时间不敏感时适合军用级
        score += (1.0 - preference.response_time_importance) * 0.2;

        score.min(1.0)
    }

    /// 计算商用级模式评分
    fn calculate_business_mode_score(
        sensitivity: &SensitivityAnalysis,
        scene: &SceneAnalysis,
        risk: &RiskAssessment,
        preference: &UserPreferenceWeight,
    ) -> f32 {
        let mut score = 0.5f32; // 商用级作为默认选择有较高基础分

        // 中等敏感内容适合商用级
        if sensitivity.overall_score >= 0.3 && sensitivity.overall_score < 0.7 {
            score += 0.3;
        }

        // 中等风险适合商用级
        if risk.risk_level == RiskLevel::Medium {
            score += 0.3;
        }

        // 大多数场景都适合商用级
        match scene.primary_scene {
            ConversationScene::BusinessMeeting => score += 0.2,
            ConversationScene::WorkDiscussion => score += 0.25,
            ConversationScene::SocialChat => score += 0.15,
            _ => score += 0.1,
        }

        // 平衡用户偏好
        score += (preference.security_preference + preference.convenience_preference) * 0.15;

        score.min(1.0)
    }

    /// 计算选择性模式评分
    fn calculate_selective_mode_score(
        sensitivity: &SensitivityAnalysis,
        scene: &SceneAnalysis,
        _risk: &RiskAssessment,
        preference: &UserPreferenceWeight,
    ) -> f32 {
        let mut score = 0.3f32;

        // 低中敏感内容适合选择性
        if sensitivity.overall_score < 0.5 {
            score += 0.3;
        }

        // 社交场景适合选择性
        match scene.primary_scene {
            ConversationScene::SocialChat | ConversationScene::CasualChat => {
                score += 0.3;
            },
            _ => {},
        }

        // 用户便利性偏好
        score += preference.convenience_preference * 0.3;

        // 响应时间重要时适合选择性
        score += preference.response_time_importance * 0.2;

        score.min(1.0)
    }

    /// 计算透明模式评分
    fn calculate_transparent_mode_score(
        sensitivity: &SensitivityAnalysis,
        scene: &SceneAnalysis,
        risk: &RiskAssessment,
        preference: &UserPreferenceWeight,
    ) -> f32 {
        let mut score = 0.2f32;

        // 低敏感内容适合透明模式
        if sensitivity.overall_score < 0.2 {
            score += 0.4;
        }

        // 低风险适合透明模式
        if risk.risk_level == RiskLevel::Minimal || risk.risk_level == RiskLevel::Low {
            score += 0.3;
        }

        // 公开场景适合透明模式
        match scene.primary_scene {
            ConversationScene::CasualChat => score += 0.3,
            _ => {},
        }

        // 响应时间要求高时适合透明模式
        score += preference.response_time_importance * 0.3;

        // 便利性优先时适合透明模式
        score += preference.convenience_preference * 0.2;

        score.min(1.0)
    }

    /// 生成决策理由
    fn generate_decision_reasoning(
        recommended_mode: EncryptionMode,
        sensitivity: &SensitivityAnalysis,
        scene: &SceneAnalysis,
        risk: &RiskAssessment,
    ) -> BoundedVec<u8, ConstU32<512>> {
        let reason = match recommended_mode {
            EncryptionMode::Military => {
                format!(
                    "推荐军用级加密：内容敏感性{:.0}%，风险等级{:?}，适合{:?}场景的最高安全保护",
                    sensitivity.overall_score * 100.0,
                    risk.risk_level,
                    scene.primary_scene
                )
            },
            EncryptionMode::Business => {
                format!(
                    "推荐商用级加密：平衡安全性与性能，适合{:?}场景，风险等级{:?}",
                    scene.primary_scene,
                    risk.risk_level
                )
            },
            EncryptionMode::Selective => {
                format!(
                    "推荐选择性加密：灵活的安全选择，适合{:?}场景，用户可根据需要调整",
                    scene.primary_scene
                )
            },
            EncryptionMode::Transparent => {
                format!(
                    "推荐透明模式：内容敏感性较低({:.0}%)，{:?}场景适合公开交流",
                    sensitivity.overall_score * 100.0,
                    scene.primary_scene
                )
            },
        };

        reason.as_bytes().to_vec().try_into().unwrap_or_default()
    }
}

/// AI分析相关的数据结构

/// 对话上下文
#[derive(Debug, Clone)]
pub struct ConversationContext<T: frame_system::Config> {
    pub group_id: GroupId,
    pub participant_count: u32,
    pub message_frequency: f32, // 消息/秒
    pub current_time: u64,
    pub recent_messages: Vec<MessageId>,
    pub group_age_days: u32,
}

/// 群组分析数据
#[derive(Debug, Clone, Default)]
pub struct GroupAnalytics {
    pub total_messages: u32,
    pub average_message_length: u32,
    pub encryption_mode_distribution: BTreeMap<EncryptionMode, u32>,
    pub peak_activity_hours: Vec<u8>,
    pub member_activity_scores: BTreeMap<String, f32>,
}

/// 敏感性分析结果
#[derive(Debug, Clone, Default)]
pub struct SensitivityAnalysis {
    pub keyword_score: f32,
    pub pattern_score: f32,
    pub pii_score: f32,
    pub financial_score: f32,
    pub technical_score: f32,
    pub overall_score: f32,
    pub confidence: f32,
}

/// 场景分析结果
#[derive(Debug, Clone, Default)]
pub struct SceneAnalysis {
    pub time_pattern: TimePattern,
    pub participant_relationship: ParticipantRelationship,
    pub group_type: GroupType,
    pub communication_frequency: CommunicationFrequency,
    pub topic_consistency: TopicConsistency,
    pub primary_scene: ConversationScene,
    pub confidence: f32,
}

/// 风险评估结果
#[derive(Debug, Clone, Default)]
pub struct RiskAssessment {
    pub content_risk: f32,
    pub context_risk: f32,
    pub transmission_risk: f32,
    pub storage_risk: f32,
    pub overall_risk: f32,
    pub risk_level: RiskLevel,
}

/// 用户偏好权重
#[derive(Debug, Clone)]
pub struct UserPreferenceWeight {
    pub security_preference: f32,
    pub convenience_preference: f32,
    pub cost_sensitivity: f32,
    pub response_time_importance: f32,
}

/// 加密推荐结果
#[derive(Debug, Clone)]
pub struct EncryptionRecommendation {
    pub recommended_mode: EncryptionMode,
    pub confidence: f32,
    pub reasoning: BoundedVec<u8, ConstU32<512>>,
    pub alternatives: BoundedVec<EncryptionMode, ConstU32<3>>,
    pub sensitivity_analysis: SensitivityAnalysis,
    pub scene_analysis: SceneAnalysis,
    pub risk_assessment: RiskAssessment,
}

/// 枚举定义

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TimePattern {
    BusinessHours,
    EveningHours,
    LateNight,
    #[default]
    Other,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ParticipantRelationship {
    OneOnOne,
    #[default]
    SmallGroup,
    MediumGroup,
    LargeGroup,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum GroupType {
    Family,
    WorkTeam,
    Company,
    Community,
    #[default]
    Social,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CommunicationFrequency {
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TopicConsistency {
    High,
    #[default]
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ConversationScene {
    BusinessMeeting,
    WorkDiscussion,
    UrgentCommunication,
    FamilyChat,
    SocialChat,
    #[default]
    CasualChat,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    #[default]
    Minimal,
}

/// AI错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum AIError {
    AnalysisFailed,
    InsufficientData,
    DecisionFailed,
    InvalidInput,
    ModelNotAvailable,
}