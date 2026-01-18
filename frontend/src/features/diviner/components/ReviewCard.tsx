/**
 * 评价卡片组件
 */

import React from 'react';
import { View, Text, StyleSheet, Pressable } from 'react-native';
import { Review } from '../types';

const THEME_COLOR = '#B2955D';

interface ReviewCardProps {
  review: Review;
  onReply?: () => void;
  showReplyButton?: boolean;
}

export function ReviewCard({ review, onReply, showReplyButton = false }: ReviewCardProps) {
  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return `${date.getFullYear()}/${date.getMonth() + 1}/${date.getDate()}`;
  };

  const renderStars = (rating: number) => {
    return '★'.repeat(rating) + '☆'.repeat(5 - rating);
  };

  const avgRating = (
    (review.overallRating + review.accuracyRating + review.attitudeRating + review.responseRating) / 4
  ).toFixed(1);

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <View style={styles.userInfo}>
          <View style={styles.avatar}>
            <Text style={styles.avatarText}>
              {review.isAnonymous ? '匿' : review.customer.slice(0, 2)}
            </Text>
          </View>
          <View>
            <Text style={styles.userName}>
              {review.isAnonymous ? '匿名用户' : `${review.customer.slice(0, 6)}...`}
            </Text>
            <Text style={styles.time}>{formatTime(review.createdAt)}</Text>
          </View>
        </View>
        <View style={styles.ratingBadge}>
          <Text style={styles.ratingValue}>{avgRating}</Text>
          <Text style={styles.ratingLabel}>分</Text>
        </View>
      </View>

      <View style={styles.ratingsGrid}>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingItemLabel}>总体</Text>
          <Text style={styles.stars}>{renderStars(review.overallRating)}</Text>
        </View>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingItemLabel}>准确度</Text>
          <Text style={styles.stars}>{renderStars(review.accuracyRating)}</Text>
        </View>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingItemLabel}>态度</Text>
          <Text style={styles.stars}>{renderStars(review.attitudeRating)}</Text>
        </View>
        <View style={styles.ratingItem}>
          <Text style={styles.ratingItemLabel}>响应</Text>
          <Text style={styles.stars}>{renderStars(review.responseRating)}</Text>
        </View>
      </View>

      {review.contentCid && (
        <View style={styles.contentBox}>
          <Text style={styles.contentText}>评价内容加载中...</Text>
        </View>
      )}

      {review.replyCid && (
        <View style={styles.replyBox}>
          <Text style={styles.replyLabel}>占卜师回复：</Text>
          <Text style={styles.replyText}>回复内容加载中...</Text>
        </View>
      )}

      {showReplyButton && !review.replyCid && (
        <Pressable style={styles.replyBtn} onPress={onReply}>
          <Text style={styles.replyBtnText}>回复评价</Text>
        </Pressable>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#FFF',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  userInfo: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 10,
  },
  avatar: {
    width: 36,
    height: 36,
    borderRadius: 18,
    backgroundColor: '#F5F5F7',
    justifyContent: 'center',
    alignItems: 'center',
  },
  avatarText: {
    fontSize: 14,
    color: '#666',
  },
  userName: {
    fontSize: 14,
    fontWeight: '500',
    color: '#000',
  },
  time: {
    fontSize: 12,
    color: '#999',
    marginTop: 2,
  },
  ratingBadge: {
    flexDirection: 'row',
    alignItems: 'baseline',
    backgroundColor: `${THEME_COLOR}20`,
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 8,
  },
  ratingValue: {
    fontSize: 18,
    fontWeight: '600',
    color: THEME_COLOR,
  },
  ratingLabel: {
    fontSize: 12,
    color: THEME_COLOR,
    marginLeft: 2,
  },
  ratingsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 8,
    marginBottom: 12,
  },
  ratingItem: {
    width: '48%',
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  ratingItemLabel: {
    fontSize: 12,
    color: '#666',
  },
  stars: {
    fontSize: 12,
    color: '#FFB800',
  },
  contentBox: {
    backgroundColor: '#F5F5F7',
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
  },
  contentText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 20,
  },
  replyBox: {
    backgroundColor: '#FFF9F0',
    borderRadius: 8,
    padding: 12,
    borderLeftWidth: 3,
    borderLeftColor: THEME_COLOR,
  },
  replyLabel: {
    fontSize: 12,
    color: THEME_COLOR,
    fontWeight: '500',
    marginBottom: 4,
  },
  replyText: {
    fontSize: 14,
    color: '#333',
    lineHeight: 20,
  },
  replyBtn: {
    height: 36,
    borderWidth: 1,
    borderColor: THEME_COLOR,
    borderRadius: 4,
    justifyContent: 'center',
    alignItems: 'center',
    marginTop: 8,
  },
  replyBtnText: {
    fontSize: 14,
    color: THEME_COLOR,
  },
});
