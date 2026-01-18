// frontend/src/divination/market/services/ipfs.service.ts

import { Platform } from 'react-native';

// IPFS 网关配置
const IPFS_GATEWAYS = [
  'https://ipfs.io/ipfs/',
  'https://gateway.pinata.cloud/ipfs/',
  'https://cloudflare-ipfs.com/ipfs/',
];

// IPFS API 配置 (用于上传)
const IPFS_API_URL = process.env.EXPO_PUBLIC_IPFS_API_URL || 'https://ipfs.infura.io:5001/api/v0';
const IPFS_PROJECT_ID = process.env.EXPO_PUBLIC_IPFS_PROJECT_ID || '';
const IPFS_PROJECT_SECRET = process.env.EXPO_PUBLIC_IPFS_PROJECT_SECRET || '';

interface IpfsUploadResult {
  cid: string;
  size: number;
}

/**
 * 上传内容到 IPFS
 */
export async function uploadToIpfs(content: string | Uint8Array): Promise<IpfsUploadResult> {
  const formData = new FormData();

  if (typeof content === 'string') {
    // 文本内容
    const blob = new Blob([content], { type: 'text/plain' });
    formData.append('file', blob);
  } else {
    // 二进制内容
    const blob = new Blob([content], { type: 'application/octet-stream' });
    formData.append('file', blob);
  }

  const headers: Record<string, string> = {};
  if (IPFS_PROJECT_ID && IPFS_PROJECT_SECRET) {
    const auth = Buffer.from(`${IPFS_PROJECT_ID}:${IPFS_PROJECT_SECRET}`).toString('base64');
    headers['Authorization'] = `Basic ${auth}`;
  }

  const response = await fetch(`${IPFS_API_URL}/add`, {
    method: 'POST',
    headers,
    body: formData,
  });

  if (!response.ok) {
    throw new Error(`IPFS upload failed: ${response.statusText}`);
  }

  const result = await response.json();
  return {
    cid: result.Hash,
    size: parseInt(result.Size, 10),
  };
}

/**
 * 从 IPFS 获取内容
 */
export async function fetchFromIpfs(cid: string): Promise<string> {
  // 尝试多个网关
  for (const gateway of IPFS_GATEWAYS) {
    try {
      const response = await fetch(`${gateway}${cid}`, {
        method: 'GET',
        headers: {
          'Accept': 'text/plain',
        },
      });

      if (response.ok) {
        return await response.text();
      }
    } catch (error) {
      console.warn(`Gateway ${gateway} failed:`, error);
      continue;
    }
  }

  throw new Error(`Failed to fetch CID ${cid} from all gateways`);
}

/**
 * 从 IPFS 获取二进制内容
 */
export async function fetchBinaryFromIpfs(cid: string): Promise<Uint8Array> {
  for (const gateway of IPFS_GATEWAYS) {
    try {
      const response = await fetch(`${gateway}${cid}`, {
        method: 'GET',
      });

      if (response.ok) {
        const buffer = await response.arrayBuffer();
        return new Uint8Array(buffer);
      }
    } catch (error) {
      console.warn(`Gateway ${gateway} failed:`, error);
      continue;
    }
  }

  throw new Error(`Failed to fetch CID ${cid} from all gateways`);
}

/**
 * 获取 IPFS 内容的 URL
 */
export function getIpfsUrl(cid: string, gatewayIndex: number = 0): string {
  const gateway = IPFS_GATEWAYS[gatewayIndex] || IPFS_GATEWAYS[0];
  return `${gateway}${cid}`;
}

/**
 * 验证 CID 格式
 */
export function isValidCid(cid: string): boolean {
  // 简单的 CID 格式验证
  // CIDv0: Qm 开头，46 字符
  // CIDv1: b 开头，变长
  if (!cid) return false;
  if (cid.startsWith('Qm') && cid.length === 46) return true;
  if (cid.startsWith('b') && cid.length >= 50) return true;
  return false;
}

/**
 * 上传 JSON 到 IPFS
 */
export async function uploadJsonToIpfs(data: object): Promise<IpfsUploadResult> {
  const content = JSON.stringify(data);
  return uploadToIpfs(content);
}

/**
 * 从 IPFS 获取 JSON
 */
export async function fetchJsonFromIpfs<T>(cid: string): Promise<T> {
  const content = await fetchFromIpfs(cid);
  return JSON.parse(content) as T;
}
