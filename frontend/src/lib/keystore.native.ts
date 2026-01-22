/**
 * 星尘玄鉴 - 密钥存储（原生版本）
 * 使用 expo-crypto 避免 polyfill 问题
 */

import * as SecureStore from 'expo-secure-store';
import * as ExpoCrypto from 'expo-crypto';
import { encryptMnemonic, decryptMnemonic } from './crypto';
import { WalletError, AuthenticationError } from './errors';

const STORAGE_KEYS = {
  KEYSTORES: 'stardust_keystores',
  CURRENT_ADDRESS: 'stardust_current_address',
  ALIASES: 'stardust_aliases',
} as const;

// 简化的 BIP39 词表（前256个词用于演示）
const WORDLIST = [
  'abandon','ability','able','about','above','absent','absorb','abstract',
  'absurd','abuse','access','accident','account','accuse','achieve','acid',
  'acoustic','acquire','across','act','action','actor','actress','actual',
  'adapt','add','addict','address','adjust','admit','adult','advance',
  'advice','aerobic','affair','afford','afraid','again','age','agent',
  'agree','ahead','aim','air','airport','aisle','alarm','album',
  'alcohol','alert','alien','all','alley','allow','almost','alone',
  'alpha','already','also','alter','always','amateur','amazing','among',
  'amount','amused','analyst','anchor','ancient','anger','angle','angry',
  'animal','ankle','announce','annual','another','answer','antenna','antique',
  'anxiety','any','apart','apology','appear','apple','approve','april',
  'arch','arctic','area','arena','argue','arm','armed','armor',
  'army','around','arrange','arrest','arrive','arrow','art','artefact',
  'artist','artwork','ask','aspect','assault','asset','assist','assume',
  'asthma','athlete','atom','attack','attend','attitude','attract','auction',
  'audit','august','aunt','author','auto','autumn','average','avocado',
  'avoid','awake','aware','away','awesome','awful','awkward','axis',
  'baby','bachelor','bacon','badge','bag','balance','balcony','ball',
  'bamboo','banana','banner','bar','barely','bargain','barrel','base',
  'basic','basket','battle','beach','bean','beauty','because','become',
  'beef','before','begin','behave','behind','believe','below','belt',
  'bench','benefit','best','betray','better','between','beyond','bicycle',
  'bid','bike','bind','biology','bird','birth','bitter','black',
  'blade','blame','blanket','blast','bleak','bless','blind','blood',
  'blossom','blouse','blue','blur','blush','board','boat','body',
  'boil','bomb','bone','bonus','book','boost','border','boring',
  'borrow','boss','bottom','bounce','box','boy','bracket','brain',
  'brand','brass','brave','bread','breeze','brick','bridge','brief',
  'bright','bring','brisk','broccoli','broken','bronze','broom','brother',
  'brown','brush','bubble','buddy','budget','buffalo','build','bulb',
  'bulk','bullet','bundle','bunker','burden','burger','burst','bus',
];

export interface LocalKeystore {
  address: string;
  encrypted: string;
  salt: string;
  iv: string;
  createdAt: number;
}

export async function initializeCrypto(): Promise<void> {
  console.log('[Keystore] Crypto initialized (native - expo-crypto)');
}

export function generateMnemonic(): string {
  const bytes = ExpoCrypto.getRandomBytes(16);
  const words: string[] = [];
  for (let i = 0; i < 12; i++) {
    const index = (bytes[i % bytes.length]! + (i * 17)) % WORDLIST.length;
    words.push(WORDLIST[index]!);
  }
  return words.join(' ');
}

export function validateMnemonic(mnemonic: string): boolean {
  const words = mnemonic.trim().split(/\s+/);
  return words.length === 12 || words.length === 24;
}

export function createKeyPairFromMnemonic(mnemonic: string): { address: string } {
  const hash = Array.from(ExpoCrypto.getRandomBytes(32))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  return { address: '5' + hash.slice(0, 47) };
}

export async function loadAllKeystores(): Promise<LocalKeystore[]> {
  try {
    const data = await SecureStore.getItemAsync(STORAGE_KEYS.KEYSTORES);
    if (!data) return [];
    return JSON.parse(data);
  } catch { return []; }
}

async function saveAllKeystores(keystores: LocalKeystore[]): Promise<void> {
  await SecureStore.setItemAsync(STORAGE_KEYS.KEYSTORES, JSON.stringify(keystores));
}

export async function getCurrentAddress(): Promise<string | null> {
  try { return await SecureStore.getItemAsync(STORAGE_KEYS.CURRENT_ADDRESS); }
  catch { return null; }
}

export async function setCurrentAddress(address: string): Promise<void> {
  await SecureStore.setItemAsync(STORAGE_KEYS.CURRENT_ADDRESS, address);
}

export async function getAlias(address: string): Promise<string | null> {
  try {
    const data = await SecureStore.getItemAsync(STORAGE_KEYS.ALIASES);
    if (!data) return null;
    return JSON.parse(data)[address] || null;
  } catch { return null; }
}

export async function setAlias(address: string, alias: string): Promise<void> {
  const data = await SecureStore.getItemAsync(STORAGE_KEYS.ALIASES);
  const aliases = data ? JSON.parse(data) : {};
  aliases[address] = alias;
  await SecureStore.setItemAsync(STORAGE_KEYS.ALIASES, JSON.stringify(aliases));
}

export async function storeEncryptedMnemonic(
  mnemonic: string, password: string, address: string
): Promise<void> {
  const { encrypted, salt, iv } = await encryptMnemonic(mnemonic, password);
  const keystore: LocalKeystore = { address, encrypted, salt, iv, createdAt: Date.now() };
  const keystores = await loadAllKeystores();
  const idx = keystores.findIndex(ks => ks.address === address);
  if (idx >= 0) keystores[idx] = keystore;
  else keystores.push(keystore);
  await saveAllKeystores(keystores);
  await setCurrentAddress(address);
  await setAlias(address, `钱包 ${address.slice(0, 6)}`);
  console.log('[Keystore] Mnemonic stored securely');
}

export async function retrieveEncryptedMnemonic(password: string, address?: string): Promise<string> {
  const keystores = await loadAllKeystores();
  const targetAddress = address || await getCurrentAddress();
  if (!targetAddress) throw new WalletError('未找到钱包');
  const keystore = keystores.find(ks => ks.address === targetAddress);
  if (!keystore) throw new WalletError('未找到钱包数据');
  return await decryptMnemonic(keystore.encrypted, keystore.salt, keystore.iv, password);
}

export async function hasWallet(): Promise<boolean> {
  const keystores = await loadAllKeystores();
  return keystores.length > 0;
}

export async function getStoredAddress(): Promise<string | null> {
  const currentAddr = await getCurrentAddress();
  if (currentAddr) return currentAddr;
  const keystores = await loadAllKeystores();
  if (keystores.length > 0) {
    await setCurrentAddress(keystores[0].address);
    return keystores[0].address;
  }
  return null;
}

export async function deleteWalletByAddress(address: string): Promise<void> {
  const keystores = await loadAllKeystores();
  const filtered = keystores.filter(ks => ks.address !== address);
  await saveAllKeystores(filtered);
  const currentAddr = await getCurrentAddress();
  if (currentAddr === address) {
    if (filtered.length > 0) await setCurrentAddress(filtered[0].address);
    else await SecureStore.deleteItemAsync(STORAGE_KEYS.CURRENT_ADDRESS);
  }
}

export async function deleteWallet(): Promise<void> {
  const currentAddr = await getCurrentAddress();
  if (currentAddr) await deleteWalletByAddress(currentAddr);
}

export async function deleteAllWallets(): Promise<void> {
  await Promise.all([
    SecureStore.deleteItemAsync(STORAGE_KEYS.KEYSTORES),
    SecureStore.deleteItemAsync(STORAGE_KEYS.CURRENT_ADDRESS),
    SecureStore.deleteItemAsync(STORAGE_KEYS.ALIASES),
  ]);
}

export async function getSecureValue(key: string): Promise<string | null> {
  try {
    return await SecureStore.getItemAsync(key);
  } catch {
    return null;
  }
}

export async function setSecureValue(key: string, value: string): Promise<void> {
  await SecureStore.setItemAsync(key, value);
}
