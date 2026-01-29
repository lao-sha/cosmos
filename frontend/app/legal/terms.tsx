import { useRouter } from 'expo-router';
import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

export default function TermsScreen() {
  const router = useRouter();

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Pressable style={styles.backButton} onPress={() => router.back()}>
          <Text style={styles.backText}>‹ 返回</Text>
        </Pressable>
        <Text style={styles.headerTitle}>服务条款</Text>
        <View style={styles.headerRight} />
      </View>

      <ScrollView style={styles.content} showsVerticalScrollIndicator={false}>
        <View style={styles.section}>
          <Text style={styles.updateDate}>最后更新：2024年1月1日</Text>
          
          <Text style={styles.title}>Cosmos 服务条款</Text>
          
          <Text style={styles.paragraph}>
            欢迎使用 Cosmos（以下简称"本平台"）。在使用本平台提供的任何服务之前，请您仔细阅读并理解以下服务条款。使用本平台即表示您同意接受这些条款的约束。
          </Text>

          <Text style={styles.heading}>1. 服务说明</Text>
          <Text style={styles.paragraph}>
            1.1 本平台是一个基于区块链技术的去中心化应用，提供玄学咨询、即时通讯、代币交易等服务。
          </Text>
          <Text style={styles.paragraph}>
            1.2 本平台上的占卜、命理分析等服务仅供参考和娱乐目的，不构成任何形式的专业建议。
          </Text>
          <Text style={styles.paragraph}>
            1.3 用户应当自行负责保管其钱包助记词和私钥，本平台不存储用户的私钥信息。
          </Text>

          <Text style={styles.heading}>2. 用户账户</Text>
          <Text style={styles.paragraph}>
            2.1 用户通过创建或导入钱包来使用本平台服务，钱包地址即为用户的唯一标识。
          </Text>
          <Text style={styles.paragraph}>
            2.2 用户应妥善保管账户信息，因用户保管不善导致的损失由用户自行承担。
          </Text>
          <Text style={styles.paragraph}>
            2.3 用户不得将账户用于任何非法目的或违反本条款的活动。
          </Text>

          <Text style={styles.heading}>3. 服务费用</Text>
          <Text style={styles.paragraph}>
            3.1 使用本平台的某些服务可能需要支付费用，包括但不限于占卜服务费、会员订阅费等。
          </Text>
          <Text style={styles.paragraph}>
            3.2 所有链上交易均需支付网络手续费（Gas费），该费用由区块链网络收取。
          </Text>
          <Text style={styles.paragraph}>
            3.3 费用标准可能会根据市场情况调整，调整前会提前通知用户。
          </Text>

          <Text style={styles.heading}>4. 用户行为规范</Text>
          <Text style={styles.paragraph}>
            用户在使用本平台时，同意不进行以下行为：
          </Text>
          <Text style={styles.listItem}>• 发布虚假、欺诈或误导性信息</Text>
          <Text style={styles.listItem}>• 侵犯他人知识产权或隐私权</Text>
          <Text style={styles.listItem}>• 进行洗钱或其他金融犯罪活动</Text>
          <Text style={styles.listItem}>• 干扰或破坏平台正常运行</Text>
          <Text style={styles.listItem}>• 未经授权访问其他用户的账户或数据</Text>

          <Text style={styles.heading}>5. 知识产权</Text>
          <Text style={styles.paragraph}>
            5.1 本平台的软件、设计、商标、标识等均为本平台或其授权方所有。
          </Text>
          <Text style={styles.paragraph}>
            5.2 用户在平台上发布的内容，用户保留其知识产权，但授予本平台展示和传播的许可。
          </Text>

          <Text style={styles.heading}>6. 免责声明</Text>
          <Text style={styles.paragraph}>
            6.1 本平台上的占卜服务仅供娱乐和参考，用户应当理性对待，不应将其作为重大决策的唯一依据。
          </Text>
          <Text style={styles.paragraph}>
            6.2 本平台不对因市场波动、网络故障、不可抗力等原因导致的损失承担责任。
          </Text>
          <Text style={styles.paragraph}>
            6.3 本平台不对用户与第三方之间的交易或纠纷承担责任。
          </Text>

          <Text style={styles.heading}>7. 隐私保护</Text>
          <Text style={styles.paragraph}>
            用户的隐私保护详见《隐私政策》，该政策构成本服务条款的组成部分。
          </Text>

          <Text style={styles.heading}>8. 服务变更与终止</Text>
          <Text style={styles.paragraph}>
            8.1 本平台有权根据业务发展需要，变更、暂停或终止部分或全部服务。
          </Text>
          <Text style={styles.paragraph}>
            8.2 用户违反本条款的，本平台有权暂停或终止向其提供服务。
          </Text>

          <Text style={styles.heading}>9. 争议解决</Text>
          <Text style={styles.paragraph}>
            9.1 本条款的解释和适用以中华人民共和国法律为准。
          </Text>
          <Text style={styles.paragraph}>
            9.2 因本条款产生的争议，双方应友好协商解决；协商不成的，可向本平台所在地有管辖权的人民法院提起诉讼。
          </Text>

          <Text style={styles.heading}>10. 条款修改</Text>
          <Text style={styles.paragraph}>
            本平台有权根据需要修改本服务条款，修改后的条款将在平台上公布。用户继续使用本平台服务即视为接受修改后的条款。
          </Text>

          <View style={styles.footer}>
            <Text style={styles.footerText}>
              如有任何疑问，请联系客服或发送邮件至 support@cosmos.app
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
  footer: {
    marginTop: 32,
    paddingTop: 20,
    borderTopWidth: 1,
    borderTopColor: '#e5e7eb',
  },
  footerText: {
    fontSize: 13,
    color: '#9ca3af',
    textAlign: 'center',
  },
});
