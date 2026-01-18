// frontend/src/divination/market/components/ReviewCard.tsx

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Review } from '../types/market.types';
import { THEME } from '../theme';
import { RatingStars } from './RatingStars';
import { Avatar } from './Avatar';
import { formatTimeAgo, truncateAddress } from '../utils/market.utils';

interface ReviewCardProps {
  review: Review;
  showReply?: boolean;
}

export const ReviewCard: React.FC<ReviewCardProps> = ({
  review,
  showReply = true,
}) => {
  const averageRating =
    (review.overallRating +
      review.accuracyRating +
      review.attitudeRating +
      review.responseRating) /
    4 /
    100;

  const displayName = review.isAnonymous
    ? '匿名用户'
    : review.customerName || truncateAddress(review.customer);

  return (
    <View style={styles.card}>
      {/* 头部 */}
      <View style={styles.header}>
        <Avatar
          name={review.isAnonymous ? '匿' : displayName}
          size={36}
        />
        <View style={styles.headerInfo}>
          <Text style={styles.name}>{displayName}</Text>
          <View style={styles.ratingRow}>
            <RatingStars rating={averageRating} size={12} showValue={false} />
            <Text style={styles.time}>{formatTimeAgo(review.createdAt)}</Text>
          </View>
        </View>
      </View>

      {/* 评价内容 */}
      {review.content && (
        <Text style={styles.content}>{review.content}</Text>
      )}

      {/* 详细评分 */}
      <View style={styles.detailedRatings}>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingLabel}>准确性</Text>
          <Text style={styles.ratingValue}>{(review.accuracyRating / 100).toFixed(1)}</Text>
        </View>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingLabel}>态度</Text>
          <Text style={styles.ratingValue}>{(review.attitudeRating / 100).toFixed(1)}</Text>
        </View>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingLabel}>响应</Text>
          <Text style={styles.ratingValue}>{(review.responseRating / 100).toFixed(1)}</Text>
        </View>
      </View>

      {/* 回复 */}
      {showReply && review.reply && (
        <View style={styles.replyContainer}>
          <Text style={styles.replyLabel}>解卦师回复：</Text>
          <Text style={styles.replyContent}>{review.reply}</Text>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: THEME.card,
    borderRadius: 10,
    padding: 14,
    marginBottom: 10,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'flex-start',
  },
  headerInfo: {
    flex: 1,
    marginLeft: 10,
  },
  name: {
    fontSize: 14,
    fontWeight: '500',
    color: THEME.text,
  },
  ratingRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginTop: 4,
    gap: 8,
  },
  time: {
    fontSize: 11,
    color: THEME.textTertiary,
  },
  content: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 20,
    marginTop: 10,
  },
  detailedRatings: {
    flexDirection: 'row',
    marginTop: 12,
    gap: 20,
  },
  ratingItem: {
    alignItems: 'center',
  },
  ratingLabel: {
    fontSize: 11,
    color: THEME.textTertiary,
  },
  ratingValue: {
    fontSize: 14,
    fontWeight: '600',
    color: THEME.primary,
    marginTop: 2,
  },
  replyContainer: {
    backgroundColor: THEME.background,
    borderRadius: 6,
    padding: 10,
    marginTop: 12,
  },
  replyLabel: {
    fontSize: 11,
    color: THEME.textTertiary,
    marginBottom: 4,
  },
  replyContent: {
    fontSize: 13,
    color: THEME.textSecondary,
    lineHeight: 18,
  },
});
