import { Stack } from 'expo-router';

export default function MeowstarLayout() {
  return (
    <Stack
      screenOptions={{
        headerStyle: {
          backgroundColor: '#1a1a2e',
        },
        headerTintColor: '#fff',
        headerTitleStyle: {
          fontWeight: 'bold',
        },
      }}
    >
      <Stack.Screen
        name="index"
        options={{
          title: '喵星宇宙',
          headerShown: true,
        }}
      />
      <Stack.Screen
        name="pets"
        options={{
          title: '我的宠物',
        }}
      />
      <Stack.Screen
        name="pet/[id]"
        options={{
          title: '宠物详情',
        }}
      />
      <Stack.Screen
        name="battle"
        options={{
          title: '战斗',
        }}
      />
      <Stack.Screen
        name="marketplace"
        options={{
          title: '市场',
        }}
      />
      <Stack.Screen
        name="staking"
        options={{
          title: '质押',
        }}
      />
      <Stack.Screen
        name="governance"
        options={{
          title: 'DAO 治理',
        }}
      />
      <Stack.Screen
        name="chat"
        options={{
          title: 'AI 陪伴',
        }}
      />
    </Stack>
  );
}
