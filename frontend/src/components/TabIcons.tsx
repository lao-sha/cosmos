/**
 * 简易图标组件（使用文本符号）
 * 后续可替换为 @expo/vector-icons
 */

import { Text } from 'react-native';

interface IconProps {
  color: string;
  size: number;
}

export function HomeOutline({ color, size }: IconProps) {
  return <Text style={{ color, fontSize: size }}>🏠</Text>;
}

export function CompassOutline({ color, size }: IconProps) {
  return <Text style={{ color, fontSize: size }}>🧭</Text>;
}

export function MessageOutline({ color, size }: IconProps) {
  return <Text style={{ color, fontSize: size }}>💬</Text>;
}

export function UserOutline({ color, size }: IconProps) {
  return <Text style={{ color, fontSize: size }}>👤</Text>;
}
