import App from '@popup/app';
export class InjectionScript {

  async setup(id: string): Promise<any> {
    console.log('InjectionScript initialized.');
    App(id);
  }
}
