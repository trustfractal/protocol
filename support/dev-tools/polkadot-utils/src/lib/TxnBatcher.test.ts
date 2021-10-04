import { TxnBatcher } from './TxnBatcher';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import settings from '../settings';

async function createPromiseApi(nodeAddress: string, types: any) {
    const provider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider, types });
    await api.isReady;
    return api;
}

describe('TxnBatcher', () => {
    it('immediately sends first txn', async () => {
        const fractalId = 1
        const api = await createPromiseApi(settings.nodeAddress, settings.types)
        const batcher = new TxnBatcher(api)
        const spy = jest.spyOn<any, any>(batcher, 'sendQueued')

        const keyring = new Keyring({ type: 'sr25519' })
        const signer = keyring.createFromUri('//Alice')
        const address = keyring.createFromUri('//Bob').address
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)

        expect(spy).toBeCalledTimes(1)
    });

      it('waits to send the second transaction', async () => {
        const fractalId = 1
        const api = await createPromiseApi(settings.nodeAddress, settings.types)
        const batcher = new TxnBatcher(api)
        const spy = jest.spyOn<any, any>(batcher, 'sendQueued')

        const keyring = new Keyring({ type: 'sr25519' })
        const signer = keyring.createFromUri('//Alice')
        const address = keyring.createFromUri('//Bob').address
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)

        expect(spy).toBeCalledTimes(2)
      });

      it('sends second transaction after first finishes', async () => {
        const fractalId = 1
        const api = await createPromiseApi(settings.nodeAddress, settings.types)
        const batcher = new TxnBatcher(api)
        const spy = jest.spyOn<any, any>(batcher, 'sendQueued')

        const keyring = new Keyring({ type: 'sr25519' })
        const signer = keyring.createFromUri('//Alice')
        const address = keyring.createFromUri('//Bob').address
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)
        batcher.send(api.tx.fractalMinting.registerIdentity(fractalId, address), signer)

        expect(spy).toBeCalledTimes(2)
      });
});
