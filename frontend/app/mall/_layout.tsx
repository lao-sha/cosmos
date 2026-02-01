import { Stack } from 'expo-router';

export default function MallLayout() {
  return (
    <Stack>
      <Stack.Screen name="shops" options={{ title: '全部店铺' }} />
      <Stack.Screen name="orders" options={{ title: '我的订单' }} />
      <Stack.Screen name="market" options={{ title: '代币交易' }} />
      <Stack.Screen name="seller" options={{ title: '卖家中心', headerShown: false }} />
      <Stack.Screen name="shop/[id]" options={{ title: '店铺详情' }} />
      <Stack.Screen name="product/[id]" options={{ title: '商品详情' }} />
      <Stack.Screen name="checkout" options={{ title: '确认订单' }} />
      <Stack.Screen name="order/[id]" options={{ title: '订单详情' }} />
      <Stack.Screen name="search" options={{ title: '搜索结果' }} />
    </Stack>
  );
}
