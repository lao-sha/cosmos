//! # 商城模块单元测试

use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};

// ============================================================================
// 店铺管理测试
// ============================================================================

#[test]
fn create_shop_works() {
    new_test_ext().execute_with(|| {
        // 创建店铺
        assert_ok!(Mall::create_shop(
            RuntimeOrigin::signed(1),
            b"Test Shop".to_vec(),
            Some(b"QmLogo123".to_vec()),
            Some(b"QmDesc456".to_vec()),
        ));

        // 验证店铺创建
        let shop = Mall::shops(0).unwrap();
        assert_eq!(shop.owner, 1);
        assert_eq!(shop.name.to_vec(), b"Test Shop".to_vec());
        assert_eq!(shop.status, ShopStatus::Pending);
        assert_eq!(shop.deposit, 100);

        // 验证用户店铺索引
        assert_eq!(Mall::user_shop(1), Some(0));

        // 验证保证金被锁定
        assert_eq!(Balances::reserved_balance(1), 100);
    });
}

#[test]
fn create_shop_fails_if_already_has_shop() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mall::create_shop(
            RuntimeOrigin::signed(1),
            b"Shop 1".to_vec(),
            None,
            None,
        ));

        assert_noop!(
            Mall::create_shop(
                RuntimeOrigin::signed(1),
                b"Shop 2".to_vec(),
                None,
                None,
            ),
            Error::<Test>::ShopAlreadyExists
        );
    });
}

#[test]
fn approve_shop_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mall::create_shop(
            RuntimeOrigin::signed(1),
            b"Test Shop".to_vec(),
            None,
            None,
        ));

        // 治理审核通过
        assert_ok!(Mall::approve_shop(RuntimeOrigin::root(), 0));

        let shop = Mall::shops(0).unwrap();
        assert_eq!(shop.status, ShopStatus::Active);
    });
}

#[test]
fn close_shop_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Mall::create_shop(
            RuntimeOrigin::signed(1),
            b"Test Shop".to_vec(),
            None,
            None,
        ));
        assert_ok!(Mall::approve_shop(RuntimeOrigin::root(), 0));

        let balance_before = Balances::free_balance(1);

        // 关闭店铺
        assert_ok!(Mall::close_shop(RuntimeOrigin::signed(1), 0));

        let shop = Mall::shops(0).unwrap();
        assert_eq!(shop.status, ShopStatus::Closed);

        // 验证保证金退还
        assert_eq!(Balances::free_balance(1), balance_before + 100);
        assert_eq!(Balances::reserved_balance(1), 0);
    });
}

// ============================================================================
// 商品管理测试
// ============================================================================

#[test]
fn create_product_works() {
    new_test_ext().execute_with(|| {
        // 创建并激活店铺
        assert_ok!(Mall::create_shop(
            RuntimeOrigin::signed(1),
            b"Test Shop".to_vec(),
            None,
            None,
        ));
        assert_ok!(Mall::approve_shop(RuntimeOrigin::root(), 0));

        // 创建商品
        assert_ok!(Mall::create_product(
            RuntimeOrigin::signed(1),
            0,
            b"QmProductName".to_vec(),
            b"QmProductImages".to_vec(),
            b"QmProductDetail".to_vec(),
            1000,
            50,
            ProductCategory::Physical,
        ));

        let product = Mall::products(0).unwrap();
        assert_eq!(product.shop_id, 0);
        assert_eq!(product.price, 1000);
        assert_eq!(product.stock, 50);
        assert_eq!(product.status, ProductStatus::Draft);
    });
}

#[test]
fn publish_product_works() {
    new_test_ext().execute_with(|| {
        // 设置店铺和商品
        assert_ok!(Mall::create_shop(RuntimeOrigin::signed(1), b"Shop".to_vec(), None, None));
        assert_ok!(Mall::approve_shop(RuntimeOrigin::root(), 0));
        assert_ok!(Mall::create_product(
            RuntimeOrigin::signed(1),
            0,
            b"QmName".to_vec(),
            b"QmImages".to_vec(),
            b"QmDetail".to_vec(),
            1000,
            50,
            ProductCategory::Physical,
        ));

        // 上架商品
        assert_ok!(Mall::publish_product(RuntimeOrigin::signed(1), 0));

        let product = Mall::products(0).unwrap();
        assert_eq!(product.status, ProductStatus::OnSale);
    });
}

// ============================================================================
// 订单流程测试
// ============================================================================

fn setup_shop_and_product() {
    // 创建店铺
    assert_ok!(Mall::create_shop(RuntimeOrigin::signed(1), b"Shop".to_vec(), None, None));
    assert_ok!(Mall::approve_shop(RuntimeOrigin::root(), 0));

    // 创建并上架商品
    assert_ok!(Mall::create_product(
        RuntimeOrigin::signed(1),
        0,
        b"QmName".to_vec(),
        b"QmImages".to_vec(),
        b"QmDetail".to_vec(),
        1000,
        50,
        ProductCategory::Physical,
    ));
    assert_ok!(Mall::publish_product(RuntimeOrigin::signed(1), 0));
}

#[test]
fn place_order_works() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();

        let buyer_balance_before = Balances::free_balance(3);

        // 下单
        assert_ok!(Mall::place_order(
            RuntimeOrigin::signed(3),
            0,
            2,
            Some(b"QmShippingAddr".to_vec()),
        ));

        // 验证订单创建
        let order = Mall::orders(0).unwrap();
        assert_eq!(order.buyer, 3);
        assert_eq!(order.seller, 1);
        assert_eq!(order.quantity, 2);
        assert_eq!(order.total_amount, 2000);
        assert_eq!(order.platform_fee, 40); // 2%
        assert_eq!(order.status, MallOrderStatus::Paid);

        // 验证资金被锁定
        assert_eq!(Balances::free_balance(3), buyer_balance_before - 2000);

        // 验证库存减少
        let product = Mall::products(0).unwrap();
        assert_eq!(product.stock, 48);
        assert_eq!(product.sold_count, 2);
    });
}

#[test]
fn place_order_fails_for_own_product() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();

        // 店主不能购买自己的商品
        assert_noop!(
            Mall::place_order(RuntimeOrigin::signed(1), 0, 1, None),
            Error::<Test>::CannotBuyOwnProduct
        );
    });
}

#[test]
fn ship_order_works() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();
        assert_ok!(Mall::place_order(RuntimeOrigin::signed(3), 0, 1, None));

        // 发货
        assert_ok!(Mall::ship_order(
            RuntimeOrigin::signed(1),
            0,
            b"QmTrackingInfo".to_vec(),
        ));

        let order = Mall::orders(0).unwrap();
        assert_eq!(order.status, MallOrderStatus::Shipped);
        assert!(order.shipped_at.is_some());
    });
}

#[test]
fn confirm_receipt_works() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();
        assert_ok!(Mall::place_order(RuntimeOrigin::signed(3), 0, 1, None));
        assert_ok!(Mall::ship_order(RuntimeOrigin::signed(1), 0, b"QmTracking".to_vec()));

        let seller_balance_before = Balances::free_balance(1);
        let platform_balance_before = Balances::free_balance(100);

        // 确认收货
        assert_ok!(Mall::confirm_receipt(RuntimeOrigin::signed(3), 0));

        let order = Mall::orders(0).unwrap();
        assert_eq!(order.status, MallOrderStatus::Completed);

        // 验证资金分配
        // 总金额 1000，平台费 20 (2%)，卖家收入 980
        assert_eq!(Balances::free_balance(1), seller_balance_before + 980);
        assert_eq!(Balances::free_balance(100), platform_balance_before + 20);
    });
}

#[test]
fn cancel_order_works() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();
        assert_ok!(Mall::place_order(RuntimeOrigin::signed(3), 0, 2, None));

        let buyer_balance_before = Balances::free_balance(3);

        // 取消订单
        assert_ok!(Mall::cancel_order(RuntimeOrigin::signed(3), 0));

        let order = Mall::orders(0).unwrap();
        assert_eq!(order.status, MallOrderStatus::Cancelled);

        // 验证退款
        assert_eq!(Balances::free_balance(3), buyer_balance_before + 2000);

        // 验证库存恢复
        let product = Mall::products(0).unwrap();
        assert_eq!(product.stock, 50);
        assert_eq!(product.sold_count, 0);
    });
}

#[test]
fn refund_flow_works() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();
        assert_ok!(Mall::place_order(RuntimeOrigin::signed(3), 0, 1, None));
        assert_ok!(Mall::ship_order(RuntimeOrigin::signed(1), 0, b"QmTracking".to_vec()));

        // 买家申请退款
        assert_ok!(Mall::request_refund(
            RuntimeOrigin::signed(3),
            0,
            b"QmReason".to_vec(),
        ));

        let order = Mall::orders(0).unwrap();
        assert_eq!(order.status, MallOrderStatus::Disputed);

        let buyer_balance_before = Balances::free_balance(3);

        // 卖家同意退款
        assert_ok!(Mall::approve_refund(RuntimeOrigin::signed(1), 0));

        let order = Mall::orders(0).unwrap();
        assert_eq!(order.status, MallOrderStatus::Refunded);

        // 验证退款
        assert_eq!(Balances::free_balance(3), buyer_balance_before + 1000);
    });
}

// ============================================================================
// 评价测试
// ============================================================================

#[test]
fn submit_review_works() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();
        assert_ok!(Mall::place_order(RuntimeOrigin::signed(3), 0, 1, None));
        assert_ok!(Mall::ship_order(RuntimeOrigin::signed(1), 0, b"QmTracking".to_vec()));
        assert_ok!(Mall::confirm_receipt(RuntimeOrigin::signed(3), 0));

        // 提交评价
        assert_ok!(Mall::submit_review(
            RuntimeOrigin::signed(3),
            0,
            5,
            Some(b"QmReviewContent".to_vec()),
        ));

        let review = Mall::reviews(0).unwrap();
        assert_eq!(review.rating, 5);

        // 验证店铺评分更新
        let shop = Mall::shops(0).unwrap();
        assert_eq!(shop.rating, 500); // 5.0 * 100
        assert_eq!(shop.rating_count, 1);
    });
}

#[test]
fn submit_review_fails_if_already_reviewed() {
    new_test_ext().execute_with(|| {
        setup_shop_and_product();
        assert_ok!(Mall::place_order(RuntimeOrigin::signed(3), 0, 1, None));
        assert_ok!(Mall::ship_order(RuntimeOrigin::signed(1), 0, b"QmTracking".to_vec()));
        assert_ok!(Mall::confirm_receipt(RuntimeOrigin::signed(3), 0));
        assert_ok!(Mall::submit_review(RuntimeOrigin::signed(3), 0, 5, None));

        // 重复评价失败
        assert_noop!(
            Mall::submit_review(RuntimeOrigin::signed(3), 0, 4, None),
            Error::<Test>::AlreadyReviewed
        );
    });
}
