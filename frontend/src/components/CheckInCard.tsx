import { useState } from 'react';
import { ActivityIndicator, Pressable, StyleSheet, Text, View } from 'react-native';

interface CheckInCardProps {
  streak: number;
  lastCheckIn?: string;
  reward: number;
  multiplier: number;
  onCheckIn?: () => Promise<void>;
}

export function CheckInCard({
  streak,
  lastCheckIn,
  reward,
  multiplier,
  onCheckIn,
}: CheckInCardProps) {
  const [loading, setLoading] = useState(false);
  const [checked, setChecked] = useState(false);

  const isCheckedToday = lastCheckIn === new Date().toISOString().split('T')[0];

  const handleCheckIn = async () => {
    if (loading || isCheckedToday || checked) return;
    setLoading(true);
    try {
      await onCheckIn?.();
      setChecked(true);
    } catch (error) {
      console.error('Check-in failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const weekDays = ['一', '二', '三', '四', '五', '六', '日'];
  const currentDayOfWeek = (new Date().getDay() + 6) % 7;

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <View>
          <Text style={styles.title}>每日签到</Text>
          <Text style={styles.subtitle}>
            已连续签到 <Text style={styles.streakNumber}>{streak}</Text> 天
          </Text>
        </View>
        <View style={styles.rewardBadge}>
          <Text style={styles.rewardText}>+{reward * multiplier} COS</Text>
          {multiplier > 1 && (
            <Text style={styles.multiplierText}>{multiplier}x 会员加成</Text>
          )}
        </View>
      </View>

      <View style={styles.weekRow}>
        {weekDays.map((day, index) => {
          const isPast = index < currentDayOfWeek;
          const isToday = index === currentDayOfWeek;
          const isChecked = isPast || (isToday && (isCheckedToday || checked));

          return (
            <View
              key={day}
              style={[
                styles.dayCircle,
                isChecked && styles.dayChecked,
                isToday && styles.dayToday,
              ]}
            >
              {isChecked ? (
                <Text style={styles.checkMark}>✓</Text>
              ) : (
                <Text style={[styles.dayText, isToday && styles.dayTextToday]}>
                  {day}
                </Text>
              )}
            </View>
          );
        })}
      </View>

      <Pressable
        style={({ pressed }) => [
          styles.checkInButton,
          (isCheckedToday || checked) && styles.checkInButtonDisabled,
          pressed && !isCheckedToday && !checked && styles.checkInButtonPressed,
        ]}
        onPress={handleCheckIn}
        disabled={isCheckedToday || checked || loading}
      >
        {loading ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text style={styles.checkInButtonText}>
            {isCheckedToday || checked ? '今日已签到' : '立即签到'}
          </Text>
        )}
      </Pressable>

      <Text style={styles.hint}>
        连续签到7天可获得额外奖励，中断则重新计算
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 20,
  },
  title: {
    fontSize: 18,
    fontWeight: '700',
    color: '#1f2937',
  },
  subtitle: {
    fontSize: 14,
    color: '#6b7280',
    marginTop: 4,
  },
  streakNumber: {
    color: '#f59e0b',
    fontWeight: '700',
  },
  rewardBadge: {
    backgroundColor: '#fef3c7',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 8,
    alignItems: 'flex-end',
  },
  rewardText: {
    fontSize: 14,
    fontWeight: '700',
    color: '#f59e0b',
  },
  multiplierText: {
    fontSize: 11,
    color: '#d97706',
    marginTop: 2,
  },
  weekRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 20,
  },
  dayCircle: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#f3f4f6',
    justifyContent: 'center',
    alignItems: 'center',
  },
  dayChecked: {
    backgroundColor: '#6D28D9',
  },
  dayToday: {
    borderWidth: 2,
    borderColor: '#6D28D9',
  },
  dayText: {
    fontSize: 14,
    color: '#6b7280',
    fontWeight: '500',
  },
  dayTextToday: {
    color: '#6D28D9',
    fontWeight: '700',
  },
  checkMark: {
    fontSize: 18,
    color: '#fff',
    fontWeight: '700',
  },
  checkInButton: {
    backgroundColor: '#6D28D9',
    paddingVertical: 14,
    borderRadius: 12,
    alignItems: 'center',
    marginBottom: 12,
  },
  checkInButtonDisabled: {
    backgroundColor: '#d1d5db',
  },
  checkInButtonPressed: {
    backgroundColor: '#5b21b6',
  },
  checkInButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  hint: {
    fontSize: 12,
    color: '#9ca3af',
    textAlign: 'center',
  },
});
