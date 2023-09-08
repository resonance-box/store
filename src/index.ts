import init, { Store } from './generated/wasm/store'

async function createStore(): Promise<Store> {
  await init()
  return new Store()
}

export {
  Store,
  type Event,
  type EventUpdater,
  type Note,
  type NoteUpdater,
} from './generated/wasm/store'
export { createStore }
