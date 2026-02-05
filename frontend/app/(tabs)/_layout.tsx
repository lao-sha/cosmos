import { Tabs } from 'expo-router';
import { useColorScheme } from 'react-native';
import { Wallet, Repeat, MessageCircle, User } from 'lucide-react-native';
import { Colors } from '@/constants/colors';

export default function TabLayout() {
  const colorScheme = useColorScheme();
  const isDark = colorScheme === 'dark';
  const theme = isDark ? Colors.dark : Colors.light;

  return (
    <Tabs
      screenOptions={{
        tabBarActiveTintColor: Colors.primary,
        tabBarInactiveTintColor: theme.textTertiary,
        tabBarStyle: {
          backgroundColor: theme.background,
          borderTopColor: theme.border,
        },
        headerStyle: {
          backgroundColor: theme.background,
        },
        headerTintColor: theme.textPrimary,
      }}
    >
      <Tabs.Screen
        name="index"
        options={{
          title: '钱包',
          tabBarIcon: ({ color, size }) => <Wallet size={size} color={color} />,
        }}
      />
      <Tabs.Screen
        name="trade"
        options={{
          title: '交易',
          tabBarIcon: ({ color, size }) => <Repeat size={size} color={color} />,
        }}
      />
      <Tabs.Screen
        name="chat"
        options={{
          title: '消息',
          tabBarIcon: ({ color, size }) => (
            <MessageCircle size={size} color={color} />
          ),
        }}
      />
      <Tabs.Screen
        name="profile"
        options={{
          title: '我的',
          tabBarIcon: ({ color, size }) => <User size={size} color={color} />,
        }}
      />
    </Tabs>
  );
}
