import { useState, useCallback } from 'react';
import { getApi } from '@/services/api';
import { Keyring } from '@polkadot/keyring';

type TxStatus = 'idle' | 'signing' | 'broadcasting' | 'inBlock' | 'finalized' | 'error';

interface UseTxResult {
  status: TxStatus;
  error: string | null;
  txHash: string | null;
  send: (pallet: string, method: string, args: any[], mnemonic: string) => Promise<string>;
  reset: () => void;
}

export function useTransaction(): UseTxResult {
  const [status, setStatus] = useState<TxStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const reset = useCallback(() => {
    setStatus('idle');
    setError(null);
    setTxHash(null);
  }, []);

  const send = useCallback(
    async (pallet: string, method: string, args: any[], mnemonic: string): Promise<string> => {
      try {
        setStatus('signing');
        setError(null);

        const api = await getApi();
        const keyring = new Keyring({ type: 'sr25519' });
        const pair = keyring.addFromMnemonic(mnemonic);

        const tx = api.tx[pallet][method](...args);

        setStatus('broadcasting');

        return new Promise((resolve, reject) => {
          tx.signAndSend(pair, ({ status, txHash, dispatchError }) => {
            if (status.isInBlock) {
              setStatus('inBlock');
              setTxHash(txHash.toHex());
            }

            if (status.isFinalized) {
              if (dispatchError) {
                if (dispatchError.isModule) {
                  const decoded = api.registry.findMetaError(dispatchError.asModule);
                  const errorMsg = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
                  setError(errorMsg);
                  setStatus('error');
                  reject(new Error(errorMsg));
                } else {
                  const errorMsg = dispatchError.toString();
                  setError(errorMsg);
                  setStatus('error');
                  reject(new Error(errorMsg));
                }
              } else {
                setStatus('finalized');
                resolve(txHash.toHex());
              }
            }
          }).catch((err) => {
            setError(err.message);
            setStatus('error');
            reject(err);
          });
        });
      } catch (err: any) {
        setError(err.message);
        setStatus('error');
        throw err;
      }
    },
    []
  );

  return { status, error, txHash, send, reset };
}
