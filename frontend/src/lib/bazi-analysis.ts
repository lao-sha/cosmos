/**
 * 星尘玄鉴 - 八字分析功能模块
 * 包含 8 个可扩展的分析功能
 */

// ============================================================================
// 1. 大运分析 (Major Luck Periods)
// ============================================================================

export interface DaYunPeriod {
  startAge: number;
  endAge: number;
  gan: number;
  zhi: number;
  wuxing: string;
  description: string;
}

export const calculateDaYun = (dayMaster: number, monthGan: number): DaYunPeriod[] => {
  // 大运按照月干推算，每 10 年一个大运
  const daYunList: DaYunPeriod[] = [];

  for (let i = 0; i < 8; i++) {
    const ganIndex = (monthGan + i + 1) % 10;
    const zhiIndex = (i) % 12;
    const wuxingList = ['木', '木', '火', '火', '土', '土', '金', '金', '水', '水'];

    daYunList.push({
      startAge: i * 10,
      endAge: (i + 1) * 10,
      gan: ganIndex,
      zhi: zhiIndex,
      wuxing: wuxingList[ganIndex],
      description: `第${i + 1}大运 (${i * 10}-${(i + 1) * 10}岁)`,
    });
  }

  return daYunList;
};

// ============================================================================
// 2. 流年分析 (Annual Luck)
// ============================================================================

export interface LiuNianData {
  year: number;
  gan: number;
  zhi: number;
  wuxing: string;
  description: string;
}

export const calculateLiuNian = (currentYear: number, count: number = 10): LiuNianData[] => {
  const liuNianList: LiuNianData[] = [];
  const wuxingList = ['木', '木', '火', '火', '土', '土', '金', '金', '水', '水'];

  for (let i = 0; i < count; i++) {
    const year = currentYear + i;
    const ganIndex = (year - 4) % 10;
    const zhiIndex = (year - 4) % 12;

    liuNianList.push({
      year,
      gan: ganIndex,
      zhi: zhiIndex,
      wuxing: wuxingList[ganIndex],
      description: `${year}年`,
    });
  }

  return liuNianList;
};

// ============================================================================
// 3. 神煞分析 (Auspicious/Inauspicious Stars)
// ============================================================================

export interface ShenSha {
  name: string;
  type: 'auspicious' | 'inauspicious' | 'neutral';
  description: string;
  location: string;
}

export const calculateShenSha = (
  yearZhi: number,
  monthZhi: number,
  dayZhi: number,
  hourZhi: number
): ShenSha[] => {
  const shenShaList: ShenSha[] = [];

  // 吉神
  const auspiciousStars = [
    { name: '天乙贵人', type: 'auspicious' as const, description: '贵人相助，逢凶化吉' },
    { name: '月德贵人', type: 'auspicious' as const, description: '月德庇护，诸事顺利' },
    { name: '天德贵人', type: 'auspicious' as const, description: '天德护佑，福泽深厚' },
    { name: '文昌贵人', type: 'auspicious' as const, description: '文昌加持，聪慧过人' },
    { name: '天赦', type: 'auspicious' as const, description: '天赦赦免，过错消解' },
  ];

  // 凶神
  const inauspiciousStars = [
    { name: '羊刃', type: 'inauspicious' as const, description: '羊刃凶险，需要谨慎' },
    { name: '劫煞', type: 'inauspicious' as const, description: '劫煞侵扰，防范灾祸' },
    { name: '灾煞', type: 'inauspicious' as const, description: '灾煞临身，需要化解' },
    { name: '孤辰', type: 'inauspicious' as const, description: '孤辰寡宿，人缘欠佳' },
    { name: '寡宿', type: 'inauspicious' as const, description: '孤辰寡宿，感情坎坷' },
  ];

  // 随机选择一些神煞
  const selectedAuspicious = auspiciousStars.slice(0, Math.floor(Math.random() * 3) + 1);
  const selectedInauspicious = inauspiciousStars.slice(0, Math.floor(Math.random() * 2) + 1);

  return [
    ...selectedAuspicious.map(star => ({
      ...star,
      location: '命盘',
    })),
    ...selectedInauspicious.map(star => ({
      ...star,
      location: '命盘',
    })),
  ];
};

// ============================================================================
// 4. 格局判断 (Chart Pattern Analysis)
// ============================================================================

export interface GeJuAnalysis {
  name: string;
  level: 'superior' | 'good' | 'average' | 'poor';
  description: string;
  characteristics: string[];
}

export const analyzeGeJu = (
  dayMaster: number,
  wuxingCount: Record<string, number>
): GeJuAnalysis => {
  const TIAN_GAN_WUXING = ['木', '木', '火', '火', '土', '土', '金', '金', '水', '水'];
  const dayMasterWuxing = TIAN_GAN_WUXING[dayMaster];

  // 计算五行平衡度
  const total = Object.values(wuxingCount).reduce((a, b) => a + (b || 0), 0);
  const balance = Object.values(wuxingCount).map(count => (count || 0) / total);
  const variance = balance.reduce((sum, val) => sum + Math.pow(val - 0.2, 2), 0);

  let level: 'superior' | 'good' | 'average' | 'poor';
  let name: string;

  if (variance < 0.01) {
    level = 'superior';
    name = '五行平衡格';
  } else if (variance < 0.02) {
    level = 'good';
    name = '五行较平衡格';
  } else if (variance < 0.04) {
    level = 'average';
    name = '五行一般格';
  } else {
    level = 'poor';
    name = '五行失衡格';
  }

  const characteristics = [
    `日主属${dayMasterWuxing}`,
    `五行方差: ${variance.toFixed(4)}`,
    `最多元素: ${Object.entries(wuxingCount).sort((a, b) => (b[1] || 0) - (a[1] || 0))[0][0]}`,
  ];

  return {
    name,
    level,
    description: `这是一个${name}，${level === 'superior' ? '五行平衡，命运较为顺利' : level === 'good' ? '五行较为平衡，整体运势不错' : level === 'average' ? '五行分布一般，需要调理' : '五行失衡，需要特别关注'}。`,
    characteristics,
  };
};

// ============================================================================
// 5. 十干禄位分析 (Heavenly Stem Wealth Position)
// ============================================================================

export interface LuWeiAnalysis {
  gan: number;
  luPosition: string;
  description: string;
}

export const analyzeLuWei = (dayMaster: number): LuWeiAnalysis => {
  const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
  const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

  // 十干禄位对应
  const luWeiMap: Record<number, number> = {
    0: 10, // 甲禄在寅
    1: 10, // 乙禄在卯
    2: 6,  // 丙禄在巳
    3: 6,  // 丁禄在午
    4: 8,  // 戊禄在午
    5: 8,  // 己禄在午
    6: 2,  // 庚禄在申
    7: 2,  // 辛禄在酉
    8: 0,  // 壬禄在亥
    9: 0,  // 癸禄在子
  };

  const luZhiIndex = luWeiMap[dayMaster];
  const luPosition = DI_ZHI[luZhiIndex];

  return {
    gan: dayMaster,
    luPosition,
    description: `${TIAN_GAN[dayMaster]}天干禄位在${luPosition}，代表财富和权力的位置。`,
  };
};

// ============================================================================
// 6. 纳音五行分析 (Nayin Five Elements)
// ============================================================================

export interface NaYinAnalysis {
  year: { gan: number; zhi: number; nayin: string };
  month: { gan: number; zhi: number; nayin: string };
  day: { gan: number; zhi: number; nayin: string };
  hour: { gan: number; zhi: number; nayin: string };
  description: string;
}

export const analyzeNaYin = (
  yearGan: number,
  yearZhi: number,
  monthGan: number,
  monthZhi: number,
  dayGan: number,
  dayZhi: number,
  hourGan: number,
  hourZhi: number
): NaYinAnalysis => {
  const TIAN_GAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
  const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

  // 纳音五行对应
  const nayinMap: Record<string, string> = {
    '甲子': '海中金', '乙丑': '海中金',
    '丙寅': '炉中火', '丁卯': '炉中火',
    '戊辰': '大林木', '己巳': '大林木',
    '庚午': '路旁土', '辛未': '路旁土',
    '壬申': '剑锋金', '癸酉': '剑锋金',
    '甲戌': '山头火', '乙亥': '山头火',
    '丙子': '洞下水', '丁丑': '洞下水',
    '戊寅': '城头土', '己卯': '城头土',
    '庚辰': '白腊金', '辛巳': '白腊金',
    '壬午': '杨柳木', '癸未': '杨柳木',
    '甲申': '泉中水', '乙酉': '泉中水',
    '丙戌': '屋上土', '丁亥': '屋上土',
    '戊子': '霹雳火', '己丑': '霹雳火',
    '庚寅': '松柏木', '辛卯': '松柏木',
    '壬辰': '长流水', '癸巳': '长流水',
    '甲午': '沙中金', '乙未': '沙中金',
    '丙申': '山下火', '丁酉': '山下火',
    '戊戌': '平地木', '己亥': '平地木',
  };

  const getNayin = (gan: number, zhi: number): string => {
    const key = TIAN_GAN[gan] + DI_ZHI[zhi];
    return nayinMap[key] || '未知';
  };

  return {
    year: { gan: yearGan, zhi: yearZhi, nayin: getNayin(yearGan, yearZhi) },
    month: { gan: monthGan, zhi: monthZhi, nayin: getNayin(monthGan, monthZhi) },
    day: { gan: dayGan, zhi: dayZhi, nayin: getNayin(dayGan, dayZhi) },
    hour: { gan: hourGan, zhi: hourZhi, nayin: getNayin(hourGan, hourZhi) },
    description: '纳音五行代表八字的音律属性，反映命主的气质和运势特征。',
  };
};

// ============================================================================
// 7. 柱间关系分析 (Pillar Relationships)
// ============================================================================

export interface PillarRelationship {
  pillar1: string;
  pillar2: string;
  relationship: 'harmony' | 'conflict' | 'punishment' | 'destruction' | 'none';
  description: string;
}

export const analyzePillarRelationships = (
  yearZhi: number,
  monthZhi: number,
  dayZhi: number,
  hourZhi: number
): PillarRelationship[] => {
  const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

  // 地支六合关系
  const harmonyMap: Record<number, number> = {
    0: 1,   // 子丑合
    1: 0,   // 丑子合
    2: 7,   // 寅未合
    3: 6,   // 卯午合
    4: 9,   // 辰酉合
    5: 10,  // 巳戌合
    6: 3,   // 午卯合
    7: 2,   // 未寅合
    8: 5,   // 申巳合
    9: 4,   // 酉辰合
    10: 5,  // 戌巳合
    11: 8,  // 亥申合
  };

  // 地支相冲关系
  const conflictMap: Record<number, number> = {
    0: 6,   // 子午冲
    1: 7,   // 丑未冲
    2: 8,   // 寅申冲
    3: 9,   // 卯酉冲
    4: 10,  // 辰戌冲
    5: 11,  // 巳亥冲
    6: 0,   // 午子冲
    7: 1,   // 未丑冲
    8: 2,   // 申寅冲
    9: 3,   // 酉卯冲
    10: 4,  // 戌辰冲
    11: 5,  // 亥巳冲
  };

  const relationships: PillarRelationship[] = [];
  const pillars = [
    { name: '年柱', zhi: yearZhi },
    { name: '月柱', zhi: monthZhi },
    { name: '日柱', zhi: dayZhi },
    { name: '时柱', zhi: hourZhi },
  ];

  for (let i = 0; i < pillars.length; i++) {
    for (let j = i + 1; j < pillars.length; j++) {
      const zhi1 = pillars[i].zhi;
      const zhi2 = pillars[j].zhi;

      let relationship: 'harmony' | 'conflict' | 'punishment' | 'destruction' | 'none' = 'none';
      let description = '无特殊关系';

      if (harmonyMap[zhi1] === zhi2) {
        relationship = 'harmony';
        description = `${DI_ZHI[zhi1]}${DI_ZHI[zhi2]}相合，相互帮助`;
      } else if (conflictMap[zhi1] === zhi2) {
        relationship = 'conflict';
        description = `${DI_ZHI[zhi1]}${DI_ZHI[zhi2]}相冲，相互克制`;
      }

      relationships.push({
        pillar1: pillars[i].name,
        pillar2: pillars[j].name,
        relationship,
        description,
      });
    }
  }

  return relationships;
};

// ============================================================================
// 8. 命宫分析 (Life Palace Analysis)
// ============================================================================

export interface MingGongAnalysis {
  mingGongZhi: number;
  description: string;
  characteristics: string[];
  fortuneLevel: 'excellent' | 'good' | 'average' | 'poor';
}

export const analyzeMingGong = (monthZhi: number, hourZhi: number): MingGongAnalysis => {
  const DI_ZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];

  // 命宫 = (月支 + 时支) % 12
  const mingGongZhi = (monthZhi + hourZhi) % 12;

  const characteristics: Record<number, string[]> = {
    0: ['聪慧', '机灵', '善于交际'],
    1: ['稳重', '踏实', '财运不错'],
    2: ['勇敢', '进取', '事业心强'],
    3: ['温和', '善良', '人缘好'],
    4: ['聪明', '多才', '变化多'],
    5: ['热情', '积极', '社交广'],
    6: ['尊贵', '权势', '领导力强'],
    7: ['温柔', '体贴', '家庭观念强'],
    8: ['聪慧', '机敏', '适应力强'],
    9: ['高雅', '气质', '品味独特'],
    10: ['稳定', '可靠', '责任心强'],
    11: ['神秘', '深沉', '思想深邃'],
  };

  const fortuneLevels: Record<number, 'excellent' | 'good' | 'average' | 'poor'> = {
    0: 'good',
    1: 'excellent',
    2: 'good',
    3: 'good',
    4: 'average',
    5: 'good',
    6: 'excellent',
    7: 'good',
    8: 'good',
    9: 'excellent',
    10: 'good',
    11: 'average',
  };

  return {
    mingGongZhi,
    description: `命宫在${DI_ZHI[mingGongZhi]}，代表命主的先天禀赋和人生基调。`,
    characteristics: characteristics[mingGongZhi] || [],
    fortuneLevel: fortuneLevels[mingGongZhi] || 'average',
  };
};
