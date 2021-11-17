import environment from '@popup/Environment';
import { MultiContext } from '@utils/MultiContext';
import { Storage } from '@utils/StorageArray';
import { ReplaySubject } from 'rxjs';

interface Tokens {
  scopes: string;
}

export class NotConnectedError extends Error {
  constructor(message?: string) {
    super(message);
    this.name = 'NotConnectedError';
  }
}

const NEXT_TOKENS_KEY = 'fractal-account-connector/will-accept-next-tokens';
const TOKENS_KEY = 'fractal-account-connector/tokens';

export class FractalAccountConnector extends MultiContext {
  tokens: Tokens | null = null;
  connectedAccount$ = new ReplaySubject<boolean>(1);

  constructor(private readonly storage: Storage) {
    super();

    this.getTokens().then((tokens) => {
      this.tokens = tokens;
      this.connectedAccount$.next(tokens != null);
    });
  }

  hasConnectedAccount(): boolean {
    return this.tokens != null;
  }

  async doConnect(urlOverride?: string) {
    await this.storage.setItem(NEXT_TOKENS_KEY, 'true');

    chrome.tabs.create({ url: urlOverride || environment.FRACTAL_WEBSITE_URL });
  }

  async willAcceptNextTokens(): Promise<boolean> {
    const stored = await this.storage.getItem(NEXT_TOKENS_KEY);
    return stored === 'true';
  }

  async setTokens(tokens: Tokens) {
    console.log('Storing session tokens', tokens);
    await this.storage.setItem(TOKENS_KEY, JSON.stringify(tokens));
    await this.storage.setItem(NEXT_TOKENS_KEY, 'false');
    // TODO(shelbyd): Show user a notification that the process completed.
  }

  async getTokens() {
    const stored = await this.storage.getItem(TOKENS_KEY);
    if (stored == null) return null;

    return JSON.parse(stored);
  }

  private async requireTokens() {
    const t = await this.getTokens();
    if (t == null) throw new NotConnectedError();
    return t;
  }

  async getCatfishToken() {
    return (await this.requireTokens()).catfish;
  }

  async getScopes() {
    return (await this.requireTokens()).scopes;
  }

  async clearTokens() {
    await this.storage.removeItem(TOKENS_KEY);
    this.tokens = null;
    this.connectedAccount$.next(false);
  }
}
