import { useRouter } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

export default function PrivacyPolicyScreen() {
  const router = useRouter();

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>隐私政策</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.section}>
          <Text style={styles.updateDate}>最后更新：2024年1月1日</Text>
          
          <Text style={styles.title}>Cosmos 隐私政策</Text>
          
          <Text style={styles.paragraph}>
            本隐私政策说明了 Cosmos（以下简称"我们"或"本平台"）如何收集、使用、存储和保护您的个人信息。我们非常重视您的隐私，并致力于保护您的个人数据安全。
          </Text>

          <Text style={styles.heading}>1. 信息收集</Text>
          <Text style={styles.subHeading}>1.1 您主动提供的信息</Text>
          <Text style={styles.listItem}>• 钱包地址（公钥）</Text>
          <Text style={styles.listItem}>• 个人资料信息（昵称、头像、简介等）</Text>
          <Text style={styles.listItem}>• 出生信息（用于占卜服务，仅在您同意时收集）</Text>
          <Text style={styles.listItem}>• 聊天消息内容</Text>

          <Text style={styles.subHeading}>1.2 自动收集的信息</Text>
          <Text style={styles.listItem}>• 设备信息（设备类型、操作系统版本）</Text>
          <Text style={styles.listItem}>• 应用使用数据（功能使用频率、崩溃日志）</Text>
          <Text style={styles.listItem}>• 区块链交易记录（公开链上数据）</Text>

          <Text style={styles.subHeading}>1.3 我们不收集的信息</Text>
          <Text style={styles.listItem}>• 助记词或私钥（仅存储在您的设备本地）</Text>
          <Text style={styles.listItem}>• 精确地理位置</Text>
          <Text style={styles.listItem}>• 通讯录或联系人信息</Text>

          <Text style={styles.heading}>2. 信息使用</Text>
          <Text style={styles.paragraph}>
            我们收集的信息用于以下目的：
          </Text>
          <Text style={styles.listItem}>• 提供和改进我们的服务</Text>
          <Text style={styles.listItem}>• 处理您的交易和订单</Text>
          <Text style={styles.listItem}>• 提供客户支持</Text>
          <Text style={styles.listItem}>• 发送服务通知和更新</Text>
          <Text style={styles.listItem}>• 防范欺诈和滥用行为</Text>
          <Text style={styles.listItem}>• 遵守法律法规要求</Text>

          <Text style={styles.heading}>3. 数据存储与安全</Text>
          <Text style={styles.subHeading}>3.1 本地存储</Text>
          <Text style={styles.paragraph}>
            您的助记词和私钥仅加密存储在您的设备本地，我们无法访问这些信息。
          </Text>

          <Text style={styles.subHeading}>3.2 链上数据</Text>
          <Text style={styles.paragraph}>
            部分数据（如交易记录、智能合约交互）存储在区块链上，这些数据是公开透明的。
          </Text>

          <Text style={styles.subHeading}>3.3 IPFS 存储</Text>
          <Text style={styles.paragraph}>
            聊天消息和部分用户数据经过加密后存储在 IPFS 分布式网络中，只有拥有密钥的用户才能解密查看。
          </Text>

          <Text style={styles.subHeading}>3.4 安全措施</Text>
          <Text style={styles.listItem}>• 端到端加密通讯</Text>
          <Text style={styles.listItem}>• 安全存储（Secure Store）保护敏感数据</Text>
          <Text style={styles.listItem}>• 定期安全审计</Text>

          <Text style={styles.heading}>4. 信息共享</Text>
          <Text style={styles.paragraph}>
            我们不会出售您的个人信息。在以下情况下，我们可能会共享您的信息：
          </Text>
          <Text style={styles.listItem}>• 经您明确同意</Text>
          <Text style={styles.listItem}>• 与服务提供商合作（如占卜师提供服务）</Text>
          <Text style={styles.listItem}>• 遵守法律要求或响应合法的政府请求</Text>
          <Text style={styles.listItem}>• 保护我们的权利和财产</Text>

          <Text style={styles.heading}>5. 您的权利</Text>
          <Text style={styles.paragraph}>
            您对个人信息享有以下权利：
          </Text>
          <Text style={styles.listItem}>• <Text style={styles.bold}>访问权</Text>：查看我们持有的您的个人信息</Text>
          <Text style={styles.listItem}>• <Text style={styles.bold}>更正权</Text>：更正不准确的个人信息</Text>
          <Text style={styles.listItem}>• <Text style={styles.bold}>删除权</Text>：请求删除您的个人信息</Text>
          <Text style={styles.listItem}>• <Text style={styles.bold}>数据可携权</Text>：获取您数据的副本</Text>
          <Text style={styles.listItem}>• <Text style={styles.bold}>撤回同意</Text>：撤回之前给予的同意</Text>

          <Text style={styles.heading}>6. Cookie 和追踪技术</Text>
          <Text style={styles.paragraph}>
            我们可能使用 Cookie 和类似技术来：
          </Text>
          <Text style={styles.listItem}>• 记住您的偏好设置</Text>
          <Text style={styles.listItem}>• 分析应用使用情况</Text>
          <Text style={styles.listItem}>• 提供个性化体验</Text>
          <Text style={styles.paragraph}>
            您可以在设备设置中管理这些偏好。
          </Text>

          <Text style={styles.heading}>7. 第三方服务</Text>
          <Text style={styles.paragraph}>
            本平台可能包含指向第三方服务的链接。我们不对这些第三方服务的隐私实践负责。建议您在使用前查阅其隐私政策。
          </Text>

          <Text style={styles.heading}>8. 儿童隐私</Text>
          <Text style={styles.paragraph}>
            本平台不面向 18 岁以下的用户。我们不会故意收集未成年人的个人信息。如果我们发现收集了未成年人的信息，将立即删除。
          </Text>

          <Text style={styles.heading}>9. 国际数据传输</Text>
          <Text style={styles.paragraph}>
            由于区块链和 IPFS 的分布式特性，您的数据可能存储在全球各地的节点上。我们会确保数据传输符合适用的数据保护法规。
          </Text>

          <Text style={styles.heading}>10. 隐私政策更新</Text>
          <Text style={styles.paragraph}>
            我们可能会不时更新本隐私政策。重大变更时，我们会通过应用内通知或其他方式告知您。继续使用本平台即表示您接受更新后的政策。
          </Text>

          <Text style={styles.heading}>11. 联系我们</Text>
          <Text style={styles.paragraph}>
            如果您对本隐私政策有任何疑问或需要行使您的权利，请通过以下方式联系我们：
          </Text>
          <Text style={styles.contactItem}>📧 邮箱：privacy@cosmos.app</Text>
          <Text style={styles.contactItem}>💬 应用内客服</Text>

          <View style={styles.footer}>
            <Text style={styles.footerText}>
              感谢您信任 Cosmos。我们承诺保护您的隐私。
            </Text>
          </View>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
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
  section: {
    padding: 20,
  },
  updateDate: {
    fontSize: 13,
    color: '#9ca3af',
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#1f2937',
    marginBottom: 20,
  },
  heading: {
    fontSize: 17,
    fontWeight: '600',
    color: '#1f2937',
    marginTop: 24,
    marginBottom: 12,
  },
  subHeading: {
    fontSize: 15,
    fontWeight: '600',
    color: '#4b5563',
    marginTop: 16,
    marginBottom: 8,
  },
  paragraph: {
    fontSize: 15,
    color: '#4b5563',
    lineHeight: 24,
    marginBottom: 12,
  },
  listItem: {
    fontSize: 15,
    color: '#4b5563',
    lineHeight: 24,
    marginLeft: 8,
    marginBottom: 4,
  },
  bold: {
    fontWeight: '600',
    color: '#1f2937',
  },
  contactItem: {
    fontSize: 15,
    color: '#6D28D9',
    lineHeight: 28,
    marginLeft: 8,
  },
  footer: {
    marginTop: 32,
    paddingTop: 20,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  footerText: {
    fontSize: 14,
    color: '#6b7280',
    textAlign: 'center',
    fontStyle: 'italic',
  },
});
