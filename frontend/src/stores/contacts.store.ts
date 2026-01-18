/**
 * 通讯录状态管理
 */

import { create } from 'zustand';
import {
  initContactsService,
  getContactsService,
} from '@/services/contacts.service';
import type {
  Contact,
  ContactGroup,
  BlockedUser,
  FriendRequest,
  ContactsStats,
} from '@/features/contacts/types';

interface ContactsState {
  // 状态
  contacts: Contact[];
  groups: ContactGroup[];
  blacklist: BlockedUser[];
  friendRequests: FriendRequest[];
  stats: ContactsStats | null;
  isLoading: boolean;
  error: string | null;

  // 初始化
  initialize: (address: string) => Promise<void>;

  // 联系人操作
  loadContacts: () => Promise<void>;
  addContact: (
    address: string,
    alias?: string,
    groups?: string[]
  ) => Promise<void>;
  removeContact: (address: string) => Promise<void>;
  updateContact: (
    address: string,
    alias?: string,
    groups?: string[]
  ) => Promise<void>;

  // 分组操作
  loadGroups: () => Promise<void>;
  createGroup: (name: string) => Promise<void>;
  deleteGroup: (name: string) => Promise<void>;
  renameGroup: (oldName: string, newName: string) => Promise<void>;

  // 黑名单操作
  loadBlacklist: () => Promise<void>;
  blockUser: (address: string, reason?: string) => Promise<void>;
  unblockUser: (address: string) => Promise<void>;

  // 好友申请操作
  loadFriendRequests: () => Promise<void>;
  sendFriendRequest: (target: string, message?: string) => Promise<void>;
  acceptFriendRequest: (requester: string) => Promise<void>;
  rejectFriendRequest: (requester: string) => Promise<void>;

  // 统计
  loadStats: () => Promise<void>;

  // 刷新全部
  refreshAll: () => Promise<void>;
}

export const useContactsStore = create<ContactsState>()((set, get) => ({
  contacts: [],
  groups: [],
  blacklist: [],
  friendRequests: [],
  stats: null,
  isLoading: false,
  error: null,

  initialize: async (address: string) => {
    set({ isLoading: true, error: null });

    try {
      const service = initContactsService(address);
      await service.init();
      await get().refreshAll();
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },

  loadContacts: async () => {
    set({ isLoading: true });
    try {
      const service = getContactsService();
      const contacts = await service.getAllContacts();
      set({ contacts, error: null });
    } catch (error) {
      set({ error: (error as Error).message });
    } finally {
      set({ isLoading: false });
    }
  },

  addContact: async (address, alias, groups) => {
    set({ isLoading: true });
    try {
      const service = getContactsService();
      await service.addContact(address, alias, groups);
      await get().loadContacts();
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  removeContact: async (address) => {
    set({ isLoading: true });
    try {
      const service = getContactsService();
      await service.removeContact(address);
      set((state) => ({
        contacts: state.contacts.filter((c) => c.address !== address),
      }));
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  updateContact: async (address, alias, groups) => {
    try {
      const service = getContactsService();
      await service.updateContact(address, alias, groups);
      await get().loadContacts();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  loadGroups: async () => {
    try {
      const service = getContactsService();
      const groups = await service.getAllGroups();
      set({ groups });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  createGroup: async (name) => {
    try {
      const service = getContactsService();
      await service.createGroup(name);
      await get().loadGroups();
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  deleteGroup: async (name) => {
    try {
      const service = getContactsService();
      await service.deleteGroup(name);
      set((state) => ({
        groups: state.groups.filter((g) => g.name !== name),
      }));
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  renameGroup: async (oldName, newName) => {
    try {
      const service = getContactsService();
      await service.renameGroup(oldName, newName);
      await get().loadGroups();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  loadBlacklist: async () => {
    try {
      const service = getContactsService();
      const blacklist = await service.getBlacklist();
      set({ blacklist });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  blockUser: async (address, reason) => {
    try {
      const service = getContactsService();
      await service.blockAccount(address, reason);
      await get().loadBlacklist();
      // 同时从联系人列表移除
      set((state) => ({
        contacts: state.contacts.filter((c) => c.address !== address),
      }));
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  unblockUser: async (address) => {
    try {
      const service = getContactsService();
      await service.unblockAccount(address);
      set((state) => ({
        blacklist: state.blacklist.filter((b) => b.address !== address),
      }));
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  loadFriendRequests: async () => {
    try {
      const service = getContactsService();
      const friendRequests = await service.getReceivedFriendRequests();
      set({ friendRequests });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  sendFriendRequest: async (target, message) => {
    try {
      const service = getContactsService();
      await service.sendFriendRequest(target, message);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  acceptFriendRequest: async (requester) => {
    try {
      const service = getContactsService();
      await service.acceptFriendRequest(requester);
      // 刷新好友申请和联系人列表
      await Promise.all([get().loadFriendRequests(), get().loadContacts()]);
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  rejectFriendRequest: async (requester) => {
    try {
      const service = getContactsService();
      await service.rejectFriendRequest(requester);
      set((state) => ({
        friendRequests: state.friendRequests.filter(
          (r) => r.requester !== requester
        ),
      }));
      await get().loadStats();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  loadStats: async () => {
    try {
      const service = getContactsService();
      const stats = await service.getStats();
      set({ stats });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  refreshAll: async () => {
    await Promise.all([
      get().loadContacts(),
      get().loadGroups(),
      get().loadBlacklist(),
      get().loadFriendRequests(),
      get().loadStats(),
    ]);
  },
}));
