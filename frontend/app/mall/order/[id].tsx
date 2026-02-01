import { useLocalSearchParams, useRouter } from 'expo-router';
import {
  Package,
  Truck,
  CheckCircle,
  Clock,
  AlertCircle,
  MapPin,
  Store,
  Star,
} from 'lucide-react-native';
import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
  TextInput,
  Modal,
} from 'react-native';

import { sharemallService } from '@/src/services/sharemall';
import { sharemallTxService } from '@/src/services/sharemall-tx';
import type { MallOrder, Shop, Product, OrderStatus } from '@/src/types/sharemall';

const STATUS_STEPS: OrderStatus[] = ['Created', 'Paid', 'Shipped', 'Completed'];

export default function OrderDetailScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();

  const [order, setOrder] = useState<MallOrder | null>(null);
  const [shop, setShop] = useState<Shop | null>(null);
  const [product, setProduct] = useState<Product | null>(null);
  const [loading, setLoading] = useState(false);
  const [reviewModalVisible, setReviewModalVisible] = useState(false);
  const [rating, setRating] = useState(5);
  const [reviewContent, setReviewContent] = useState('');

  const orderId = parseInt(id || '0', 10);

  const loadData = async () => {
    if (!orderId) return;
    try {
      const orderData = await sharemallService.getOrder(orderId);
      setOrder(orderData);

      if (orderData) {
        const [shopData, productData] = await Promise.all([
          sharemallService.getShop(orderData.shopId),
          sharemallService.getProduct(orderData.productId),
        ]);
        setShop(shopData);
        setProduct(productData);
      }
    } catch (error) {
      console.error('Failed to load order:', error);
    }
  };

  useEffect(() => {
    loadData();
  }, [orderId]);

  const formatPrice = (price: string) => {
    const num = BigInt(price);
    return (Number(num) / 1e10).toFixed(2);
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusIndex = (status: OrderStatus) => {
    return STATUS_STEPS.indexOf(status);
  };

  const handleConfirmReceipt = async () => {
    if (!order) return;

    Alert.alert('确认收货', '确定已收到商品吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确定',
        onPress: async () => {
          setLoading(true);
          try {
            await sharemallTxService.confirmReceipt(order.id);
            Alert.alert('成功', '已确认收货');
            loadData();
          } catch (error: any) {
            Alert.alert('错误', error.message || '操作失败');
          } finally {
            setLoading(false);
          }
        },
      },
    ]);
  };

  const handleRequestRefund = () => {
    if (!order) return;

    Alert.prompt(
      '申请退款',
      '请输入退款原因 (IPFS CID)',
      [
        { text: '取消', style: 'cancel' },
        {
          text: '提交',
          onPress: async (reasonCid) => {
            if (!reasonCid) return;
            setLoading(true);
            try {
              await sharemallTxService.requestRefund(order.id, reasonCid);
              Alert.alert('成功', '退款申请已提交');
              loadData();
            } catch (error: any) {
              Alert.alert('错误', error.message || '申请失败');
            } finally {
              setLoading(false);
            }
          },
        },
      ],
      'plain-text'
    );
  };

  const handleCancelOrder = async () => {
    if (!order) return;

    Alert.alert('取消订单', '确定要取消订单吗？', [
      { text: '取消', style: 'cancel' },
      {
        text: '确定',
        style: 'destructive',
        onPress: async () => {
          setLoading(true);
          try {
            await sharemallTxService.cancelOrder(order.id);
            Alert.alert('成功', '订单已取消');
            loadData();
          } catch (error: any) {
            Alert.alert('错误', error.message || '取消失败');
          } finally {
            setLoading(false);
          }
        },
      },
    ]);
  };

  const handleSubmitReview = async () => {
    if (!order) return;

    setLoading(true);
    try {
      await sharemallTxService.submitReview(
        order.id,
        rating,
        reviewContent.trim() || undefined
      );
      Alert.alert('成功', '评价已提交');
      setReviewModalVisible(false);
      loadData();
    } catch (error: any) {
      Alert.alert('错误', error.message || '评价失败');
    } finally {
      setLoading(false);
    }
  };

  if (!order) {
    return (
      <View style={styles.loadingContainer}>
        <Text>加载中...</Text>
      </View>
    );
  }

  const currentStep = getStatusIndex(order.status);

  return (
    <View style={styles.container}>
      <ScrollView style={styles.scrollView}>
        {/* 订单状态 */}
        <View style={styles.statusCard}>
          <View style={styles.statusHeader}>
            {order.status === 'Completed' ? (
              <CheckCircle size={32} color="#388E3C" />
            ) : order.status === 'Shipped' ? (
              <Truck size={32} color="#7B1FA2" />
            ) : order.status === 'Disputed' ? (
              <AlertCircle size={32} color="#D32F2F" />
            ) : (
              <Clock size={32} color="#F57C00" />
            )}
            <Text style={styles.statusTitle}>
              {order.status === 'Created'
                ? '待支付'
                : order.status === 'Paid'
                ? '待发货'
                : order.status === 'Shipped'
                ? '已发货'
                : order.status === 'Completed'
                ? '已完成'
                : order.status === 'Disputed'
                ? '争议中'
                : order.status}
            </Text>
          </View>

          {/* 进度条 */}
          {!['Cancelled', 'Refunded', 'Disputed', 'Expired'].includes(order.status) && (
            <View style={styles.progressBar}>
              {STATUS_STEPS.map((step, index) => (
                <React.Fragment key={step}>
                  <View
                    style={[
                      styles.progressDot,
                      index <= currentStep && styles.progressDotActive,
                    ]}
                  />
                  {index < STATUS_STEPS.length - 1 && (
                    <View
                      style={[
                        styles.progressLine,
                        index < currentStep && styles.progressLineActive,
                      ]}
                    />
                  )}
                </React.Fragment>
              ))}
            </View>
          )}
        </View>

        {/* 物流信息 */}
        {order.trackingCid && (
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <Truck size={18} color="#666" />
              <Text style={styles.sectionTitle}>物流信息</Text>
            </View>
            <Text style={styles.trackingCid}>物流 CID: {order.trackingCid}</Text>
          </View>
        )}

        {/* 收货地址 */}
        {order.shippingCid && (
          <View style={styles.section}>
            <View style={styles.sectionHeader}>
              <MapPin size={18} color="#666" />
              <Text style={styles.sectionTitle}>收货地址</Text>
            </View>
            <Text style={styles.shippingCid}>地址 CID: {order.shippingCid}</Text>
          </View>
        )}

        {/* 商品信息 */}
        <View style={styles.section}>
          <TouchableOpacity
            style={styles.shopRow}
            onPress={() => shop && router.push(`/mall/shop/${shop.id}` as any)}
          >
            <Store size={16} color="#666" />
            <Text style={styles.shopName}>{shop?.name || '店铺'}</Text>
          </TouchableOpacity>

          <View style={styles.productCard}>
            <View style={styles.productImage}>
              <Package size={32} color="#ccc" />
            </View>
            <View style={styles.productInfo}>
              <Text style={styles.productName}>商品 #{order.productId}</Text>
              <Text style={styles.productPrice}>¥{formatPrice(order.unitPrice)}</Text>
            </View>
            <Text style={styles.quantity}>x{order.quantity}</Text>
          </View>
        </View>

        {/* 订单金额 */}
        <View style={styles.section}>
          <View style={styles.amountRow}>
            <Text style={styles.amountLabel}>商品金额</Text>
            <Text style={styles.amountValue}>¥{formatPrice(order.totalAmount)}</Text>
          </View>
          <View style={styles.amountRow}>
            <Text style={styles.amountLabel}>平台手续费</Text>
            <Text style={styles.amountValue}>¥{formatPrice(order.platformFee)}</Text>
          </View>
          <View style={[styles.amountRow, styles.totalRow]}>
            <Text style={styles.totalLabel}>实付金额</Text>
            <Text style={styles.totalValue}>¥{formatPrice(order.totalAmount)}</Text>
          </View>
        </View>

        {/* 订单信息 */}
        <View style={styles.section}>
          <Text style={styles.sectionTitle}>订单信息</Text>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>订单编号</Text>
            <Text style={styles.infoValue}>{order.id}</Text>
          </View>
          <View style={styles.infoRow}>
            <Text style={styles.infoLabel}>创建时间</Text>
            <Text style={styles.infoValue}>{formatTime(order.createdAt)}</Text>
          </View>
          {order.paidAt && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>支付时间</Text>
              <Text style={styles.infoValue}>{formatTime(order.paidAt)}</Text>
            </View>
          )}
          {order.shippedAt && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>发货时间</Text>
              <Text style={styles.infoValue}>{formatTime(order.shippedAt)}</Text>
            </View>
          )}
          {order.completedAt && (
            <View style={styles.infoRow}>
              <Text style={styles.infoLabel}>完成时间</Text>
              <Text style={styles.infoValue}>{formatTime(order.completedAt)}</Text>
            </View>
          )}
        </View>
      </ScrollView>

      {/* 底部操作 */}
      <View style={styles.bottomBar}>
        {order.status === 'Created' && (
          <TouchableOpacity
            style={styles.cancelBtn}
            onPress={handleCancelOrder}
            disabled={loading}
          >
            <Text style={styles.cancelBtnText}>取消订单</Text>
          </TouchableOpacity>
        )}
        {order.status === 'Shipped' && (
          <>
            <TouchableOpacity
              style={styles.refundBtn}
              onPress={handleRequestRefund}
              disabled={loading}
            >
              <Text style={styles.refundBtnText}>申请退款</Text>
            </TouchableOpacity>
            <TouchableOpacity
              style={styles.confirmBtn}
              onPress={handleConfirmReceipt}
              disabled={loading}
            >
              <Text style={styles.confirmBtnText}>确认收货</Text>
            </TouchableOpacity>
          </>
        )}
        {order.status === 'Completed' && (
          <TouchableOpacity
            style={styles.reviewBtn}
            onPress={() => setReviewModalVisible(true)}
          >
            <Text style={styles.reviewBtnText}>评价订单</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* 评价弹窗 */}
      <Modal visible={reviewModalVisible} transparent animationType="fade">
        <View style={styles.modalOverlay}>
          <View style={styles.modalContent}>
            <Text style={styles.modalTitle}>评价订单</Text>

            <Text style={styles.modalLabel}>评分</Text>
            <View style={styles.ratingRow}>
              {[1, 2, 3, 4, 5].map((star) => (
                <TouchableOpacity key={star} onPress={() => setRating(star)}>
                  <Star
                    size={32}
                    color={star <= rating ? '#FFB300' : '#ddd'}
                    fill={star <= rating ? '#FFB300' : 'transparent'}
                  />
                </TouchableOpacity>
              ))}
            </View>

            <Text style={styles.modalLabel}>评价内容 (IPFS CID, 可选)</Text>
            <TextInput
              style={styles.modalInput}
              placeholder="输入评价内容的 IPFS CID"
              value={reviewContent}
              onChangeText={setReviewContent}
            />

            <View style={styles.modalActions}>
              <TouchableOpacity
                style={styles.modalCancelBtn}
                onPress={() => setReviewModalVisible(false)}
              >
                <Text style={styles.modalCancelText}>取消</Text>
              </TouchableOpacity>
              <TouchableOpacity
                style={styles.modalConfirmBtn}
                onPress={handleSubmitReview}
                disabled={loading}
              >
                <Text style={styles.modalConfirmText}>
                  {loading ? '提交中...' : '提交评价'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  loadingContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  scrollView: {
    flex: 1,
  },
  statusCard: {
    backgroundColor: '#fff',
    margin: 12,
    borderRadius: 12,
    padding: 20,
  },
  statusHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  statusTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginLeft: 12,
  },
  progressBar: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  progressDot: {
    width: 12,
    height: 12,
    borderRadius: 6,
    backgroundColor: '#ddd',
  },
  progressDotActive: {
    backgroundColor: '#1976D2',
  },
  progressLine: {
    flex: 1,
    height: 2,
    backgroundColor: '#ddd',
  },
  progressLineActive: {
    backgroundColor: '#1976D2',
  },
  section: {
    backgroundColor: '#fff',
    marginHorizontal: 12,
    marginBottom: 12,
    borderRadius: 12,
    padding: 16,
  },
  sectionHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  sectionTitle: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
    marginLeft: 6,
  },
  trackingCid: {
    fontSize: 13,
    color: '#666',
  },
  shippingCid: {
    fontSize: 13,
    color: '#666',
  },
  shopRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#f0f0f0',
  },
  shopName: {
    fontSize: 14,
    color: '#333',
    marginLeft: 8,
  },
  productCard: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  productImage: {
    width: 64,
    height: 64,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
    alignItems: 'center',
    justifyContent: 'center',
  },
  productInfo: {
    flex: 1,
    marginLeft: 12,
  },
  productName: {
    fontSize: 14,
    color: '#333',
    marginBottom: 4,
  },
  productPrice: {
    fontSize: 14,
    color: '#E53935',
  },
  quantity: {
    fontSize: 14,
    color: '#666',
  },
  amountRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
  },
  amountLabel: {
    fontSize: 13,
    color: '#666',
  },
  amountValue: {
    fontSize: 13,
    color: '#333',
  },
  totalRow: {
    borderTopWidth: 1,
    borderTopColor: '#f0f0f0',
    marginTop: 8,
    paddingTop: 12,
  },
  totalLabel: {
    fontSize: 14,
    fontWeight: '500',
    color: '#333',
  },
  totalValue: {
    fontSize: 16,
    fontWeight: '600',
    color: '#E53935',
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 6,
  },
  infoLabel: {
    fontSize: 13,
    color: '#999',
  },
  infoValue: {
    fontSize: 13,
    color: '#333',
  },
  bottomBar: {
    flexDirection: 'row',
    backgroundColor: '#fff',
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderTopWidth: 1,
    borderTopColor: '#eee',
  },
  cancelBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderRadius: 20,
    borderWidth: 1,
    borderColor: '#ddd',
  },
  cancelBtnText: {
    color: '#666',
    fontSize: 14,
  },
  refundBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderRadius: 20,
    borderWidth: 1,
    borderColor: '#F57C00',
    marginRight: 12,
  },
  refundBtnText: {
    color: '#F57C00',
    fontSize: 14,
  },
  confirmBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderRadius: 20,
    backgroundColor: '#1976D2',
  },
  confirmBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  reviewBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    borderRadius: 20,
    backgroundColor: '#FFB300',
  },
  reviewBtnText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    backgroundColor: '#fff',
    borderRadius: 12,
    padding: 20,
    width: '85%',
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '600',
    color: '#333',
    marginBottom: 20,
    textAlign: 'center',
  },
  modalLabel: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
  },
  ratingRow: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginBottom: 20,
  },
  modalInput: {
    backgroundColor: '#f5f5f5',
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 12,
    fontSize: 14,
    marginBottom: 20,
  },
  modalActions: {
    flexDirection: 'row',
  },
  modalCancelBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    marginRight: 8,
    borderRadius: 8,
    backgroundColor: '#f5f5f5',
  },
  modalCancelText: {
    color: '#666',
    fontSize: 14,
  },
  modalConfirmBtn: {
    flex: 1,
    paddingVertical: 12,
    alignItems: 'center',
    marginLeft: 8,
    borderRadius: 8,
    backgroundColor: '#FFB300',
  },
  modalConfirmText: {
    color: '#fff',
    fontSize: 14,
    fontWeight: '500',
  },
});
