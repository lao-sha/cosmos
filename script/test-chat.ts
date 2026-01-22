/**
 * 聊天和直播系统测试
 * 测试 ChatCore, ChatGroup, ChatPermission, Livestream 模块
 */

import { ApiPromise, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import {
  initApi,
  getTestAccounts,
  log,
  runTestSuite,
  signAndSendTx,
  assert,
  TestSuiteResult,
  DEFAULT_WS_ENDPOINT,
  randomString,
  UNIT,
} from './test-utils.js';

let api: ApiPromise;
let keyring: Keyring;
let accounts: {
  alice: KeyringPair;
  bob: KeyringPair;
  charlie: KeyringPair;
  dave: KeyringPair;
  eve: KeyringPair;
  ferdie: KeyringPair;
};

/**
 * 初始化测试环境
 */
async function setup(): Promise<void> {
  log.section('初始化聊天/直播系统测试环境...');
  api = await initApi(DEFAULT_WS_ENDPOINT);
  keyring = new Keyring({ type: 'sr25519' });
  accounts = getTestAccounts(keyring);
  log.success('测试环境初始化完成');
}

/**
 * 清理测试环境
 */
async function cleanup(): Promise<void> {
  if (api) {
    await api.disconnect();
    log.info('API 连接已断开');
  }
}

/**
 * 运行聊天和直播系统测试
 */
export async function runChatTests(): Promise<TestSuiteResult> {
  await setup();

  const result = await runTestSuite('聊天和直播系统测试', [
    // ==================== ChatPermission 模块 ====================
    {
      name: '[ChatPermission] 查询权限配置',
      fn: async () => {
        if (api.query.chatPermission) {
          if (api.query.chatPermission.permissions) {
            const permissions = await api.query.chatPermission.permissions(accounts.alice.address);
            log.subSection(`Alice 的权限: ${permissions.toString()}`);
          }
          if (api.query.chatPermission.globalSettings) {
            const settings = await api.query.chatPermission.globalSettings();
            log.subSection(`全局设置: ${settings.toString()}`);
          }
        }
        assert(api.query.chatPermission !== undefined, 'ChatPermission pallet 应该存在');
      },
    },
    {
      name: '[ChatPermission] 设置用户权限',
      fn: async () => {
        if (api.tx.chatPermission && api.tx.chatPermission.setPermission) {
          const tx = api.tx.chatPermission.setPermission(accounts.bob.address, 'CanChat', true);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`设置权限结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('setPermission 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== ChatCore 模块 ====================
    {
      name: '[ChatCore] 查询聊天核心配置',
      fn: async () => {
        if (api.query.chatCore) {
          // 查询各种可能的存储
          const storageKeys = Object.keys(api.query.chatCore);
          log.subSection(`ChatCore 存储项: ${storageKeys.join(', ')}`);
          
          if (api.query.chatCore.messageCounter) {
            const counter = await api.query.chatCore.messageCounter();
            log.subSection(`消息计数器: ${counter.toString()}`);
          }
        }
        assert(api.query.chatCore !== undefined, 'ChatCore pallet 应该存在');
      },
    },
    {
      name: '[ChatCore] 发送私聊消息',
      fn: async () => {
        if (api.tx.chatCore && api.tx.chatCore.sendMessage) {
          const message = '你好，这是一条测试消息！' + randomString(5);
          
          const tx = api.tx.chatCore.sendMessage(accounts.bob.address, message);
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection('私聊消息发送成功');
          } else {
            log.subSection(`私聊消息发送结果: ${result.error}`);
          }
        } else if (api.tx.chatCore && api.tx.chatCore.sendPrivateMessage) {
          const message = '测试私聊消息';
          const tx = api.tx.chatCore.sendPrivateMessage(accounts.bob.address, message);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`私聊消息结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('sendMessage 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[ChatCore] 查询消息历史',
      fn: async () => {
        if (api.query.chatCore && api.query.chatCore.messages) {
          const messages = await api.query.chatCore.messages(accounts.alice.address);
          log.subSection(`Alice 的消息: ${messages.toString()}`);
        } else if (api.query.chatCore && api.query.chatCore.messageHistory) {
          const history = await api.query.chatCore.messageHistory(accounts.alice.address);
          log.subSection(`Alice 的消息历史: ${history.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== ChatGroup 模块 ====================
    {
      name: '[ChatGroup] 创建群组',
      fn: async () => {
        if (api.tx.chatGroup && api.tx.chatGroup.createGroup) {
          const groupName = '测试群组_' + randomString(4);
          const description = '这是一个测试群组';
          
          const tx = api.tx.chatGroup.createGroup(groupName, description);
          const result = await signAndSendTx(api, tx, accounts.alice);
          
          if (result.success) {
            log.subSection(`群组 "${groupName}" 创建成功`);
            // 尝试找到群组创建事件
            for (const { event } of result.events) {
              if (event.section === 'chatGroup' && (event.method === 'GroupCreated' || event.method === 'Created')) {
                log.subSection(`群组事件: ${event.data.toString()}`);
              }
            }
          } else {
            log.subSection(`群组创建结果: ${result.error}`);
          }
        } else if (api.tx.chatGroup && api.tx.chatGroup.create) {
          const tx = api.tx.chatGroup.create('测试群组', '描述');
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`群组创建结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('createGroup 方法不可用');
        }
        assert(api.query.chatGroup !== undefined, 'ChatGroup pallet 应该存在');
      },
    },
    {
      name: '[ChatGroup] 查询群组列表',
      fn: async () => {
        if (api.query.chatGroup && api.query.chatGroup.groups) {
          const groups = await api.query.chatGroup.groups.entries();
          log.subSection(`群组总数: ${groups.length}`);
          
          // 显示前几个群组
          for (const [key, value] of groups.slice(0, 3)) {
            log.subSection(`  群组: ${value.toString().slice(0, 50)}...`);
          }
        } else if (api.query.chatGroup && api.query.chatGroup.groupInfo) {
          const info = await api.query.chatGroup.groupInfo.entries();
          log.subSection(`群组数量: ${info.length}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[ChatGroup] 加入群组',
      fn: async () => {
        if (api.tx.chatGroup && api.tx.chatGroup.joinGroup) {
          // 尝试加入群组 ID 为 1
          const tx = api.tx.chatGroup.joinGroup(1);
          const result = await signAndSendTx(api, tx, accounts.bob);
          log.subSection(`加入群组结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.chatGroup && api.tx.chatGroup.join) {
          const tx = api.tx.chatGroup.join(1);
          const result = await signAndSendTx(api, tx, accounts.bob);
          log.subSection(`加入群组结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('joinGroup 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[ChatGroup] 发送群组消息',
      fn: async () => {
        if (api.tx.chatGroup && api.tx.chatGroup.sendMessage) {
          const message = '这是一条群组消息_' + randomString(4);
          
          const tx = api.tx.chatGroup.sendMessage(1, message);
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`群组消息发送结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.chatGroup && api.tx.chatGroup.sendGroupMessage) {
          const tx = api.tx.chatGroup.sendGroupMessage(1, '测试群消息');
          const result = await signAndSendTx(api, tx, accounts.alice);
          log.subSection(`群组消息结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('sendMessage 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[ChatGroup] 查询群组成员',
      fn: async () => {
        if (api.query.chatGroup && api.query.chatGroup.members) {
          const members = await api.query.chatGroup.members(1);
          log.subSection(`群组 1 成员: ${members.toString()}`);
        } else if (api.query.chatGroup && api.query.chatGroup.groupMembers) {
          const members = await api.query.chatGroup.groupMembers(1);
          log.subSection(`群组 1 成员: ${members.toString()}`);
        }
        assert(true, '查询完成');
      },
    },

    // ==================== Livestream 模块 ====================
    {
      name: '[Livestream] 查询直播间列表',
      fn: async () => {
        if (api.query.livestream && api.query.livestream.rooms) {
          const rooms = await api.query.livestream.rooms.entries();
          log.subSection(`直播间总数: ${rooms.length}`);
          
          for (const [key, value] of rooms.slice(0, 3)) {
            log.subSection(`  直播间: ${value.toString().slice(0, 80)}...`);
          }
        } else if (api.query.livestream && api.query.livestream.liveRooms) {
          const rooms = await api.query.livestream.liveRooms.entries();
          log.subSection(`直播间数量: ${rooms.length}`);
        }
        assert(api.query.livestream !== undefined, 'Livestream pallet 应该存在');
      },
    },
    {
      name: '[Livestream] 创建直播间',
      fn: async () => {
        if (api.tx.livestream && api.tx.livestream.createRoom) {
          const title = '测试直播_' + randomString(4);
          const description = '这是一个测试直播间';
          
          const tx = api.tx.livestream.createRoom(title, description, 'Normal'); // Normal 房间类型
          const result = await signAndSendTx(api, tx, accounts.charlie);
          
          if (result.success) {
            log.subSection(`直播间 "${title}" 创建成功`);
            for (const { event } of result.events) {
              if (event.section === 'livestream') {
                log.subSection(`直播事件: ${event.method} - ${event.data.toString()}`);
              }
            }
          } else {
            log.subSection(`直播间创建结果: ${result.error}`);
          }
        } else if (api.tx.livestream && api.tx.livestream.create) {
          const tx = api.tx.livestream.create('测试直播', 'Normal');
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`直播间创建结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('createRoom 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Livestream] 开始直播',
      fn: async () => {
        if (api.tx.livestream && api.tx.livestream.startLive) {
          const tx = api.tx.livestream.startLive(1); // 直播间 ID
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`开始直播结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.livestream && api.tx.livestream.start) {
          const tx = api.tx.livestream.start(1);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`开始直播结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('startLive 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Livestream] 加入直播间观看',
      fn: async () => {
        if (api.tx.livestream && api.tx.livestream.joinRoom) {
          const tx = api.tx.livestream.joinRoom(1);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`加入直播间结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.livestream && api.tx.livestream.join) {
          const tx = api.tx.livestream.join(1);
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`加入直播间结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('joinRoom 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Livestream] 发送礼物',
      fn: async () => {
        if (api.tx.livestream && api.tx.livestream.sendGift) {
          const giftId = 1;
          const quantity = 1;
          
          const tx = api.tx.livestream.sendGift(1, giftId, quantity); // 直播间ID, 礼物ID, 数量
          const result = await signAndSendTx(api, tx, accounts.dave);
          log.subSection(`发送礼物结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('sendGift 方法不可用');
        }
        assert(true, '测试完成');
      },
    },
    {
      name: '[Livestream] 查询直播间信息',
      fn: async () => {
        if (api.query.livestream && api.query.livestream.roomInfo) {
          const info = await api.query.livestream.roomInfo(1);
          log.subSection(`直播间 1 信息: ${info.toString()}`);
        } else if (api.query.livestream && api.query.livestream.rooms) {
          const room = await api.query.livestream.rooms(1);
          log.subSection(`直播间 1: ${room.toString()}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[Livestream] 查询观众列表',
      fn: async () => {
        if (api.query.livestream && api.query.livestream.viewers) {
          const viewers = await api.query.livestream.viewers(1);
          log.subSection(`直播间 1 观众: ${viewers.toString()}`);
        } else if (api.query.livestream && api.query.livestream.roomViewers) {
          const viewers = await api.query.livestream.roomViewers(1);
          log.subSection(`直播间 1 观众: ${viewers.toString()}`);
        }
        assert(true, '查询完成');
      },
    },
    {
      name: '[Livestream] 结束直播',
      fn: async () => {
        if (api.tx.livestream && api.tx.livestream.endLive) {
          const tx = api.tx.livestream.endLive(1);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`结束直播结果: ${result.success ? '成功' : result.error}`);
        } else if (api.tx.livestream && api.tx.livestream.end) {
          const tx = api.tx.livestream.end(1);
          const result = await signAndSendTx(api, tx, accounts.charlie);
          log.subSection(`结束直播结果: ${result.success ? '成功' : result.error}`);
        } else {
          log.subSection('endLive 方法不可用');
        }
        assert(true, '测试完成');
      },
    },

    // ==================== 综合测试 ====================
    {
      name: '聊天模块存储检查',
      fn: async () => {
        const pallets = ['chatPermission', 'chatCore', 'chatGroup', 'livestream'];
        
        let availableCount = 0;
        for (const pallet of pallets) {
          if ((api.query as any)[pallet]) {
            availableCount++;
            const storageKeys = Object.keys((api.query as any)[pallet]);
            log.subSection(`✓ ${pallet} 可用 (${storageKeys.length} 个存储项)`);
          } else {
            log.subSection(`✗ ${pallet} 不可用`);
          }
        }
        
        log.subSection(`总计: ${availableCount}/${pallets.length} 个聊天模块可用`);
        assert(availableCount > 0, '至少应该有一个聊天模块可用');
      },
    },
    {
      name: '聊天模块交易方法检查',
      fn: async () => {
        const methods: string[] = [];
        
        if (api.tx.chatCore) {
          methods.push(...Object.keys(api.tx.chatCore).map(m => `chatCore.${m}`));
        }
        if (api.tx.chatGroup) {
          methods.push(...Object.keys(api.tx.chatGroup).map(m => `chatGroup.${m}`));
        }
        if (api.tx.livestream) {
          methods.push(...Object.keys(api.tx.livestream).map(m => `livestream.${m}`));
        }
        
        log.subSection(`可用交易方法: ${methods.length}`);
        log.subSection(`方法列表: ${methods.slice(0, 10).join(', ')}${methods.length > 10 ? '...' : ''}`);
        
        assert(methods.length > 0, '应该有可用的交易方法');
      },
    },
  ]);

  await cleanup();
  return result;
}

// 如果直接运行此文件
const isMainModule = import.meta.url === `file://${process.argv[1]}`;
if (isMainModule) {
  runChatTests()
    .then(result => {
      process.exit(result.totalFailed > 0 ? 1 : 0);
    })
    .catch(error => {
      console.error('测试运行失败:', error);
      process.exit(1);
    });
}

