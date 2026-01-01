//! 塔罗牌常量定义 - 牌意数据
//!
//! 本模块提供塔罗牌的静态数据，包括：
//! - 大阿卡纳牌名、牌义（正位/逆位）、关键词、描述
//! - 小阿卡纳花色、牌名、牌义
//! - 大阿卡纳星座/行星对应
//! - 牌意查询辅助函数
//!
//! 这些数据主要供前端展示和链下 AI 解读服务使用。
//!
//! ## 数据来源
//!
//! 牌义数据参考经典塔罗解读体系，结合现代心理学诠释。

/// 大阿卡纳牌名（中文）
///
/// 索引对应牌 ID (0-21)
pub const MAJOR_ARCANA_NAMES_CN: [&str; 22] = [
    "愚者",     // 0
    "魔术师",   // 1
    "女祭司",   // 2
    "女皇",     // 3
    "皇帝",     // 4
    "教皇",     // 5
    "恋人",     // 6
    "战车",     // 7
    "力量",     // 8
    "隐者",     // 9
    "命运之轮", // 10
    "正义",     // 11
    "倒吊人",   // 12
    "死神",     // 13
    "节制",     // 14
    "恶魔",     // 15
    "塔",       // 16
    "星星",     // 17
    "月亮",     // 18
    "太阳",     // 19
    "审判",     // 20
    "世界",     // 21
];

/// 大阿卡纳牌名（英文）
pub const MAJOR_ARCANA_NAMES_EN: [&str; 22] = [
    "The Fool",
    "The Magician",
    "The High Priestess",
    "The Empress",
    "The Emperor",
    "The Hierophant",
    "The Lovers",
    "The Chariot",
    "Strength",
    "The Hermit",
    "Wheel of Fortune",
    "Justice",
    "The Hanged Man",
    "Death",
    "Temperance",
    "The Devil",
    "The Tower",
    "The Star",
    "The Moon",
    "The Sun",
    "Judgement",
    "The World",
];

// ============================================================================
// 大阿卡纳牌义数据
// ============================================================================

/// 大阿卡纳关键词
///
/// 每张牌的核心象征意义，用于快速理解牌意
pub const MAJOR_ARCANA_KEYWORDS: [&str; 22] = [
    "自由、天真、冒险、新开始",           // 0 愚者
    "创造力、意志力、技能、自信",         // 1 魔术师
    "直觉、神秘、潜意识、内在智慧",       // 2 女祭司
    "丰收、母性、创造、自然",             // 3 女皇
    "权威、结构、控制、父性",             // 4 皇帝
    "传统、信仰、教导、精神指引",         // 5 教皇
    "爱情、选择、和谐、价值观",           // 6 恋人
    "意志、胜利、决心、行动力",           // 7 战车
    "勇气、耐心、内在力量、自律",         // 8 力量
    "内省、智慧、独处、寻求真理",         // 9 隐者
    "命运、转折、机遇、因果循环",         // 10 命运之轮
    "公正、平衡、因果、真相",             // 11 正义
    "牺牲、等待、新视角、放下",           // 12 倒吊人
    "结束、转变、重生、放手",             // 13 死神
    "平衡、耐心、调和、中庸之道",         // 14 节制
    "束缚、欲望、物质主义、阴暗面",       // 15 恶魔
    "突变、毁灭、觉醒、解放",             // 16 塔
    "希望、灵感、宁静、信心",             // 17 星星
    "幻觉、恐惧、潜意识、直觉",           // 18 月亮
    "成功、活力、快乐、光明",             // 19 太阳
    "觉醒、重生、召唤、自我评价",         // 20 审判
    "完成、整合、成就、圆满",             // 21 世界
];

/// 大阿卡纳正位含义
///
/// 牌面正向时的积极解读
pub const MAJOR_ARCANA_UPRIGHT: [&str; 22] = [
    // 0 愚者
    "新的开始、冒险精神、自由自在、天真无邪、信任直觉、无限可能、勇于尝试、活在当下",
    // 1 魔术师
    "创造力旺盛、意志坚定、技能娴熟、资源充沛、新计划启动、把握机会、化想法为现实",
    // 2 女祭司
    "直觉敏锐、内在智慧、神秘力量、潜意识信息、静观其变、保守秘密、灵性发展",
    // 3 女皇
    "丰收在望、创造力强、母性关怀、感官享受、自然之美、孕育新生、物质充裕",
    // 4 皇帝
    "权威稳固、领导有方、结构清晰、自律严格、目标明确、保护他人、建立秩序",
    // 5 教皇
    "传统智慧、精神指引、道德准则、教育学习、寻求建议、信仰坚定、文化传承",
    // 6 恋人
    "真爱降临、和谐关系、重要选择、价值统一、心灵契合、承诺与责任、自我认同",
    // 7 战车
    "胜利在望、意志坚强、克服障碍、目标明确、行动果断、掌控局面、凯旋而归",
    // 8 力量
    "内在勇气、温和坚定、自我控制、耐心等待、以柔克刚、克服恐惧、慈悲心怀",
    // 9 隐者
    "内省思考、寻求真理、独处时光、智慧指引、精神追求、自我发现、深度思考",
    // 10 命运之轮
    "命运转折、好运来临、因果循环、抓住机遇、顺应变化、人生新阶段、时来运转",
    // 11 正义
    "公正判决、因果报应、诚实正直、平衡考量、法律事务顺利、真相大白、负责任",
    // 12 倒吊人
    "换个角度、自愿牺牲、等待时机、放下执念、灵性觉醒、新的领悟、暂停反思",
    // 13 死神
    "旧事结束、重大转变、放下过去、新生开始、蜕变成长、必要的结束、深刻改变",
    // 14 节制
    "平衡调和、耐心等待、中庸之道、融合对立、自我节制、目标明确、循序渐进",
    // 15 恶魔
    "面对阴暗面、认清束缚、物质享受、激情与欲望、打破限制、了解自我、掌控欲望",
    // 16 塔
    "突然变化、打破旧有、觉醒时刻、解放束缚、重建基础、真相揭露、危机转机",
    // 17 星星
    "希望重燃、灵感涌现、心灵宁静、信心恢复、美好愿景、疗愈进行、乐观向前",
    // 18 月亮
    "直觉增强、潜意识信息、面对恐惧、探索未知、创意灵感、情绪起伏、梦境启示",
    // 19 太阳
    "成功达成、活力充沛、快乐幸福、光明前景、自信满满、好消息来临、温暖关系",
    // 20 审判
    "重要决定、自我审视、觉醒时刻、听从召唤、放下过去、新的人生阶段、重生",
    // 21 世界
    "圆满完成、目标达成、整合统一、旅程结束、成就感、新循环开始、世界在手",
];

/// 大阿卡纳逆位含义
///
/// 牌面逆向时需要注意的解读
pub const MAJOR_ARCANA_REVERSED: [&str; 22] = [
    // 0 愚者
    "鲁莽冒进、不负责任、愚蠢决定、逃避现实、缺乏方向、冒不必要的险、漫无目的",
    // 1 魔术师
    "欺骗操控、才能未发挥、缺乏方向、投机取巧、自欺欺人、计划受阻、资源浪费",
    // 2 女祭司
    "忽视直觉、秘密泄露、表面化、过度理性、与内在失联、信息混乱、灵性阻塞",
    // 3 女皇
    "创造力受阻、过度依赖、忽视自我、物质匮乏、缺乏滋养、与自然疏离、不孕问题",
    // 4 皇帝
    "专制霸道、控制欲强、缺乏弹性、权威丧失、结构崩溃、逃避责任、父亲问题",
    // 5 教皇
    "打破传统、教条主义、不良建议、信仰危机、叛逆心理、精神空虚、虚伪权威",
    // 6 恋人
    "关系失衡、选择困难、价值冲突、不忠诚、自我否定、逃避承诺、内心矛盾",
    // 7 战车
    "失去方向、缺乏自律、攻击性强、受阻挫败、控制失调、强行推进、意志消沉",
    // 8 力量
    "自我怀疑、缺乏勇气、失控、自卑感、暴躁易怒、内在软弱、过度压抑",
    // 9 隐者
    "过度孤立、逃避社交、固执己见、迷失方向、过度内省、与世隔绝、拒绝指引",
    // 10 命运之轮
    "厄运连连、抗拒改变、失去控制、错失良机、负面循环、时运不济、命运捉弄",
    // 11 正义
    "不公正待遇、逃避责任、自欺欺人、法律问题、偏见判断、不诚实、因果报应",
    // 12 倒吊人
    "无意义的牺牲、拖延、固执不变、受害者心态、白费努力、停滞不前、抗拒放手",
    // 13 死神
    "抗拒改变、恐惧结束、停滞不前、无法放下、拖延转变、痛苦执着、不愿面对",
    // 14 节制
    "失去平衡、过度极端、缺乏耐心、冲动行事、目标模糊、不和谐、自我放纵",
    // 15 恶魔
    "沉溺欲望、受束缚、自我毁灭、成瘾问题、否认阴暗面、被操控、物质主义",
    // 16 塔
    "逃避变化、延长痛苦、小规模灾难、重复错误、抗拒觉醒、恐惧改变、内在动荡",
    // 17 星星
    "失去希望、缺乏信心、创意枯竭、与灵性失联、悲观消极、自我否定、不切实际",
    // 18 月亮
    "恐惧主导、幻觉困扰、情绪失控、逃避现实、焦虑不安、欺骗与误解、噩梦",
    // 19 太阳
    "过度乐观、自大傲慢、延迟成功、缺乏热情、被遮蔽的真相、暂时挫折、内在不快",
    // 20 审判
    "自我否定、逃避审视、错失机会、重复过去错误、无法放下、听不到召唤、拖延",
    // 21 世界
    "缺乏完成、目标未达、受限制、寻求结束、未完成的旅程、不完整感、循环受阻",
];

/// 大阿卡纳牌面描述
///
/// 对牌面图像和象征意义的简要描述
pub const MAJOR_ARCANA_DESCRIPTIONS: [&str; 22] = [
    // 0 愚者
    "一个年轻人站在悬崖边，背着行囊，手持白玫瑰，一只小狗陪伴。象征纯真与冒险精神。",
    // 1 魔术师
    "一位法师站在桌前，桌上摆放四元素工具，一手指天一手指地，头上有无限符号。象征连接天地的创造力。",
    // 2 女祭司
    "身着蓝袍的女性坐在两根柱子之间，手持卷轴，身后是石榴帷幕。象征神秘智慧与直觉。",
    // 3 女皇
    "华丽女性坐在丰饶的花园中，手持权杖，身边是金色麦穗。象征大地母亲与丰收。",
    // 4 皇帝
    "威严男性坐在石制宝座上，手持权杖和宝球，背景是山脉。象征稳固权威与秩序。",
    // 5 教皇
    "身着宗教法衣的智者坐在两根柱子之间，面前跪着两位信徒。象征精神指导与传统。",
    // 6 恋人
    "一男一女站在天使之下，背景有生命树和知识树。象征爱情、选择与结合。",
    // 7 战车
    "武士驾驶由两只狮身人面兽拉动的战车，手持权杖。象征意志力与胜利。",
    // 8 力量
    "女性温柔地抚摸狮子的头部，头上有无限符号。象征内在力量与慈悲。",
    // 9 隐者
    "灰袍老者手持灯笼和手杖，独立于山顶。象征内省与智慧的寻求。",
    // 10 命运之轮
    "巨大的轮子上有神秘符号，四角有四活物，轮子旁有上升和下降的生物。象征命运循环。",
    // 11 正义
    "女性端坐手持天平与剑，位于两根柱子之间。象征公正与因果。",
    // 12 倒吊人
    "一个人倒挂在T形架上，面容平静，头部发光。象征牺牲与新视角。",
    // 13 死神
    "骑着白马的骷髅手持旗帜，旗上有白玫瑰，众人倒在地上。象征转变与重生。",
    // 14 节制
    "天使站在水边，将水从一个杯子倒入另一个杯子，一脚在水中一脚在陆地。象征平衡与调和。",
    // 15 恶魔
    "有翼的魔鬼坐在基座上，一男一女被锁链锁住。象征束缚与物质欲望。",
    // 16 塔
    "雷电击中高塔，两人从塔顶坠落，火焰和石块四散。象征突变与解放。",
    // 17 星星
    "裸女跪在水边，将水倒入水中和土地，天空有八颗星星。象征希望与灵感。",
    // 18 月亮
    "月亮照耀，狼和狗对月嚎叫，小龙虾从水中爬出，远处有双塔。象征潜意识与恐惧。",
    // 19 太阳
    "灿烂的太阳下，一个孩子骑在白马上，背景是向日葵。象征成功与快乐。",
    // 20 审判
    "天使吹响号角，死者从棺材中复活，举手迎接。象征觉醒与重生。",
    // 21 世界
    "裸女手持两根权杖，被花环环绕，四角有四活物。象征完成与圆满。",
];

/// 大阿卡纳对应星座/行星
///
/// (对应天体/星座, 元素属性)
pub const MAJOR_ARCANA_ASTROLOGY: [(&str, &str); 22] = [
    ("天王星", "风"),      // 0 愚者
    ("水星", "风"),        // 1 魔术师
    ("月亮", "水"),        // 2 女祭司
    ("金星", "土"),        // 3 女皇
    ("白羊座", "火"),      // 4 皇帝
    ("金牛座", "土"),      // 5 教皇
    ("双子座", "风"),      // 6 恋人
    ("巨蟹座", "水"),      // 7 战车
    ("狮子座", "火"),      // 8 力量
    ("处女座", "土"),      // 9 隐者
    ("木星", "火"),        // 10 命运之轮
    ("天秤座", "风"),      // 11 正义
    ("海王星", "水"),      // 12 倒吊人
    ("天蝎座", "水"),      // 13 死神
    ("射手座", "火"),      // 14 节制
    ("摩羯座", "土"),      // 15 恶魔
    ("火星", "火"),        // 16 塔
    ("水瓶座", "风"),      // 17 星星
    ("双鱼座", "水"),      // 18 月亮
    ("太阳", "火"),        // 19 太阳
    ("冥王星", "水"),      // 20 审判
    ("土星", "土"),        // 21 世界
];

/// 大阿卡纳数字象征
///
/// 每个数字的神秘学含义
pub const MAJOR_ARCANA_NUMEROLOGY: [&str; 22] = [
    "0 - 虚空与无限可能",                // 愚者
    "1 - 起点与统一",                    // 魔术师
    "2 - 二元与平衡",                    // 女祭司
    "3 - 创造与表达",                    // 女皇
    "4 - 稳定与结构",                    // 皇帝
    "5 - 变化与自由",                    // 教皇
    "6 - 和谐与责任",                    // 恋人
    "7 - 探索与智慧",                    // 战车
    "8 - 力量与无限",                    // 力量
    "9 - 完成与内省",                    // 隐者
    "10 - 周期与转折 (1+0=1)",           // 命运之轮
    "11 - 主导数字：灵性与直觉 (1+1=2)", // 正义
    "12 - 牺牲与完美 (1+2=3)",           // 倒吊人
    "13 - 转变与重生 (1+3=4)",           // 死神
    "14 - 调和与平衡 (1+4=5)",           // 节制
    "15 - 物质与诱惑 (1+5=6)",           // 恶魔
    "16 - 突变与觉醒 (1+6=7)",           // 塔
    "17 - 希望与灵感 (1+7=8)",           // 星星
    "18 - 幻象与潜意识 (1+8=9)",         // 月亮
    "19 - 成功与光明 (1+9=10=1)",        // 太阳
    "20 - 觉醒与审判 (2+0=2)",           // 审判
    "21 - 完成与世界 (2+1=3)",           // 世界
];

/// 花色名称（中文）
///
/// 索引: 0=无(大阿卡纳), 1=权杖, 2=圣杯, 3=宝剑, 4=星币
pub const SUIT_NAMES_CN: [&str; 5] = ["", "权杖", "圣杯", "宝剑", "星币"];

/// 花色名称（英文）
pub const SUIT_NAMES_EN: [&str; 5] = ["", "Wands", "Cups", "Swords", "Pentacles"];

/// 花色对应元素
pub const SUIT_ELEMENTS: [&str; 5] = ["", "火", "水", "风", "土"];

/// 小阿卡纳数字牌名（中文）
///
/// 索引: 0=Ace(1), 1=二, ..., 9=十
pub const NUMBER_NAMES_CN: [&str; 10] = [
    "Ace", "二", "三", "四", "五", "六", "七", "八", "九", "十",
];

/// 宫廷牌名（中文）
///
/// 索引: 0=侍从(11), 1=骑士(12), 2=王后(13), 3=国王(14)
pub const COURT_NAMES_CN: [&str; 4] = ["侍从", "骑士", "王后", "国王"];

/// 宫廷牌名（英文）
pub const COURT_NAMES_EN: [&str; 4] = ["Page", "Knight", "Queen", "King"];

// ============================================================================
// 小阿卡纳牌义数据
// ============================================================================

/// 小阿卡纳数字牌正位含义
///
/// 按花色和数字组织: [花色][数字]
/// 花色: 0=权杖, 1=圣杯, 2=宝剑, 3=星币
/// 数字: 0=Ace, 1=二, ..., 9=十
pub const MINOR_NUMBER_UPRIGHT: [[&str; 10]; 4] = [
    // 权杖 (Wands) - 火元素，行动、热情、创造力
    [
        "新的开始、创意灵感、热情动力、机会来临、潜能释放",           // Ace
        "计划中、等待时机、决策考量、展望未来、世界在手",             // 二
        "进展顺利、远见卓识、领导力、探索机遇、商业扩展",             // 三
        "庆祝成功、和谐稳定、家庭幸福、里程碑、稳固基础",             // 四
        "竞争激烈、冲突挑战、多方角逐、观点碰撞、健康竞争",           // 五
        "胜利荣耀、公众认可、领袖风范、成就达成、好消息",             // 六
        "坚持立场、捍卫信念、勇气面对、不轻言放弃、挑战重重",         // 七
        "快速行动、迅速发展、消息来临、出行顺利、事情加速",           // 八
        "坚韧不拔、最后防线、警惕防备、经验教训、恢复中",             // 九
        "责任过重、压力巨大、硬撑到底、负担沉重、接近极限",           // 十
    ],
    // 圣杯 (Cups) - 水元素，情感、直觉、关系
    [
        "新感情、情感觉醒、直觉增强、创意灵感、精神满足",             // Ace
        "吸引力、伙伴关系、和谐结合、相互尊重、新恋情",               // 二
        "庆祝友谊、社交聚会、创意合作、共同喜悦、团体和谐",           // 三
        "内省反思、重新评估、错失良机、不满足、寻求更多",             // 四
        "失落悲伤、关注失去、悔恨情绪、需要疗愈、看不到希望",         // 五
        "童年回忆、怀旧情怀、天真美好、故人重逢、旧时光",             // 六
        "幻想迷惑、选择困难、白日梦、诱惑陷阱、虚幻期望",             // 七
        "离开过去、寻找更好、情感转变、放下执念、精神追求",           // 八
        "愿望成真、满足感、情感富足、称心如意、梦想实现",             // 九
        "家庭和睦、情感圆满、幸福美满、内心平静、长久幸福",           // 十
    ],
    // 宝剑 (Swords) - 风元素，思维、沟通、挑战
    [
        "清晰思维、真相大白、新想法、头脑风暴、突破口",               // Ace
        "僵持局面、逃避现实、艰难决定、内心冲突、暂时平衡",           // 二
        "心碎痛苦、悲伤分离、失去、拒绝、情感创伤",                   // 三
        "休养生息、静思冥想、恢复期、暂时退却、内在平静",             // 四
        "冲突失败、自私行为、不光彩胜利、空虚、声誉受损",             // 五
        "过渡期、离开困境、转变中、需要改变、前往未知",               // 六
        "欺骗策略、谨慎行事、秘密计划、投机取巧、需要智慧",           // 七
        "限制束缚、受害者心态、无力感、被困、自我设限",               // 八
        "焦虑噩梦、过度担忧、绝望感、恐惧未来、内心煎熬",             // 九
        "结束痛苦、最坏已过、黎明前、触底反弹、新的开始",             // 十
    ],
    // 星币 (Pentacles) - 土元素，物质、工作、健康
    [
        "新财源、物质机会、繁荣开始、务实计划、身体健康",             // Ace
        "平衡协调、适应变化、灵活应对、多任务处理、优先排序",         // 二
        "团队合作、技能精进、学习成长、工匠精神、质量提升",           // 三
        "财务稳定、保守理财、安全感、控制欲、物质执着",               // 四
        "困难时期、经济困境、孤立无援、贫困担忧、精神贫乏",           // 五
        "慷慨给予、接受帮助、公平交换、施与受、财务和谐",             // 六
        "耐心等待、长期投资、坚持努力、评估进展、收获在即",           // 七
        "专注精进、技能磨练、勤奋工作、细致入微、精益求精",           // 八
        "独立成就、自给自足、奢华享受、财务自由、功成名就",           // 九
        "财富传承、家族兴旺、长久富足、稳定安康、遗产继承",           // 十
    ],
];

/// 小阿卡纳数字牌逆位含义
///
/// 按花色和数字组织
pub const MINOR_NUMBER_REVERSED: [[&str; 10]; 4] = [
    // 权杖 (Wands)
    [
        "创意受阻、延迟开始、热情消退、错失机会、方向不明",           // Ace
        "缺乏计划、恐惧未知、犹豫不决、等待过久、失去平衡",           // 二
        "进展受阻、缺乏远见、错失良机、方向错误、计划失败",           // 三
        "不稳定、缺乏和谐、关系紧张、不满现状、外部不安",             // 四
        "避免冲突、内部竞争、分歧加深、逃避挑战、表面和平",           // 五
        "自我怀疑、虚假成功、名誉受损、不被认可、内心不安",           // 六
        "放弃抵抗、力不从心、被压制、妥协退让、失去勇气",             // 七
        "方向混乱、延误延迟、草率行事、缺乏动力、受阻停滞",           // 八
        "固执己见、偏执多疑、过度防御、不愿改变、孤立无援",           // 九
        "无法承受、崩溃边缘、推卸责任、选择放弃、压力释放",           // 十
    ],
    // 圣杯 (Cups)
    [
        "情感阻塞、创意枯竭、情绪压抑、空虚感、失去联结",             // Ace
        "关系失衡、沟通不畅、分离破裂、不和谐、自我为中心",           // 二
        "过度放纵、孤立自我、创意受阻、社交困难、第三者",             // 三
        "觉醒行动、新机会、不再沉溺、打破僵局、醒悟",                 // 四
        "走出悲伤、接受现实、开始疗愈、看到希望、放下过去",           // 五
        "活在过去、沉溺回忆、不切实际、无法前进、童年创伤",           // 六
        "清醒面对、拒绝诱惑、务实选择、明确方向、不再幻想",           // 七
        "害怕改变、逃避成长、留恋过去、不愿放手、犹豫不决",           // 八
        "不满足、贪得无厌、物质主义、内心空虚、表面满足",             // 九
        "破碎家庭、情感问题、不和谐、关系危机、价值观冲突",           // 十
    ],
    // 宝剑 (Swords)
    [
        "思维混乱、错误判断、恶意用脑、欺骗、自欺",                   // Ace
        "情绪爆发、无法逃避、被迫面对、信息过载、做出选择",           // 二
        "疗愈中、释放痛苦、原谅过去、情感修复、走出阴影",             // 三
        "恢复活力、重新出发、结束休整、打破僵局、行动起来",           // 四
        "和解调停、改变观点、承认错误、放下恩怨、挽回局面",           // 五
        "困在原地、无法改变、拒绝帮助、情绪包袱、不愿前进",           // 六
        "坦诚相待、放弃欺骗、直面真相、改变策略、被揭穿",             // 七
        "释放自我、打破限制、重获自由、找到出路、自我觉醒",           // 八
        "希望重现、焦虑缓解、走出恐惧、光明将至、内心平静",           // 九
        "重生恢复、走出困境、最坏已过、苦尽甘来、渐入佳境",           // 十
    ],
    // 星币 (Pentacles)
    [
        "错失机会、财务问题、计划不周、短视近利、不切实际",           // Ace
        "失去平衡、杂乱无章、财务混乱、难以兼顾、力不从心",           // 二
        "缺乏团队、能力不足、工作问题、质量下降、不被认可",           // 三
        "贪婪执着、过度控制、吝啬小气、物质不安、占有欲",             // 四
        "走出困境、财务恢复、精神觉醒、找到支持、重建信心",           // 五
        "单方付出、不公平、债务问题、施恩图报、有条件的爱",           // 六
        "缺乏耐心、急功近利、放弃太早、收获不佳、方向错误",           // 七
        "敷衍了事、缺乏热情、工作不专注、技能停滞、厌倦工作",         // 八
        "过度奢侈、财务危机、物质依赖、形式主义、虚有其表",           // 九
        "家族问题、遗产纠纷、传统负担、财富流失、根基不稳",           // 十
    ],
];

/// 宫廷牌正位含义
///
/// 按花色和等级组织: [花色][等级]
/// 花色: 0=权杖, 1=圣杯, 2=宝剑, 3=星币
/// 等级: 0=侍从, 1=骑士, 2=王后, 3=国王
pub const COURT_UPRIGHT: [[&str; 4]; 4] = [
    // 权杖 (Wands)
    [
        "热情洋溢、新想法、好消息、探索精神、活力四射",               // 侍从
        "冒险精神、充满热情、行动派、魅力四射、变化来临",             // 骑士
        "自信独立、温暖热情、乐观积极、社交能力、创业精神",           // 王后
        "领袖风范、远见卓识、企业家精神、大局观、诚信正直",           // 国王
    ],
    // 圣杯 (Cups)
    [
        "创意梦想、好消息、直觉敏感、艺术天赋、情感开放",             // 侍从
        "浪漫追求者、理想主义、艺术灵魂、情感丰富、魅力邀请",         // 骑士
        "情感成熟、直觉力强、关怀他人、艺术气质、同理心",             // 王后
        "情感智慧、冷静平衡、外交手腕、艺术鉴赏、慷慨大方",           // 国王
    ],
    // 宝剑 (Swords)
    [
        "好奇心强、新想法、真相探求、警觉敏锐、直言不讳",             // 侍从
        "行动迅速、直接果断、雄心勃勃、战斗精神、追求真理",           // 骑士
        "独立思考、清晰判断、直接沟通、知识丰富、边界清晰",           // 王后
        "清晰思维、权威判断、智慧领导、公正无私、真理化身",           // 国王
    ],
    // 星币 (Pentacles)
    [
        "学习机会、新计划、务实态度、财务消息、脚踏实地",             // 侍从
        "稳定可靠、努力工作、耐心坚持、责任心强、务实进取",           // 骑士
        "实际能干、财务精明、慷慨大方、家庭照顾、物质丰盛",           // 王后
        "财务成功、商业头脑、稳定富足、领导才能、物质大师",           // 国王
    ],
];

/// 宫廷牌逆位含义
pub const COURT_REVERSED: [[&str; 4]; 4] = [
    // 权杖 (Wands)
    [
        "缺乏方向、消息延迟、热情消退、不成熟、三分钟热度",           // 侍从
        "鲁莽冲动、浮躁易怒、缺乏方向、自大傲慢、挫折延迟",           // 骑士
        "自私固执、嫉妒心强、控制欲强、缺乏自信、情绪化",             // 王后
        "专制霸道、高期望、不切实际、冲动决策、傲慢自大",             // 国王
    ],
    // 圣杯 (Cups)
    [
        "情感不成熟、逃避现实、创意受阻、坏消息、过于敏感",           // 侍从
        "情绪化、不切实际、善变无常、花心、逃避承诺",                 // 骑士
        "情绪不稳、过度敏感、依赖他人、情感操控、不切实际",           // 王后
        "情绪压抑、操控欲强、情感疏离、冷漠无情、双重标准",           // 国王
    ],
    // 宝剑 (Swords)
    [
        "搬弄是非、八卦流言、谎言欺骗、轻率鲁莽、心怀恶意",           // 侍从
        "激进极端、言语伤人、缺乏同理、冷酷无情、冲动行事",           // 骑士
        "冷酷无情、过度批评、悲观消极、沟通障碍、情感封闭",           // 王后
        "独裁暴虐、滥用权力、冷酷无情、操控他人、不公正",             // 国王
    ],
    // 星币 (Pentacles)
    [
        "缺乏进展、财务问题、不务实、消息不佳、眼高手低",             // 侍从
        "停滞不前、懒惰散漫、缺乏耐心、不负责任、半途而废",           // 骑士
        "物质至上、过度工作、忽视家庭、自私吝啬、占有欲强",           // 王后
        "物质主义、贪婪腐败、独断专行、过度控制、财务问题",           // 国王
    ],
];

/// 花色元素详细描述
///
/// 每个花色的象征意义和能量特点
pub const SUIT_DESCRIPTIONS: [&str; 4] = [
    // 权杖
    "权杖代表火元素，象征行动力、热情、创造力与精神能量。这组牌关注事业、野心、冒险和个人成长。权杖的能量是积极主动的，鼓励追求目标和实现愿景。",
    // 圣杯
    "圣杯代表水元素，象征情感、直觉、关系与潜意识。这组牌关注爱情、友谊、家庭和情感状态。圣杯的能量是流动的，反映内心世界和人际连接。",
    // 宝剑
    "宝剑代表风元素，象征思维、沟通、冲突与真相。这组牌关注智力、决策、挑战和人生教训。宝剑的能量是锋利的，可以带来清晰也可能带来伤害。",
    // 星币
    "星币代表土元素，象征物质、工作、健康与实际事务。这组牌关注财务、职业、身体和日常生活。星币的能量是稳定的，强调务实和脚踏实地。",
];

/// 数字象征意义
///
/// 1-10 在塔罗中的普遍含义
pub const NUMBER_SYMBOLISM: [&str; 10] = [
    "Ace (1) - 起点、纯粹能量、潜能、新的开始、种子",
    "2 - 二元性、平衡、选择、伙伴关系、对立统一",
    "3 - 表达、创造、成长、扩展、第一个成果",
    "4 - 稳定、结构、基础、暂停、巩固",
    "5 - 挑战、冲突、变化、不稳定、学习的机会",
    "6 - 和谐、平衡、给予与接受、调整、交流",
    "7 - 反思、评估、内在工作、挑战、深化",
    "8 - 行动、力量、运动、掌控、成就",
    "9 - 完成、高潮、独立、智慧、接近终点",
    "10 - 结束与新开始、周期完成、圆满、传承",
];

/// 获取大阿卡纳牌的中文名称
///
/// # 参数
/// - `card_id`: 牌 ID (0-21)
///
/// # 返回
/// - 牌名，如果 ID 超出范围返回 None
pub fn get_major_arcana_name(card_id: u8) -> Option<&'static str> {
    if card_id < 22 {
        Some(MAJOR_ARCANA_NAMES_CN[card_id as usize])
    } else {
        None
    }
}

/// 获取小阿卡纳牌的完整名称
///
/// # 参数
/// - `card_id`: 牌 ID (22-77)
///
/// # 返回
/// - (花色名, 牌面名)，例如 ("权杖", "Ace")
pub fn get_minor_arcana_name(card_id: u8) -> Option<(&'static str, &'static str)> {
    if card_id < 22 || card_id > 77 {
        return None;
    }

    let minor_id = card_id - 22; // 0-55
    let suit_index = (minor_id / 14) as usize + 1; // 1-4
    let card_number = (minor_id % 14) + 1; // 1-14

    let suit_name = if suit_index < SUIT_NAMES_CN.len() {
        SUIT_NAMES_CN[suit_index]
    } else {
        return None;
    };

    let card_name = if card_number <= 10 {
        NUMBER_NAMES_CN[(card_number - 1) as usize]
    } else {
        COURT_NAMES_CN[(card_number - 11) as usize]
    };

    Some((suit_name, card_name))
}

/// 获取任意牌的完整中文名称
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - 完整牌名字符串（静态引用或组合名）
///
/// # 示例
/// - card_id=0 -> "愚者"
/// - card_id=22 -> "权杖Ace"
/// - card_id=35 -> "权杖国王"
pub fn get_card_display_name(card_id: u8) -> (&'static str, Option<&'static str>) {
    if card_id < 22 {
        // 大阿卡纳
        (MAJOR_ARCANA_NAMES_CN[card_id as usize], None)
    } else if card_id < 78 {
        // 小阿卡纳
        if let Some((suit, name)) = get_minor_arcana_name(card_id) {
            (suit, Some(name))
        } else {
            ("未知", None)
        }
    } else {
        ("无效牌", None)
    }
}

/// 获取牌所属的花色索引
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - 花色索引: 0=无(大阿卡纳), 1=权杖, 2=圣杯, 3=宝剑, 4=星币
pub fn get_suit_index(card_id: u8) -> u8 {
    if card_id < 22 {
        0 // 大阿卡纳无花色
    } else if card_id < 78 {
        ((card_id - 22) / 14) + 1
    } else {
        0
    }
}

/// 获取牌的元素属性
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - 元素名称（火/水/风/土），大阿卡纳返回空字符串
pub fn get_card_element(card_id: u8) -> &'static str {
    let suit_index = get_suit_index(card_id) as usize;
    if suit_index < SUIT_ELEMENTS.len() {
        SUIT_ELEMENTS[suit_index]
    } else {
        ""
    }
}

/// 判断牌是否为大阿卡纳
#[inline]
pub fn is_major_arcana(card_id: u8) -> bool {
    card_id < 22
}

/// 判断牌是否为宫廷牌
#[inline]
pub fn is_court_card(card_id: u8) -> bool {
    if card_id < 22 || card_id > 77 {
        return false;
    }
    let number = ((card_id - 22) % 14) + 1;
    number >= 11
}

/// 判断牌是否为数字牌（Ace-10）
#[inline]
pub fn is_number_card(card_id: u8) -> bool {
    if card_id < 22 || card_id > 77 {
        return false;
    }
    let number = ((card_id - 22) % 14) + 1;
    number <= 10
}

// ============================================================================
// 牌义获取函数
// ============================================================================

/// 获取牌的正位含义
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - 正位含义字符串，无效 ID 返回空字符串
pub fn get_upright_meaning(card_id: u8) -> &'static str {
    if card_id < 22 {
        // 大阿卡纳
        MAJOR_ARCANA_UPRIGHT[card_id as usize]
    } else if card_id < 78 {
        // 小阿卡纳
        let minor_id = card_id - 22; // 0-55
        let suit_index = (minor_id / 14) as usize; // 0-3
        let card_number = (minor_id % 14) as usize; // 0-13

        if card_number < 10 {
            // 数字牌 (Ace-10)
            MINOR_NUMBER_UPRIGHT[suit_index][card_number]
        } else {
            // 宫廷牌 (侍从-国王)
            COURT_UPRIGHT[suit_index][card_number - 10]
        }
    } else {
        ""
    }
}

/// 获取牌的逆位含义
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - 逆位含义字符串，无效 ID 返回空字符串
pub fn get_reversed_meaning(card_id: u8) -> &'static str {
    if card_id < 22 {
        // 大阿卡纳
        MAJOR_ARCANA_REVERSED[card_id as usize]
    } else if card_id < 78 {
        // 小阿卡纳
        let minor_id = card_id - 22;
        let suit_index = (minor_id / 14) as usize;
        let card_number = (minor_id % 14) as usize;

        if card_number < 10 {
            MINOR_NUMBER_REVERSED[suit_index][card_number]
        } else {
            COURT_REVERSED[suit_index][card_number - 10]
        }
    } else {
        ""
    }
}

/// 获取牌的关键词
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - 关键词字符串，小阿卡纳返回花色元素相关关键词
pub fn get_keywords(card_id: u8) -> &'static str {
    if card_id < 22 {
        MAJOR_ARCANA_KEYWORDS[card_id as usize]
    } else if card_id < 78 {
        // 小阿卡纳返回正位含义的前半部分作为关键词
        get_upright_meaning(card_id)
    } else {
        ""
    }
}

/// 获取大阿卡纳的描述
///
/// # 参数
/// - `card_id`: 牌 ID (0-21)
///
/// # 返回
/// - 牌面描述，非大阿卡纳返回 None
pub fn get_major_description(card_id: u8) -> Option<&'static str> {
    if card_id < 22 {
        Some(MAJOR_ARCANA_DESCRIPTIONS[card_id as usize])
    } else {
        None
    }
}

/// 获取大阿卡纳的星座/行星对应
///
/// # 参数
/// - `card_id`: 牌 ID (0-21)
///
/// # 返回
/// - (天体/星座名称, 元素属性)，非大阿卡纳返回 None
pub fn get_major_astrology(card_id: u8) -> Option<(&'static str, &'static str)> {
    if card_id < 22 {
        Some(MAJOR_ARCANA_ASTROLOGY[card_id as usize])
    } else {
        None
    }
}

/// 获取大阿卡纳的数字象征
///
/// # 参数
/// - `card_id`: 牌 ID (0-21)
///
/// # 返回
/// - 数字象征意义，非大阿卡纳返回 None
pub fn get_major_numerology(card_id: u8) -> Option<&'static str> {
    if card_id < 22 {
        Some(MAJOR_ARCANA_NUMEROLOGY[card_id as usize])
    } else {
        None
    }
}

/// 获取花色的详细描述
///
/// # 参数
/// - `suit_index`: 花色索引 (1=权杖, 2=圣杯, 3=宝剑, 4=星币)
///
/// # 返回
/// - 花色描述，无效索引返回 None
pub fn get_suit_description(suit_index: u8) -> Option<&'static str> {
    if suit_index >= 1 && suit_index <= 4 {
        Some(SUIT_DESCRIPTIONS[(suit_index - 1) as usize])
    } else {
        None
    }
}

/// 获取数字的象征意义
///
/// # 参数
/// - `number`: 牌面数字 (1-10, 1=Ace)
///
/// # 返回
/// - 数字象征意义，无效数字返回 None
pub fn get_number_symbolism(number: u8) -> Option<&'static str> {
    if number >= 1 && number <= 10 {
        Some(NUMBER_SYMBOLISM[(number - 1) as usize])
    } else {
        None
    }
}

/// 牌义详情结构
///
/// 包含牌的完整信息
#[derive(Clone, Debug)]
pub struct CardMeaning {
    /// 牌名
    pub name: &'static str,
    /// 英文名
    pub name_en: &'static str,
    /// 关键词
    pub keywords: &'static str,
    /// 正位含义
    pub upright: &'static str,
    /// 逆位含义
    pub reversed: &'static str,
    /// 元素
    pub element: &'static str,
    /// 描述（仅大阿卡纳）
    pub description: Option<&'static str>,
    /// 星座/行星（仅大阿卡纳）
    pub astrology: Option<(&'static str, &'static str)>,
}

/// 获取牌的完整牌义信息
///
/// # 参数
/// - `card_id`: 牌 ID (0-77)
///
/// # 返回
/// - CardMeaning 结构，包含所有牌义信息
pub fn get_card_meaning(card_id: u8) -> Option<CardMeaning> {
    if card_id > 77 {
        return None;
    }

    let (name, name_en, element) = if card_id < 22 {
        (
            MAJOR_ARCANA_NAMES_CN[card_id as usize],
            MAJOR_ARCANA_NAMES_EN[card_id as usize],
            MAJOR_ARCANA_ASTROLOGY[card_id as usize].1,
        )
    } else {
        let (suit, _card_name) = get_minor_arcana_name(card_id)?;
        let suit_index = get_suit_index(card_id) as usize;
        let name_en_part = if suit_index > 0 && suit_index <= 4 {
            SUIT_NAMES_EN[suit_index]
        } else {
            ""
        };
        (
            suit,  // 这里简化处理，实际应该组合 suit + card_name
            name_en_part,
            get_card_element(card_id),
        )
    };

    Some(CardMeaning {
        name,
        name_en,
        keywords: get_keywords(card_id),
        upright: get_upright_meaning(card_id),
        reversed: get_reversed_meaning(card_id),
        element,
        description: get_major_description(card_id),
        astrology: get_major_astrology(card_id),
    })
}

// ============================================================================
// 牌阵位置详细信息
// ============================================================================

/// 牌阵位置详情结构
#[derive(Clone, Debug)]
pub struct SpreadPositionInfo {
    /// 位置名称
    pub name: &'static str,
    /// 位置描述
    pub description: &'static str,
    /// 解读指导（当此位置出现牌时的解读方向）
    pub interpretation_guide: &'static str,
}

/// 单张牌牌阵位置详情
pub const SINGLE_CARD_POSITIONS: [SpreadPositionInfo; 1] = [
    SpreadPositionInfo {
        name: "当前指引",
        description: "代表当下最需要关注的讯息",
        interpretation_guide: "这张牌揭示了你当前最需要知道的事情，仔细感受牌面带给你的第一印象，那往往就是宇宙想要告诉你的讯息。",
    },
];

/// 时间三张牌牌阵位置详情
pub const THREE_CARD_TIME_POSITIONS: [SpreadPositionInfo; 3] = [
    SpreadPositionInfo {
        name: "过去",
        description: "影响当前情况的过去事件或能量",
        interpretation_guide: "这张牌代表导致现状的根源，可能是过去的经历、决定或未解决的问题。理解过去有助于更好地把握现在。",
    },
    SpreadPositionInfo {
        name: "现在",
        description: "当前的状况、挑战或需要关注的事项",
        interpretation_guide: "这张牌揭示了你目前所处的位置，以及当下最需要面对的课题。它是连接过去与未来的桥梁。",
    },
    SpreadPositionInfo {
        name: "未来",
        description: "如果继续当前路径可能出现的结果",
        interpretation_guide: "这张牌显示了事情可能的发展方向。记住，未来不是固定的，而是基于当前的选择和行动而变化的。",
    },
];

/// 情况三张牌牌阵位置详情
pub const THREE_CARD_SITUATION_POSITIONS: [SpreadPositionInfo; 3] = [
    SpreadPositionInfo {
        name: "情况",
        description: "问题或情况的本质",
        interpretation_guide: "这张牌揭示了问题的核心是什么，帮助你从更清晰的角度看待当前的情况。",
    },
    SpreadPositionInfo {
        name: "行动",
        description: "建议采取的行动或态度",
        interpretation_guide: "这张牌指示你应该采取的行动或应有的态度，是问题解决方案的关键。",
    },
    SpreadPositionInfo {
        name: "结果",
        description: "采取建议行动后可能的结果",
        interpretation_guide: "这张牌展示了如果你按照行动牌的指引去做，事情可能会如何发展。",
    },
];

/// 爱情关系牌阵位置详情
pub const LOVE_RELATIONSHIP_POSITIONS: [SpreadPositionInfo; 5] = [
    SpreadPositionInfo {
        name: "你的感受",
        description: "你在这段关系中的感受和态度",
        interpretation_guide: "这张牌反映了你对这段关系的真实感受，包括你可能没有意识到的深层情感。",
    },
    SpreadPositionInfo {
        name: "对方的感受",
        description: "对方在这段关系中可能的感受",
        interpretation_guide: "这张牌代表对方的感受和态度，帮助你从另一个角度理解这段关系。",
    },
    SpreadPositionInfo {
        name: "关系现状",
        description: "关系目前的状态和能量",
        interpretation_guide: "这张牌显示了两人之间当前的互动模式和关系氛围。",
    },
    SpreadPositionInfo {
        name: "挑战",
        description: "关系面临的挑战或需要克服的障碍",
        interpretation_guide: "这张牌揭示了影响关系发展的障碍或需要共同面对的问题。",
    },
    SpreadPositionInfo {
        name: "未来发展",
        description: "关系可能的发展方向",
        interpretation_guide: "这张牌指示了关系可能的走向，以及你们可以共同期待的未来。",
    },
];

/// 事业指导牌阵位置详情
pub const CAREER_GUIDANCE_POSITIONS: [SpreadPositionInfo; 6] = [
    SpreadPositionInfo {
        name: "当前状况",
        description: "你目前的职业状态和环境",
        interpretation_guide: "这张牌反映了你当前的工作环境、职业状态或事业发展阶段。",
    },
    SpreadPositionInfo {
        name: "优势",
        description: "你在职业发展中的优势和资源",
        interpretation_guide: "这张牌揭示了你可以利用的优势、技能或机会，是成功的基础。",
    },
    SpreadPositionInfo {
        name: "挑战",
        description: "需要面对的职业挑战",
        interpretation_guide: "这张牌指出了阻碍你职业发展的因素或需要克服的困难。",
    },
    SpreadPositionInfo {
        name: "机会",
        description: "潜在的职业机会",
        interpretation_guide: "这张牌揭示了即将到来或可以创造的机会，需要你保持警觉。",
    },
    SpreadPositionInfo {
        name: "建议行动",
        description: "建议采取的职业行动",
        interpretation_guide: "这张牌指示了你应该采取的具体行动或态度调整。",
    },
    SpreadPositionInfo {
        name: "未来前景",
        description: "事业发展的可能方向",
        interpretation_guide: "这张牌展示了按照建议行动后，事业可能的发展前景。",
    },
];

/// 决策分析牌阵位置详情
pub const DECISION_MAKING_POSITIONS: [SpreadPositionInfo; 7] = [
    SpreadPositionInfo {
        name: "当前情况",
        description: "你面临的决策情境",
        interpretation_guide: "这张牌帮助你理解当前需要做出决定的背景和原因。",
    },
    SpreadPositionInfo {
        name: "选择A",
        description: "第一个选择的本质",
        interpretation_guide: "这张牌揭示了第一个选择的核心特质和能量。",
    },
    SpreadPositionInfo {
        name: "选择A结果",
        description: "选择A的可能结果",
        interpretation_guide: "这张牌显示了如果选择第一个选项，可能会带来的结果。",
    },
    SpreadPositionInfo {
        name: "选择B",
        description: "第二个选择的本质",
        interpretation_guide: "这张牌揭示了第二个选择的核心特质和能量。",
    },
    SpreadPositionInfo {
        name: "选择B结果",
        description: "选择B的可能结果",
        interpretation_guide: "这张牌显示了如果选择第二个选项，可能会带来的结果。",
    },
    SpreadPositionInfo {
        name: "外在影响",
        description: "影响决策的外部因素",
        interpretation_guide: "这张牌揭示了可能影响你决定的外在因素或他人的影响。",
    },
    SpreadPositionInfo {
        name: "最佳建议",
        description: "关于这个决定的整体建议",
        interpretation_guide: "这张牌提供了关于如何做出最佳决定的指导和智慧。",
    },
];

/// 凯尔特十字牌阵位置详情
pub const CELTIC_CROSS_POSITIONS: [SpreadPositionInfo; 10] = [
    SpreadPositionInfo {
        name: "当前状况",
        description: "问题的核心，你目前的处境",
        interpretation_guide: "这是牌阵的核心，代表问题或情况的本质。它揭示了你当前所处的位置。",
    },
    SpreadPositionInfo {
        name: "挑战",
        description: "横跨的挑战或需要面对的障碍",
        interpretation_guide: "这张牌代表你目前面临的主要挑战或障碍，它与核心牌相互作用。",
    },
    SpreadPositionInfo {
        name: "远因",
        description: "问题的根源或远因",
        interpretation_guide: "这张牌揭示了导致当前情况的深层原因，可能是过去的事件或潜意识的因素。",
    },
    SpreadPositionInfo {
        name: "近因",
        description: "最近发生的影响当前状况的事件",
        interpretation_guide: "这张牌代表近期发生的、正在影响当前情况的事件或能量。",
    },
    SpreadPositionInfo {
        name: "可能结果",
        description: "最佳可能的结果",
        interpretation_guide: "这张牌显示了事情可能达到的最好结果，是你可以努力实现的目标。",
    },
    SpreadPositionInfo {
        name: "近期发展",
        description: "即将发生或正在发展的事情",
        interpretation_guide: "这张牌揭示了在不久的将来可能发生的事情或即将进入你生活的能量。",
    },
    SpreadPositionInfo {
        name: "你的态度",
        description: "你对情况的态度和立场",
        interpretation_guide: "这张牌反映了你对当前情况的态度、恐惧或期望，是内在的视角。",
    },
    SpreadPositionInfo {
        name: "外在影响",
        description: "外部环境和他人的影响",
        interpretation_guide: "这张牌代表来自外界的影响，包括他人的看法、环境因素等。",
    },
    SpreadPositionInfo {
        name: "内心期望",
        description: "你内心深处的希望或恐惧",
        interpretation_guide: "这张牌揭示了你潜意识中的希望或恐惧，它们会影响事情的发展。",
    },
    SpreadPositionInfo {
        name: "最终结果",
        description: "事情最可能的最终结果",
        interpretation_guide: "这张牌显示了综合所有因素后，事情最可能的发展结果。记住，结果可以通过行动来改变。",
    },
];

/// 年度运势牌阵位置详情
pub const YEAR_FORECAST_POSITIONS: [SpreadPositionInfo; 12] = [
    SpreadPositionInfo { name: "一月", description: "一月的运势主题", interpretation_guide: "这张牌揭示了一月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "二月", description: "二月的运势主题", interpretation_guide: "这张牌揭示了二月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "三月", description: "三月的运势主题", interpretation_guide: "这张牌揭示了三月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "四月", description: "四月的运势主题", interpretation_guide: "这张牌揭示了四月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "五月", description: "五月的运势主题", interpretation_guide: "这张牌揭示了五月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "六月", description: "六月的运势主题", interpretation_guide: "这张牌揭示了六月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "七月", description: "七月的运势主题", interpretation_guide: "这张牌揭示了七月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "八月", description: "八月的运势主题", interpretation_guide: "这张牌揭示了八月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "九月", description: "九月的运势主题", interpretation_guide: "这张牌揭示了九月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "十月", description: "十月的运势主题", interpretation_guide: "这张牌揭示了十月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "十一月", description: "十一月的运势主题", interpretation_guide: "这张牌揭示了十一月的主要能量和可能发生的事件。" },
    SpreadPositionInfo { name: "十二月", description: "十二月的运势主题", interpretation_guide: "这张牌揭示了十二月的主要能量和可能发生的事件。" },
];

/// 获取牌阵位置详情
///
/// # 参数
/// - `spread_type`: 牌阵类型索引
///   - 1: 单张牌
///   - 3: 时间三张牌
///   - 4: 情况三张牌
///   - 5: 爱情关系
///   - 6: 事业指导
///   - 7: 决策分析
///   - 10: 凯尔特十字
///   - 12: 年度运势
/// - `position`: 位置索引（0-based）
///
/// # 返回
/// - 位置详情，如果参数无效返回 None
pub fn get_spread_position_info(spread_type: u8, position: usize) -> Option<&'static SpreadPositionInfo> {
    match spread_type {
        1 => SINGLE_CARD_POSITIONS.get(position),
        3 => THREE_CARD_TIME_POSITIONS.get(position),
        4 => THREE_CARD_SITUATION_POSITIONS.get(position),
        5 => LOVE_RELATIONSHIP_POSITIONS.get(position),
        6 => CAREER_GUIDANCE_POSITIONS.get(position),
        7 => DECISION_MAKING_POSITIONS.get(position),
        10 => CELTIC_CROSS_POSITIONS.get(position),
        12 => YEAR_FORECAST_POSITIONS.get(position),
        _ => None,
    }
}

/// 获取牌阵的所有位置详情
pub fn get_spread_all_positions(spread_type: u8) -> Option<&'static [SpreadPositionInfo]> {
    match spread_type {
        1 => Some(&SINGLE_CARD_POSITIONS),
        3 => Some(&THREE_CARD_TIME_POSITIONS),
        4 => Some(&THREE_CARD_SITUATION_POSITIONS),
        5 => Some(&LOVE_RELATIONSHIP_POSITIONS),
        6 => Some(&CAREER_GUIDANCE_POSITIONS),
        7 => Some(&DECISION_MAKING_POSITIONS),
        10 => Some(&CELTIC_CROSS_POSITIONS),
        12 => Some(&YEAR_FORECAST_POSITIONS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_major_arcana_names() {
        assert_eq!(get_major_arcana_name(0), Some("愚者"));
        assert_eq!(get_major_arcana_name(21), Some("世界"));
        assert_eq!(get_major_arcana_name(22), None);
    }

    #[test]
    fn test_minor_arcana_names() {
        // 权杖 Ace (card_id = 22)
        assert_eq!(get_minor_arcana_name(22), Some(("权杖", "Ace")));

        // 权杖国王 (card_id = 35)
        assert_eq!(get_minor_arcana_name(35), Some(("权杖", "国王")));

        // 圣杯 Ace (card_id = 36)
        assert_eq!(get_minor_arcana_name(36), Some(("圣杯", "Ace")));

        // 星币国王 (card_id = 77)
        assert_eq!(get_minor_arcana_name(77), Some(("星币", "国王")));

        // 无效 ID
        assert_eq!(get_minor_arcana_name(21), None);
        assert_eq!(get_minor_arcana_name(78), None);
    }

    #[test]
    fn test_card_display_name() {
        assert_eq!(get_card_display_name(0), ("愚者", None));
        assert_eq!(get_card_display_name(22), ("权杖", Some("Ace")));
        assert_eq!(get_card_display_name(77), ("星币", Some("国王")));
    }

    #[test]
    fn test_suit_index() {
        assert_eq!(get_suit_index(0), 0); // 大阿卡纳
        assert_eq!(get_suit_index(22), 1); // 权杖
        assert_eq!(get_suit_index(36), 2); // 圣杯
        assert_eq!(get_suit_index(50), 3); // 宝剑
        assert_eq!(get_suit_index(64), 4); // 星币
    }

    #[test]
    fn test_card_element() {
        assert_eq!(get_card_element(0), ""); // 大阿卡纳无元素
        assert_eq!(get_card_element(22), "火"); // 权杖
        assert_eq!(get_card_element(36), "水"); // 圣杯
        assert_eq!(get_card_element(50), "风"); // 宝剑
        assert_eq!(get_card_element(64), "土"); // 星币
    }

    #[test]
    fn test_card_type_checks() {
        // 大阿卡纳
        assert!(is_major_arcana(0));
        assert!(is_major_arcana(21));
        assert!(!is_major_arcana(22));

        // 宫廷牌
        assert!(!is_court_card(0)); // 大阿卡纳
        assert!(!is_court_card(22)); // 权杖 Ace
        assert!(is_court_card(32)); // 权杖侍从
        assert!(is_court_card(35)); // 权杖国王

        // 数字牌
        assert!(!is_number_card(0)); // 大阿卡纳
        assert!(is_number_card(22)); // 权杖 Ace
        assert!(is_number_card(31)); // 权杖十
        assert!(!is_number_card(32)); // 权杖侍从
    }

    #[test]
    fn test_major_arcana_meanings() {
        // 测试愚者的牌义
        let upright = get_upright_meaning(0);
        assert!(upright.contains("新的开始"));
        assert!(upright.contains("冒险"));

        let reversed = get_reversed_meaning(0);
        assert!(reversed.contains("鲁莽"));

        let keywords = get_keywords(0);
        assert!(keywords.contains("自由"));

        // 测试世界的牌义
        let upright = get_upright_meaning(21);
        assert!(upright.contains("圆满"));

        // 测试描述
        let desc = get_major_description(0);
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("悬崖"));

        // 测试星座对应
        let astro = get_major_astrology(0);
        assert!(astro.is_some());
        assert_eq!(astro.unwrap().0, "天王星");
    }

    #[test]
    fn test_minor_arcana_meanings() {
        // 权杖 Ace (card_id = 22)
        let upright = get_upright_meaning(22);
        assert!(upright.contains("创意"));

        // 圣杯 Ace (card_id = 36)
        let upright = get_upright_meaning(36);
        assert!(upright.contains("情感"));

        // 宝剑 Ace (card_id = 50)
        let upright = get_upright_meaning(50);
        assert!(upright.contains("清晰"));

        // 星币 Ace (card_id = 64)
        let upright = get_upright_meaning(64);
        assert!(upright.contains("财源"));
    }

    #[test]
    fn test_court_card_meanings() {
        // 权杖侍从 (card_id = 32)
        let upright = get_upright_meaning(32);
        assert!(upright.contains("热情"));

        // 圣杯国王 (card_id = 49)
        let upright = get_upright_meaning(49);
        assert!(upright.contains("情感"));

        // 宝剑王后 (card_id = 62)
        let upright = get_upright_meaning(62);
        assert!(upright.contains("独立"));

        // 星币国王 (card_id = 77)
        let upright = get_upright_meaning(77);
        assert!(upright.contains("财务"));
    }

    #[test]
    fn test_get_card_meaning() {
        // 测试大阿卡纳
        let meaning = get_card_meaning(0);
        assert!(meaning.is_some());
        let m = meaning.unwrap();
        assert_eq!(m.name, "愚者");
        assert_eq!(m.name_en, "The Fool");
        assert!(m.description.is_some());
        assert!(m.astrology.is_some());

        // 测试小阿卡纳
        let meaning = get_card_meaning(22);
        assert!(meaning.is_some());
        let m = meaning.unwrap();
        assert_eq!(m.name, "权杖");
        assert_eq!(m.element, "火");
        assert!(m.description.is_none());

        // 测试无效 ID
        assert!(get_card_meaning(78).is_none());
    }

    #[test]
    fn test_suit_description() {
        assert!(get_suit_description(1).unwrap().contains("火元素"));
        assert!(get_suit_description(2).unwrap().contains("水元素"));
        assert!(get_suit_description(3).unwrap().contains("风元素"));
        assert!(get_suit_description(4).unwrap().contains("土元素"));
        assert!(get_suit_description(0).is_none());
        assert!(get_suit_description(5).is_none());
    }

    #[test]
    fn test_number_symbolism() {
        assert!(get_number_symbolism(1).unwrap().contains("起点"));
        assert!(get_number_symbolism(10).unwrap().contains("周期完成"));
        assert!(get_number_symbolism(0).is_none());
        assert!(get_number_symbolism(11).is_none());
    }
}
