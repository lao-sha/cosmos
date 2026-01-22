/**
 * 简化版首页 - 用于测试
 */

import { View, Text, StyleSheet } from 'react-native';

export default function SimpleHomePage() {
  console.log('[SimpleHomePage] Rendering...');
  
  return (
    <View style={styles.container}>
      <Text style={styles.title}>✅ 应用已启动</Text>
      <Text style={styles.text}>如果你看到这个页面，说明应用正常运行了</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0a0f',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  title: {
    fontSize: 32,
    color: '#00ff00',
    marginBottom: 20,
    fontWeight: 'bold',
  },
  text: {
    fontSize: 16,
    color: '#fff',
    textAlign: 'center',
  },
});
