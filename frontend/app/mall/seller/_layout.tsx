import { Stack } from 'expo-router';

export default function SellerLayout() {
  return (
    <Stack>
      <Stack.Screen name="index" options={{ title: '卖家中心' }} />
      <Stack.Screen name="create" options={{ title: '创建店铺' }} />
      <Stack.Screen name="settings" options={{ title: '店铺设置' }} />
      <Stack.Screen name="products" options={{ title: '商品管理' }} />
      <Stack.Screen name="product-create" options={{ title: '发布商品' }} />
      <Stack.Screen name="orders" options={{ title: '订单管理' }} />
      <Stack.Screen name="token" options={{ title: '代币管理' }} />
      <Stack.Screen name="members" options={{ title: '会员管理' }} />
      <Stack.Screen name="commission" options={{ title: '返佣配置' }} />
    </Stack>
  );
}
