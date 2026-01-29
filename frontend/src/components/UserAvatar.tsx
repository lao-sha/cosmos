import { Image, StyleSheet, Text, View } from 'react-native';

interface UserAvatarProps {
  name: string;
  uri?: string;
  size?: 'small' | 'medium' | 'large';
  isOnline?: boolean;
}

const SIZES = {
  small: { container: 32, text: 14, dot: 8 },
  medium: { container: 48, text: 18, dot: 12 },
  large: { container: 72, text: 28, dot: 16 },
};

const COLORS = [
  '#6D28D9', '#dc2626', '#16a34a', '#2563eb', 
  '#d97706', '#7c3aed', '#0891b2', '#be185d',
];

function getColorFromName(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  return COLORS[Math.abs(hash) % COLORS.length];
}

export function UserAvatar({ name, uri, size = 'medium', isOnline }: UserAvatarProps) {
  const dimensions = SIZES[size];
  const backgroundColor = getColorFromName(name);
  const initial = name.charAt(0).toUpperCase();

  return (
    <View style={[styles.container, { width: dimensions.container, height: dimensions.container }]}>
      {uri ? (
        <Image
          source={{ uri }}
          style={[
            styles.image,
            { 
              width: dimensions.container, 
              height: dimensions.container,
              borderRadius: dimensions.container / 2,
            }
          ]}
        />
      ) : (
        <View
          style={[
            styles.placeholder,
            { 
              width: dimensions.container, 
              height: dimensions.container,
              borderRadius: dimensions.container / 2,
              backgroundColor,
            }
          ]}
        >
          <Text style={[styles.initial, { fontSize: dimensions.text }]}>
            {initial}
          </Text>
        </View>
      )}
      
      {isOnline && (
        <View
          style={[
            styles.onlineIndicator,
            {
              width: dimensions.dot,
              height: dimensions.dot,
              borderRadius: dimensions.dot / 2,
              borderWidth: dimensions.dot / 4,
            }
          ]}
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'relative',
  },
  image: {
    backgroundColor: '#e5e7eb',
  },
  placeholder: {
    justifyContent: 'center',
    alignItems: 'center',
  },
  initial: {
    color: '#fff',
    fontWeight: '600',
  },
  onlineIndicator: {
    position: 'absolute',
    bottom: 0,
    right: 0,
    backgroundColor: '#22c55e',
    borderColor: '#fff',
  },
});
