import { View, Text, StyleSheet } from 'react-native';

export default function TestSimple() {
  return (
    <View style={styles.container}>
      <Text style={styles.text}>测试页面 - 如果你看到这个，说明应用正常运行</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
  text: {
    fontSize: 18,
    color: '#333',
  },
});
