// frontend/src/divination/market/components/PriceDisplay.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { THEME } from '../theme';
import { formatBalance } from '../utils/market.utils';
import { TOKEN_SYMBOL } from '../constants/market.constants';

interface PriceDisplayProps {
  amount: bigint | string | number;
  size?: 'small' | 'medium' | 'large';
  showSymbol?: boolean;
  color?: string;
  strikethrough?: boolean;
}

export const PriceDisplay: React.FC<PriceDisplayProps> = ({
  amount,
  size = 'medium',
  showSymbol = true,
  color,
  strikethrough = false,
}) => {
  const formattedAmount = formatBalance(amount);

  const sizeStyles = {
    small: { fontSize: 12, symbolSize: 10 },
    medium: { fontSize: 16, symbolSize: 12 },
    large: { fontSize: 20, symbolSize: 14 },
  };

  const s = sizeStyles[size];
  const textColor = color || THEME.primary;

  return (
    <View style={styles.container}>
      <Text
        style={[
          styles.amount,
          {
            fontSize: s.fontSize,
            color: textColor,
            textDecorationLine: strikethrough ? 'line-through' : 'none',
          },
        ]}
      >
        {formattedAmount}
      </Text>
      {showSymbol && (
        <Text
          style={[
            styles.symbol,
            {
              fontSize: s.symbolSize,
              color: textColor,
              textDecorationLine: strikethrough ? 'line-through' : 'none',
            },
          ]}
        >
          {TOKEN_SYMBOL}
        </Text>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'baseline',
  },
  amount: {
    fontWeight: '600',
  },
  symbol: {
    marginLeft: 2,
    fontWeight: '500',
  },
});
