import init, {
  Store,
  type Event,
  type EventInput,
  type EventUpdater,
  type Note,
  type NoteInput,
  type NoteUpdater,
} from './generated/wasm/store'

async function createStore(): Promise<Store> {
  await init()
  return new Store()
}

export {
  Store,
  createStore,
  type Event,
  type EventInput,
  type EventUpdater,
  type Note,
  type NoteInput,
  type NoteUpdater,
}
