import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
    Linking,
    Pressable,
    ScrollView,
    StyleSheet,
    Text,
    TextInput,
    View,
} from 'react-native';

interface FaqItem {
  id: string;
  question: string;
  answer: string;
  category: string;
}

const FAQ_DATA: FaqItem[] = [
  {
    id: '1',
    category: '账户',
    question: '如何创建钱包？',
    answer: '点击"个人中心" -> "创建钱包"，系统会自动生成助记词。请务必妥善保管助记词，这是恢复钱包的唯一方式。',
  },
  {
    id: '2',
    category: '账户',
    question: '忘记助记词怎么办？',
    answer: '助记词是恢复钱包的唯一凭证，如果丢失将无法找回。建议在安全的地方备份助记词，切勿截图或在线存储。',
  },
  {
    id: '3',
    category: '会员',
    question: '如何升级会员？',
    answer: '进入"会员中心"，选择想要的会员套餐，完成支付即可升级。系统会自动计算差价并延长有效期。',
  },
  {
    id: '4',
    category: '会员',
    question: 'COS奖励如何使用？',
    answer: 'COS是平台积分，可用于抵扣服务费用、兑换礼品等。通过签到、完成任务、邀请好友等方式获取。',
  },
  {
    id: '5',
    category: '占卜',
    question: '如何预约占卜服务？',
    answer: '在"占卜市场"选择占卜师，查看其服务套餐和评价，选择合适的套餐后支付即可预约。',
  },
  {
    id: '6',
    category: '占卜',
    question: '占卜结果不满意怎么办？',
    answer: '如对服务质量有异议，可在订单详情页发起申诉。平台会介入调解，必要时提交仲裁委员会裁决。',
  },
  {
    id: '7',
    category: 'OTC',
    question: 'OTC交易如何保障安全？',
    answer: '平台采用托管机制，买家付款后卖家确认收款才会释放代币。如有争议可申请仲裁。',
  },
  {
    id: '8',
    category: 'OTC',
    question: '交易对手不放币怎么办？',
    answer: '付款后如果对方长时间不确认，可发起申诉。提供付款凭证后，平台会介入处理。',
  },
  {
    id: '9',
    category: '合婚',
    question: '合婚匹配是如何计算的？',
    answer: '基于传统命理学中的日柱合婚、五行互补、性格匹配等多维度综合分析，由算法自动计算匹配度。',
  },
  {
    id: '10',
    category: '隐私',
    question: '我的八字信息安全吗？',
    answer: '所有敏感信息都经过端到端加密存储在区块链上，只有你本人才能解密查看。平台无法获取你的明文数据。',
  },
];

const CATEGORIES = ['全部', '账户', '会员', '占卜', 'OTC', '合婚', '隐私'];

export default function HelpScreen() {
  const router = useRouter();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('全部');
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const filteredFaq = FAQ_DATA.filter((item) => {
    const matchCategory = selectedCategory === '全部' || item.category === selectedCategory;
    const matchSearch = item.question.includes(searchQuery) || item.answer.includes(searchQuery);
    return matchCategory && (searchQuery === '' || matchSearch);
  });

  const handleContact = () => {
    Linking.openURL('mailto:support@cosmos.app');
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>帮助中心</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.searchContainer}>
          <TextInput
            style={styles.searchInput}
            placeholder="搜索问题..."
            placeholderTextColor="#9ca3af"
            value={searchQuery}
            onChangeText={setSearchQuery}
          />
        </View>

        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          style={styles.categoryScroll}
          contentContainerStyle={styles.categoryContent}
        >
          {CATEGORIES.map((category) => (
            <Pressable
              key={category}
              style={[
                styles.categoryButton,
                selectedCategory === category && styles.categoryActive,
              ]}
              onPress={() => setSelectedCategory(category)}
            >
              <Text
                style={[
                  styles.categoryText,
                  selectedCategory === category && styles.categoryTextActive,
                ]}
              >
                {category}
              </Text>
            </Pressable>
          ))}
        </ScrollView>

        <View style={styles.faqSection}>
          <Text style={styles.sectionTitle}>常见问题</Text>
          {filteredFaq.map((item) => (
            <Pressable
              key={item.id}
              style={styles.faqItem}
              onPress={() => setExpandedId(expandedId === item.id ? null : item.id)}
            >
              <View style={styles.faqHeader}>
                <Text style={styles.faqQuestion}>{item.question}</Text>
                <Text style={styles.faqToggle}>
                  {expandedId === item.id ? '−' : '+'}
                </Text>
              </View>
              {expandedId === item.id && (
                <Text style={styles.faqAnswer}>{item.answer}</Text>
              )}
            </Pressable>
          ))}
          {filteredFaq.length === 0 && (
            <Text style={styles.noResults}>未找到相关问题</Text>
          )}
        </View>

        <View style={styles.contactSection}>
          <Text style={styles.contactTitle}>没有找到答案？</Text>
          <Text style={styles.contactDesc}>
            如果以上内容未能解决你的问题，请联系客服获取帮助
          </Text>
          <Pressable style={styles.contactButton} onPress={handleContact}>
            <Text style={styles.contactButtonText}>📧 联系客服</Text>
          </Pressable>
        </View>

        <View style={styles.linksSection}>
          <Pressable style={styles.linkItem} onPress={() => router.push('/legal/terms')}>
            <Text style={styles.linkIcon}>📄</Text>
            <Text style={styles.linkText}>服务条款</Text>
            <Text style={styles.linkArrow}>›</Text>
          </Pressable>
          <Pressable style={styles.linkItem} onPress={() => router.push('/legal/privacy')}>
            <Text style={styles.linkIcon}>🔒</Text>
            <Text style={styles.linkText}>隐私政策</Text>
            <Text style={styles.linkArrow}>›</Text>
          </Pressable>
          <Pressable style={styles.linkItem} onPress={() => router.push('/wallet')}>
            <Text style={styles.linkIcon}>📘</Text>
            <Text style={styles.linkText}>使用教程</Text>
            <Text style={styles.linkArrow}>›</Text>
          </Pressable>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#fff',
    paddingTop: 50,
    paddingBottom: 12,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  backButton: {
    padding: 4,
  },
  backText: {
    fontSize: 17,
    color: '#6D28D9',
  },
  headerTitle: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
  },
  headerRight: {
    width: 50,
  },
  content: {
    flex: 1,
  },
  searchContainer: {
    padding: 16,
    backgroundColor: '#fff',
  },
  searchInput: {
    backgroundColor: '#f3f4f6',
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 12,
    fontSize: 15,
    color: '#1f2937',
  },
  categoryScroll: {
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e5e7eb',
  },
  categoryContent: {
    paddingHorizontal: 12,
    paddingVertical: 12,
    gap: 8,
  },
  categoryButton: {
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
    backgroundColor: '#f3f4f6',
  },
  categoryActive: {
    backgroundColor: '#6D28D9',
  },
  categoryText: {
    fontSize: 14,
    color: '#6b7280',
  },
  categoryTextActive: {
    color: '#fff',
    fontWeight: '600',
  },
  faqSection: {
    padding: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 12,
  },
  faqItem: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 16,
    marginBottom: 8,
  },
  faqHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  faqQuestion: {
    flex: 1,
    fontSize: 15,
    fontWeight: '500',
    color: '#1f2937',
    paddingRight: 12,
  },
  faqToggle: {
    fontSize: 20,
    color: '#6D28D9',
    fontWeight: '600',
  },
  faqAnswer: {
    fontSize: 14,
    color: '#6b7280',
    lineHeight: 22,
    marginTop: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: '#f3f4f6',
  },
  noResults: {
    textAlign: 'center',
    color: '#9ca3af',
    paddingVertical: 24,
  },
  contactSection: {
    backgroundColor: '#f5f3ff',
    margin: 16,
    borderRadius: 16,
    padding: 20,
    alignItems: 'center',
  },
  contactTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 8,
  },
  contactDesc: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    marginBottom: 16,
  },
  contactButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  contactButtonText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
  linksSection: {
    backgroundColor: '#fff',
    marginHorizontal: 16,
    marginBottom: 32,
    borderRadius: 12,
    overflow: 'hidden',
  },
  linkItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#f3f4f6',
  },
  linkIcon: {
    fontSize: 18,
    marginRight: 12,
  },
  linkText: {
    flex: 1,
    fontSize: 15,
    color: '#1f2937',
  },
  linkArrow: {
    fontSize: 18,
    color: '#d1d5db',
  },
});
