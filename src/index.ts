import init, { Store } from './generated/wasm/store'

async function createStore(): Promise<Store> {
  await init()
  return new Store()
}

export { Store, createStore }
