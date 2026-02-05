import { useColorScheme } from 'react-native';
import { Colors } from '@/constants/colors';

export function useColors() {
  const scheme = useColorScheme();
  const isDark = scheme === 'dark';

  return {
    ...Colors,
    ...(isDark ? Colors.dark : Colors.light),
    isDark,
  };
}

export function useThemeColor(
  props: { light?: string; dark?: string },
  colorName: keyof typeof Colors.light & keyof typeof Colors.dark
) {
  const scheme = useColorScheme() ?? 'light';
  const colorFromProps = props[scheme];

  if (colorFromProps) {
    return colorFromProps;
  }
  return Colors[scheme][colorName];
}
