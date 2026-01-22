//! # 八字解盘 Runtime API
//!
//! 本模块定义了八字解盘系统的 Runtime API，供前端通过 RPC 免费调用。
//!
//! ## 功能说明
//!
//! ### 基础功能
//! - `get_interpretation`: 获取完整解盘（核心指标+性格分析+扩展忌神）
//! - `get_full_bazi_chart`: 获取完整八字命盘（主星、藏干、副星、星运、空亡、纳音、神煞）★ V5 新增
//! - `chart_exists`: 检查命盘是否存在
//! - `get_chart_owner`: 获取命盘创建者
//!
//! ### 加密命盘功能
//! - `get_encrypted_chart_interpretation`: 获取加密命盘的解盘结果
//! - `encrypted_chart_exists`: 检查加密命盘是否存在
//! - `get_encrypted_chart_owner`: 获取加密命盘创建者
//!
//! ### 多方授权加密系统（V6 新增）
//! - `get_user_encryption_key`: 获取用户加密公钥
//! - `get_service_provider`: 获取服务提供者信息
//! - `get_providers_by_type`: 获取某类型的服务提供者列表
//! - `get_provider_grants`: 获取被授权访问的命盘列表
//! - `get_multi_key_encrypted_chart_info`: 获取多方授权加密命盘基础信息
//!
//! ## 使用方式
//!
//! 前端通过 polkadot.js API 调用：
//! ```javascript
//! // 获取完整解盘
//! const result = await api.call.baziChartApi.getInterpretation(chartId);
//!
//! // 访问核心数据
//! const { geJu, qiangRuo, yongShen, score } = result.core;
//!
//! // 访问性格分析
//! const { zhuYaoTeDian, youDian } = result.xingGe;
//!
//! // 获取完整八字命盘（V5 新增）
//! const fullChart = await api.call.baziChartApi.getFullBaziChart(chartId);
//! // 访问主星
//! console.log(fullChart.sizhu.yearZhu.tianganShishen);
//! // 访问空亡
//! console.log(fullChart.kongwang.dayIsKong);
//! // 访问神煞
//! fullChart.shenshaList.forEach(s => console.log(s.shensha, s.nature));
//!
//! // 获取加密命盘的解盘
//! const encryptedResult = await api.call.baziChartApi.getEncryptedChartInterpretation(chartId);
//!
//! // V6 新增：多方授权功能
//! // 获取用户加密公钥
//! const pubKey = await api.call.baziChartApi.getUserEncryptionKey(accountId);
//!
//! // 获取命理师列表
//! const providers = await api.call.baziChartApi.getProvidersByType(0); // 0=MingLiShi
//!
//! // 获取被授权的命盘列表
//! const grants = await api.call.baziChartApi.getProviderGrants(accountId);
//! ```
//!
//! ## 版本说明
//!
//! V6 版本新增多方授权加密系统支持：
//! - 新增 `get_user_encryption_key` 接口
//! - 新增 `get_service_provider` 接口
//! - 新增 `get_providers_by_type` 接口
//! - 新增 `get_provider_grants` 接口
//! - 新增 `get_multi_key_encrypted_chart_info` 接口
//!
//! V5 版本新增完整命盘接口：
//! - V4 原有功能保持不变
//! - 新增 `get_full_bazi_chart` 接口，返回 `FullBaziChart`
//! - 包含：主星、藏干（副星）、星运（十二长生）、空亡、纳音、神煞
//!
//! 历史版本：
//! - V4: 加密命盘支持
//! - V3: FullInterpretation（核心指标+性格分析）
//! - V2: SimplifiedInterpretation

use crate::interpretation::FullInterpretation;
use codec::Codec;
use scale_info::prelude::string::String;

sp_api::decl_runtime_apis! {
    /// 八字解盘 Runtime API
    ///
    /// 提供八字命盘的免费查询接口，无需支付 Gas 费用。
    ///
    /// ## 设计原则
    ///
    /// - 单一接口：只提供 `get_interpretation`，返回完整数据
    /// - 前端按需使用：只需核心数据时访问 `.core`，需要性格分析时访问 `.xing_ge`
    /// - 零成本抽象：性格分析计算开销极小（< 1μs）
    /// - 隐私保护：支持加密命盘的解盘查询
    pub trait BaziChartApi<AccountId>
    where
        AccountId: Codec,
    {
        /// 获取完整解盘（唯一接口）
        ///
        /// 返回数据结构：
        /// ```text
        /// FullInterpretation
        /// ├── core: CoreInterpretation (13 bytes)
        /// │   ├── ge_ju          格局（正格、从强格、从弱格等）
        /// │   ├── qiang_ruo      强弱（身旺、身弱、中和等）
        /// │   ├── yong_shen      用神（金、木、水、火、土）
        /// │   ├── yong_shen_type 用神类型（扶抑、调候、通关、专旺）
        /// │   ├── xi_shen        喜神
        /// │   ├── ji_shen        忌神
        /// │   ├── score          综合评分 (0-100)
        /// │   ├── confidence     可信度 (0-100)
        /// │   ├── timestamp      时间戳（区块号）
        /// │   └── algorithm_version 算法版本
        /// │
        /// ├── xing_ge: Option<CompactXingGe> (性格分析)
        /// │   ├── zhu_yao_te_dian  主要性格特点（最多3个）
        /// │   ├── you_dian         优点（最多3个）
        /// │   ├── que_dian         缺点（最多2个）
        /// │   └── shi_he_zhi_ye    适合职业（最多4个）
        /// │
        /// └── extended_ji_shen: Option<ExtendedJiShen> (扩展忌神)
        ///     └── secondary        次忌神列表（最多2个）
        /// ```
        ///
        /// # 参数
        /// - `chart_id`: 八字命盘 ID
        ///
        /// # 返回
        /// - `Some(FullInterpretation)`: 完整解盘结果
        /// - `None`: 命盘不存在
        ///
        /// # 示例
        /// ```javascript
        /// // 前端调用
        /// const result = await api.call.baziChartApi.getInterpretation(chartId);
        ///
        /// // 只需核心数据（等价于旧版 V2/V3 Core）
        /// const core = result.core;
        /// console.log(core.geJu, core.score);
        ///
        /// // 需要性格分析
        /// if (result.xingGe) {
        ///     console.log(result.xingGe.zhuYaoTeDian);
        /// }
        /// ```
        fn get_interpretation(chart_id: u64) -> Option<FullInterpretation>;

        /// 获取完整八字命盘（V5 新增）
        ///
        /// 返回包含所有计算字段的完整命盘数据：
        /// - **主星**: 天干十神 + 地支本气十神
        /// - **藏干（副星）**: 藏干详细信息及十神关系
        /// - **星运**: 四柱十二长生状态
        /// - **空亡**: 旬空判断和标识
        /// - **纳音**: 六十甲子纳音五行
        /// - **神煞**: 吉凶神煞列表
        ///
        /// # 参数
        /// - `chart_id`: 八字命盘 ID
        ///
        /// # 返回
        /// - `Some(FullBaziChartForApi)`: 完整命盘数据
        /// - `None`: 命盘不存在
        ///
        /// # 特点
        /// - 完全免费（无 gas 费用）
        /// - 响应快速（< 100ms）
        /// - 包含所有计算字段
        /// - 算法自动更新（无需数据迁移）
        ///
        /// # 示例
        /// ```javascript
        /// // 前端调用
        /// const fullChart = await api.call.baziChartApi.getFullBaziChart(chartId);
        ///
        /// // 访问主星
        /// console.log('年柱天干十神:', fullChart.sizhu.yearZhu.tianganShishen);
        /// console.log('年柱本气十神:', fullChart.sizhu.yearZhu.dizhiBenqiShishen);
        ///
        /// // 访问藏干（副星）
        /// fullChart.sizhu.yearZhu.cangganList.forEach(cg => {
        ///     console.log(`藏干 ${cg.gan}，十神 ${cg.shishen}，权重 ${cg.weight}`);
        /// });
        ///
        /// // 访问星运（十二长生）
        /// console.log('日柱十二长生:', fullChart.sizhu.dayZhu.changsheng);
        /// console.log('四柱星运:', fullChart.xingyun);
        ///
        /// // 访问空亡
        /// if (fullChart.kongwang.dayIsKong) {
        ///     console.log('日柱落空亡');
        /// }
        ///
        /// // 访问纳音
        /// console.log('日柱纳音:', fullChart.sizhu.dayZhu.nayin);
        ///
        /// // 访问神煞
        /// fullChart.shenshaList.forEach(s => {
        ///     console.log(`${s.shensha} 在 ${s.position}，${s.nature}`);
        /// });
        /// ```
        ///
        /// # 调试友好（方案1实现）
        ///
        /// 返回的 JSON 字符串包含可读的枚举名称：
        /// - `gender`: "Male" 而不是 0
        /// - `shenSha`: "TianYiGuiRen" 而不是 0
        /// - `changsheng`: "ChangSheng" 而不是 0
        ///
        /// 方便前端调试，无需查找枚举映射表。
        fn get_full_bazi_chart(chart_id: u64) -> Option<String>;

        /// 检查命盘是否存在
        ///
        /// # 参数
        /// - `chart_id`: 八字命盘 ID
        ///
        /// # 返回
        /// - `true`: 命盘存在
        /// - `false`: 命盘不存在
        fn chart_exists(chart_id: u64) -> bool;

        /// 获取命盘创建者
        ///
        /// # 参数
        /// - `chart_id`: 八字命盘 ID
        ///
        /// # 返回
        /// - `Some(AccountId)`: 命盘创建者地址
        /// - `None`: 命盘不存在
        fn get_chart_owner(chart_id: u64) -> Option<AccountId>;

        /// 获取加密命盘的完整解盘
        ///
        /// 基于加密命盘的四柱索引计算解盘，无需解密敏感数据。
        ///
        /// # 参数
        /// - `chart_id`: 加密八字命盘 ID
        ///
        /// # 返回
        /// - `Some(FullInterpretation)`: 完整解盘结果
        /// - `None`: 命盘不存在
        ///
        /// # 特点
        /// - 完全免费（无 gas 费用）
        /// - 保护用户隐私（不访问加密数据）
        /// - 基于四柱索引计算，可信度略低于完整命盘
        ///
        /// # 示例
        /// ```javascript
        /// const result = await api.call.baziChartApi.getEncryptedChartInterpretation(chartId);
        /// console.log(result.core.geJu, result.core.score);
        /// ```
        fn get_encrypted_chart_interpretation(chart_id: u64) -> Option<FullInterpretation>;

        /// 检查加密命盘是否存在
        ///
        /// # 参数
        /// - `chart_id`: 加密八字命盘 ID
        ///
        /// # 返回
        /// - `true`: 命盘存在
        /// - `false`: 命盘不存在
        fn encrypted_chart_exists(chart_id: u64) -> bool;

        /// 获取加密命盘创建者
        ///
        /// # 参数
        /// - `chart_id`: 加密八字命盘 ID
        ///
        /// # 返回
        /// - `Some(AccountId)`: 命盘创建者地址
        /// - `None`: 命盘不存在
        fn get_encrypted_chart_owner(chart_id: u64) -> Option<AccountId>;

        /// 临时排盘（统一接口，不存储，免费）
        ///
        /// 根据输入类型计算八字命盘，但不存储到链上。
        /// 支持三种输入方式：公历、农历、四柱直接输入。
        /// 适用于用户"试用"功能，决定是否保存后再调用交易接口。
        ///
        /// # 参数
        /// - `input_type`: 输入类型标识
        ///   - 0: 公历日期 (Solar)
        ///   - 1: 农历日期 (Lunar)
        ///   - 2: 四柱直接输入 (SiZhu)
        /// - `params`: 参数数组（根据 input_type 解释）
        ///   - Solar: [year, month, day, hour, minute] 或 [year, month, day, hour, minute, longitude]
        ///   - Lunar: [year, month, day, is_leap_month(0/1), hour, minute] 或 [..., longitude]
        ///   - SiZhu: [year_gz, month_gz, day_gz, hour_gz, birth_year]
        ///   - longitude: 可选，经度×100的整数（如12050=120.50°E，负数=西经），用于真太阳时修正
        /// - `gender`: 性别 (0=Male, 1=Female)
        /// - `zishi_mode`: 子时模式 (1=Traditional, 2=Modern)
        ///
        /// # 返回
        /// - `Some(String)`: JSON 格式的完整命盘数据
        /// - `None`: 输入参数无效
        ///
        /// # 特点
        /// - ✅ 完全免费（无 Gas 费用）
        /// - ✅ 支持三种输入方式
        /// - ✅ 支持真太阳时修正（可选经度参数）
        /// - ✅ 响应快速（< 100ms）
        /// - ❌ 不存储（关闭页面后数据丢失）
        ///
        /// # 示例
        /// ```javascript
        /// // 公历输入（不启用真太阳时）
        /// const chart = await api.call.baziChartApi.calculateBaziTempUnified(
        ///     0,                    // input_type: Solar
        ///     [1990, 5, 15, 14, 30], // params: year, month, day, hour, minute
        ///     0,                    // gender: Male
        ///     2,                    // zishi_mode: Modern
        /// );
        ///
        /// // 公历输入（启用真太阳时，北京经度116.40°E）
        /// const chart = await api.call.baziChartApi.calculateBaziTempUnified(
        ///     0,                          // input_type: Solar
        ///     [1990, 5, 15, 14, 30, 11640], // params: 含经度
        ///     0,                          // gender: Male
        ///     2,                          // zishi_mode: Modern
        /// );
        ///
        /// // 农历输入
        /// const chart = await api.call.baziChartApi.calculateBaziTempUnified(
        ///     1,                        // input_type: Lunar
        ///     [2024, 1, 1, 0, 12, 30],  // params: year, month, day, is_leap(0), hour, minute
        ///     1,                        // gender: Female
        ///     1,                        // zishi_mode: Traditional
        /// );
        ///
        /// // 四柱直接输入
        /// const chart = await api.call.baziChartApi.calculateBaziTempUnified(
        ///     2,                      // input_type: SiZhu
        ///     [0, 2, 4, 0, 1984],     // params: year_gz, month_gz, day_gz, hour_gz, birth_year
        ///     0,                      // gender: Male
        ///     2,                      // zishi_mode: Modern
        /// );
        /// ```
        fn calculate_bazi_temp_unified(
            input_type: u8,
            params: sp_std::vec::Vec<u16>,
            gender: u8,
            zishi_mode: u8,
        ) -> Option<String>;

        // ================================
        // V6 新增：多方授权加密系统 API
        // ================================

        /// 获取用户加密公钥
        ///
        /// 用于在授权前获取目标用户的 X25519 公钥
        ///
        /// # 参数
        /// - `account`: 用户账户
        ///
        /// # 返回
        /// - `Some([u8; 32])`: X25519 公钥
        /// - `None`: 用户未注册加密公钥
        ///
        /// # 示例
        /// ```javascript
        /// const pubKey = await api.call.baziChartApi.getUserEncryptionKey(accountId);
        /// if (pubKey.isSome) {
        ///     const key = pubKey.unwrap();
        ///     // 使用公钥加密 DataKey
        /// }
        /// ```
        fn get_user_encryption_key(account: AccountId) -> Option<[u8; 32]>;

        /// 获取服务提供者信息
        ///
        /// 获取服务提供者的详细信息（类型、公钥、信誉分等）
        ///
        /// # 参数
        /// - `account`: 服务提供者账户
        ///
        /// # 返回
        /// - `Some(String)`: JSON 格式的服务提供者信息
        ///   - provider_type: 服务类型
        ///   - public_key: X25519 公钥（hex）
        ///   - reputation: 信誉分（0-100）
        ///   - registered_at: 注册区块号
        ///   - is_active: 是否激活
        /// - `None`: 未注册为服务提供者
        ///
        /// # 示例
        /// ```javascript
        /// const info = await api.call.baziChartApi.getServiceProvider(accountId);
        /// if (info.isSome) {
        ///     const provider = JSON.parse(info.unwrap());
        ///     console.log(`类型: ${provider.provider_type}, 信誉: ${provider.reputation}`);
        /// }
        /// ```
        fn get_service_provider(account: AccountId) -> Option<String>;

        /// 获取某类型的服务提供者列表
        ///
        /// 按服务类型获取所有激活的服务提供者账户
        ///
        /// # 参数
        /// - `provider_type`: 服务类型
        ///   - 0: 命理师 (MingLiShi)
        ///   - 1: AI服务 (AiService)
        ///   - 2: 家族成员 (FamilyMember)
        ///   - 3: 研究机构 (Research)
        ///
        /// # 返回
        /// - 服务提供者账户列表
        ///
        /// # 示例
        /// ```javascript
        /// // 获取所有命理师
        /// const providers = await api.call.baziChartApi.getProvidersByType(0);
        /// for (const account of providers) {
        ///     const info = await api.call.baziChartApi.getServiceProvider(account);
        ///     // 显示命理师信息
        /// }
        /// ```
        fn get_providers_by_type(provider_type: u8) -> sp_std::vec::Vec<AccountId>;

        /// 获取被授权访问的命盘列表
        ///
        /// 服务提供者或用户查询自己被授权访问的所有命盘
        ///
        /// # 参数
        /// - `account`: 账户
        ///
        /// # 返回
        /// - 被授权访问的命盘 ID 列表
        ///
        /// # 示例
        /// ```javascript
        /// const grants = await api.call.baziChartApi.getProviderGrants(accountId);
        /// console.log(`被授权访问 ${grants.length} 个命盘`);
        /// for (const chartId of grants) {
        ///     // 解密并查看命盘
        /// }
        /// ```
        fn get_provider_grants(account: AccountId) -> sp_std::vec::Vec<u64>;

        /// 获取多方授权加密命盘的基础信息
        ///
        /// 返回命盘的元数据，不包含加密数据和密钥
        ///
        /// # 参数
        /// - `chart_id`: 命盘 ID
        ///
        /// # 返回
        /// - `Some(String)`: JSON 格式的命盘基础信息
        ///   - owner: 所有者账户
        ///   - sizhu_index: 四柱索引（明文）
        ///   - gender: 性别
        ///   - created_at: 创建区块号
        ///   - grants_count: 授权数量
        ///   - grant_accounts: 被授权账户列表（不含加密密钥）
        /// - `None`: 命盘不存在
        ///
        /// # 示例
        /// ```javascript
        /// const info = await api.call.baziChartApi.getMultiKeyEncryptedChartInfo(chartId);
        /// if (info.isSome) {
        ///     const chart = JSON.parse(info.unwrap());
        ///     console.log(`授权数: ${chart.grants_count}`);
        ///     console.log(`被授权账户: ${chart.grant_accounts.join(', ')}`);
        /// }
        /// ```
        fn get_multi_key_encrypted_chart_info(chart_id: u64) -> Option<String>;

        /// 获取多方授权加密命盘的解盘
        ///
        /// 基于四柱索引计算解盘，无需解密敏感数据
        ///
        /// # 参数
        /// - `chart_id`: 多方授权加密命盘 ID
        ///
        /// # 返回
        /// - `Some(FullInterpretation)`: 完整解盘结果
        /// - `None`: 命盘不存在
        ///
        /// # 示例
        /// ```javascript
        /// const result = await api.call.baziChartApi.getMultiKeyEncryptedChartInterpretation(chartId);
        /// console.log(result.core.geJu, result.core.score);
        /// ```
        fn get_multi_key_encrypted_chart_interpretation(chart_id: u64) -> Option<FullInterpretation>;
    }
}
