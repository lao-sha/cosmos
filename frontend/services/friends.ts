import { getApi } from './api';

export type FriendStatus = 'pending' | 'accepted' | 'blocked';

export interface Friend {
  id: string;
  address: string;
  name: string;
  avatar?: string;
  remark?: string;
  status: FriendStatus;
  isOnline: boolean;
  lastSeen?: number;
  addedAt: number;
}

export interface FriendRequest {
  id: string;
  fromAddress: string;
  fromName: string;
  fromAvatar?: string;
  message?: string;
  createdAt: number;
}

export async function getFriends(address: string): Promise<Friend[]> {
  try {
    const api = await getApi();
    const data = await api.query.chatCore.contacts(address);
    
    if (data.isEmpty) return [];
    
    const friends = data.toJSON() as any[];
    return friends
      .filter((f: any) => f.status === 'accepted')
      .map((f: any) => ({
        id: f.id,
        address: f.address,
        name: f.name || '用户',
        avatar: f.avatar,
        remark: f.remark,
        status: f.status as FriendStatus,
        isOnline: f.isOnline || false,
        lastSeen: f.lastSeen,
        addedAt: f.addedAt || Date.now(),
      }));
  } catch (error) {
    console.error('Failed to get friends:', error);
    return getMockFriends();
  }
}

export async function getFriendRequests(address: string): Promise<FriendRequest[]> {
  try {
    const api = await getApi();
    const data = await api.query.chatCore.friendRequests(address);
    
    if (data.isEmpty) return [];
    
    const requests = data.toJSON() as any[];
    return requests.map((r: any) => ({
      id: r.id,
      fromAddress: r.fromAddress,
      fromName: r.fromName || '用户',
      fromAvatar: r.fromAvatar,
      message: r.message,
      createdAt: r.createdAt || Date.now(),
    }));
  } catch (error) {
    console.error('Failed to get friend requests:', error);
    return [];
  }
}

export async function sendFriendRequest(
  targetAddress: string,
  message: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .sendFriendRequest(targetAddress, message)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function acceptFriendRequest(
  requestId: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .acceptFriendRequest(requestId)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function rejectFriendRequest(
  requestId: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .rejectFriendRequest(requestId)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function removeFriend(
  friendAddress: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .removeFriend(friendAddress)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

export async function setFriendRemark(
  friendAddress: string,
  remark: string,
  mnemonic: string
): Promise<void> {
  const api = await getApi();
  const { Keyring } = await import('@polkadot/keyring');
  const keyring = new Keyring({ type: 'sr25519' });
  const pair = keyring.addFromMnemonic(mnemonic);

  return new Promise((resolve, reject) => {
    api.tx.chatCore
      .setFriendRemark(friendAddress, remark)
      .signAndSend(pair, ({ status, dispatchError }) => {
        if (status.isFinalized) {
          if (dispatchError) reject(new Error(dispatchError.toString()));
          else resolve();
        }
      })
      .catch(reject);
  });
}

function getMockFriends(): Friend[] {
  return [
    {
      id: '1',
      address: '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
      name: '金牌商家',
      status: 'accepted',
      isOnline: true,
      addedAt: Date.now() - 86400000,
    },
  ];
}
