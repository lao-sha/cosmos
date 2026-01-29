import { useState } from 'react';
import { Pressable, StyleSheet, Text, View } from 'react-native';

interface AddressDisplayProps {
  address: string;
  label?: string;
  showFull?: boolean;
  size?: 'small' | 'medium' | 'large';
}

export function AddressDisplay({
  address,
  label,
  showFull = false,
  size = 'medium',
}: AddressDisplayProps) {
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(showFull);

  const truncateAddress = (addr: string) => {
    if (addr.length <= 16) return addr;
    return `${addr.slice(0, 8)}...${addr.slice(-8)}`;
  };

  const handleCopy = async () => {
    try {
      // Web fallback
      if (typeof navigator !== 'undefined' && navigator.clipboard) {
        await navigator.clipboard.writeText(address);
      }
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Copy failed:', e);
    }
  };

  const displayAddress = expanded ? address : truncateAddress(address);

  const sizeStyles = {
    small: { fontSize: 12, padding: 8 },
    medium: { fontSize: 14, padding: 12 },
    large: { fontSize: 16, padding: 16 },
  };

  return (
    <View style={styles.container}>
      {label && <Text style={styles.label}>{label}</Text>}
      <View style={[styles.addressBox, { padding: sizeStyles[size].padding }]}>
        <Pressable
          style={styles.addressContent}
          onPress={() => setExpanded(!expanded)}
        >
          <Text
            style={[styles.address, { fontSize: sizeStyles[size].fontSize }]}
            selectable
          >
            {displayAddress}
          </Text>
        </Pressable>
        <Pressable style={styles.copyButton} onPress={handleCopy}>
          <Text style={styles.copyIcon}>{copied ? '✓' : '📋'}</Text>
        </Pressable>
      </View>
      {!showFull && (
        <Text style={styles.hint}>
          {expanded ? '点击地址收起' : '点击地址展开完整'}
        </Text>
      )}
    </View>
  );
}

interface QRAddressDisplayProps {
  address: string;
  label?: string;
}

export function QRAddressDisplay({ address, label }: QRAddressDisplayProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      if (typeof navigator !== 'undefined' && navigator.clipboard) {
        await navigator.clipboard.writeText(address);
      }
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Copy failed:', e);
    }
  };

  return (
    <View style={styles.qrContainer}>
      {label && <Text style={styles.qrLabel}>{label}</Text>}
      
      <View style={styles.qrPlaceholder}>
        <Text style={styles.qrEmoji}>📱</Text>
        <Text style={styles.qrText}>二维码</Text>
      </View>

      <View style={styles.qrAddressBox}>
        <Text style={styles.qrAddress} selectable numberOfLines={2}>
          {address}
        </Text>
      </View>

      <Pressable style={styles.qrCopyButton} onPress={handleCopy}>
        <Text style={styles.qrCopyText}>
          {copied ? '✓ 已复制' : '复制地址'}
        </Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {},
  label: {
    fontSize: 14,
    color: '#6b7280',
    marginBottom: 8,
  },
  addressBox: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#f9fafb',
    borderRadius: 12,
    borderWidth: 1,
    borderColor: '#e5e7eb',
  },
  addressContent: {
    flex: 1,
  },
  address: {
    fontFamily: 'monospace',
    color: '#1f2937',
  },
  copyButton: {
    padding: 8,
    marginLeft: 8,
  },
  copyIcon: {
    fontSize: 16,
  },
  hint: {
    fontSize: 11,
    color: '#9ca3af',
    marginTop: 4,
    textAlign: 'center',
  },
  qrContainer: {
    alignItems: 'center',
    padding: 20,
  },
  qrLabel: {
    fontSize: 16,
    fontWeight: '600',
    color: '#1f2937',
    marginBottom: 16,
  },
  qrPlaceholder: {
    width: 180,
    height: 180,
    backgroundColor: '#f3f4f6',
    borderRadius: 12,
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16,
  },
  qrEmoji: {
    fontSize: 48,
    marginBottom: 8,
  },
  qrText: {
    fontSize: 14,
    color: '#9ca3af',
  },
  qrAddressBox: {
    backgroundColor: '#f9fafb',
    borderRadius: 8,
    padding: 12,
    width: '100%',
    marginBottom: 16,
  },
  qrAddress: {
    fontSize: 12,
    fontFamily: 'monospace',
    color: '#4b5563',
    textAlign: 'center',
  },
  qrCopyButton: {
    backgroundColor: '#6D28D9',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  qrCopyText: {
    color: '#fff',
    fontSize: 15,
    fontWeight: '600',
  },
});
