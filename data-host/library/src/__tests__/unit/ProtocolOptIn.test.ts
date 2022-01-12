import { ProtocolOptIn } from '@services/ProtocolOptIn';

class MockStorage {
  private items = new Map();

  setItem(key: string, value: string) {
    this.items.set(key, value);
  }
  getItem(key: string) {
    return this.items.get(key);
  }
  hasItem(key: string) {
    return this.items.has(key);
  }
}

type testOptions = {
  [key: string]: jest.Mock<any, any>;
};

describe('ProtocolOptIn', () => {
  function createSpyObj(props: any[]) {
    const obj: testOptions = {};
    for (const prop of props) {
      obj[prop] = jest.fn().mockName(prop);
    }
    return obj;
  }

  function graph(deps?: any) {
    deps = deps || {};

    const storage = deps.storage || new MockStorage();

    const protocol =
      deps.protocol || createSpyObj(['ensureIdentityRegistered']);

    const windows = deps.windows || createSpyObj(['openTab']);

    const livenessUrl = deps.livenessUrl || 'http://fractal-liveness.com';

    const optIn = new ProtocolOptIn(storage, protocol, windows, livenessUrl);
    return { storage, optIn, windows, livenessUrl, protocol };
  }

  it('starts as not isOptedIn', async () => {
    const { optIn } = graph();

    expect(await optIn.isOptedIn()).toEqual(false);
  });

  describe('optIn', () => {
    it('registers identity in Fractal', async () => {
      const { protocol, optIn } = graph();

      await optIn.optIn('some mnemonic');

      expect(protocol.ensureIdentityRegistered).toHaveBeenCalledWith();
    });

    it('isOptedIn is true', async () => {
      const { optIn } = graph();

      await optIn.optIn('some mnemonic');

      expect(await optIn.isOptedIn()).toEqual(true);
    });

    it('calls provided callbacks', async () => {
      const { optIn } = graph();
      const cb = jest.fn();
      optIn.postOptInCallbacks.push(cb);

      await optIn.optIn('some mnemonic');

      expect(cb).toHaveBeenCalled();
    });
  });

  it('loads isOptedIn from storage', async () => {
    const { storage, optIn } = graph();

    await optIn.optIn('some mnemonic');

    const { optIn: newOptIn } = graph({ storage });

    expect(await newOptIn.isOptedIn()).toEqual(true);
  });

  describe('postOptInLiveness', () => {
    it('tries to ensureIdentityRegistered', async () => {
      const { protocol, optIn } = graph();

      await optIn.optIn('some mnemonic');

      await optIn.postOptInLiveness();

      expect(protocol.ensureIdentityRegistered).toHaveBeenCalled();
    });

    it('sets liveness on instances', async () => {
      const { protocol, optIn } = graph();

      await optIn.optIn('some mnemonic');

      await optIn.postOptInLiveness();

      expect(protocol.ensureIdentityRegistered).toHaveBeenCalled();
    });

    it('opens liveness journey if no liveness', async () => {
      const { optIn, windows, livenessUrl } = graph();

      await optIn.optIn('some mnemonic');
      await optIn.postOptInLiveness();

      expect(windows.openTab).toHaveBeenCalledWith(livenessUrl);
    });
  });

  describe('re-opting in', () => {
    it('is not opted-in when active network changes', async () => {
      const { optIn } = graph();
      await optIn.optIn('some mnemonic');

      expect(await optIn.isOptedIn()).toEqual(false);
    });
  });

  describe('checkOptIn', () => {
    it('calls callbacks if opted in from another instance', async () => {
      const { storage, optIn } = graph();
      const { optIn: otherOptIn } = graph({ storage });
      await otherOptIn.optIn('some mnemonic');

      const cb = jest.fn();
      optIn.postOptInCallbacks.push(cb);
      await optIn.checkOptIn();

      expect(cb).toHaveBeenCalled();
    });

    it('does not call callbacks if already called', async () => {
      const { storage, optIn } = graph();
      const { optIn: otherOptIn } = graph({ storage });
      await otherOptIn.optIn('some mnemonic');

      const cb = jest.fn();
      optIn.postOptInCallbacks.push(cb);
      await optIn.checkOptIn();
      await optIn.checkOptIn();

      expect(cb.mock.calls.length).toEqual(1);
    });
  });
});
