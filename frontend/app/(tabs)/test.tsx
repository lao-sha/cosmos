import { useState } from 'react';
import { Platform, Pressable, StyleSheet, Text, View } from 'react-native';

export default function TestScreen() {
  const [count, setCount] = useState(0);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>最小化点击测试</Text>
      <Text style={styles.count}>点击次数: {count}</Text>
      
      <Pressable
        style={styles.button}
        onPress={() => {
          setCount(c => c + 1);
          console.log('Pressable clicked:', count + 1);
        }}
      >
        <Text style={styles.buttonText}>Pressable 按钮</Text>
      </Pressable>

      {Platform.OS === 'web' && (
        <button
          type="button"
          onClick={() => {
            setCount(c => c + 1);
            console.log('HTML button clicked:', count + 1);
            window.alert('HTML 按钮点击成功！次数: ' + (count + 1));
          }}
          style={{
            backgroundColor: '#DC2626',
            color: 'white',
            padding: '16px 32px',
            borderRadius: 8,
            border: 'none',
            cursor: 'pointer',
            fontWeight: 'bold',
            marginTop: 12,
            fontSize: 16,
          }}
        >
          HTML Button (点我测试)
        </button>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
    backgroundColor: '#f5f5f5',
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 10,
    color: '#1f2937',
  },
  count: {
    fontSize: 18,
    marginBottom: 30,
    color: '#6b7280',
  },
  button: {
    backgroundColor: '#6D28D9',
    paddingVertical: 16,
    paddingHorizontal: 32,
    borderRadius: 12,
    marginBottom: 12,
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
});
