import * as Y from 'yjs';
import {
  encodeSyncStep1Message,
  encodeSyncStep2Message,
  encodeUpdateMessage,
  decodeMessage,
  MessageType,
  DocMessageType,
} from './protocol';

const STORAGE_KEY_PREFIX = 'yjs_unsynced_';

export class DocWebSocketProvider {
  private ws: WebSocket | null = null;
  private doc: Y.Doc;
  private roomName: string;
  private url: string;
  private synced: boolean = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private updateHandler: (update: Uint8Array, origin: any) => void;
  private handlerRegistered: boolean = false;
  private storageKey: string;

  constructor(
    doc: Y.Doc,
    roomName: string,
    options: { baseUrl?: string } = {},
  ) {
    this.doc = doc;
    this.roomName = roomName;
    this.storageKey = STORAGE_KEY_PREFIX + roomName;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = options.baseUrl || window.location.host;
    this.url = `${protocol}//${host}/api/websocket/doc/${encodeURIComponent(roomName)}`;

    this.updateHandler = (update: Uint8Array, origin: any) => {
      console.log('[WS] update event fired, origin:', origin, 'update len:', update.length);
      if (origin === this) {
        console.log('[WS] update skipped (self-origin)');
        return;
      }
      // Cache local edits to sessionStorage so they survive a page refresh
      // before the server acknowledges them.
      this.cacheUnsynced();
      this.sendUpdate(update);
    };

    this.connect();
  }

  /** Persist the current doc state to sessionStorage.
   *  Called on every local edit so that a refresh before WS sync
   *  doesn't lose data. Cleared once the server acknowledges sync. */
  private cacheUnsynced() {
    try {
      const update = Y.encodeStateAsUpdate(this.doc);
      const base64 = btoa(String.fromCharCode(...update));
      sessionStorage.setItem(this.storageKey, base64);
    } catch (e) {
      console.warn('[WS] Failed to cache unsynced content:', e);
    }
  }

  /** Remove the unsynced cache. Called after a successful sync with the
   *  server, which means the server has our latest state. */
  private clearUnsyncedCache() {
    try {
      sessionStorage.removeItem(this.storageKey);
    } catch (_) {}
  }

  /** Load any previously cached unsynced content for this room.
   *  Should be called once during doc initialization, before the WS
   *  connection is established. */
  static loadUnsyncedCache(roomName: string): Uint8Array | null {
    try {
      const base64 = sessionStorage.getItem(STORAGE_KEY_PREFIX + roomName);
      if (!base64) return null;
      const binary = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
      return binary.length > 0 ? binary : null;
    } catch (_) {
      return null;
    }
  }

  private connect() {
    if (this.ws) {
      this.ws.close();
    }
    this.unregisterUpdateHandler();
    this.synced = false;

    this.ws = new WebSocket(this.url);
    this.ws.binaryType = 'arraybuffer';

    this.ws.onopen = () => {
      this.sendStep1();
    };

    this.ws.onmessage = (event: MessageEvent) => {
      const data = new Uint8Array(event.data);
      const msg = decodeMessage(data);
      if (!msg) return;

      switch (msg.messageType) {
        case MessageType.Sync:
        case MessageType.SyncReply:
          this.handleSync(msg);
          break;
        case MessageType.Close:
          this.disconnect();
          break;
      }
    };

    this.ws.onclose = (event: CloseEvent) => {
      console.log('[WS] onclose, code:', event.code, 'reason:', event.reason, 'wasClean:', event.wasClean);
      this.synced = false;
      this.scheduleReconnect();
    };

    this.ws.onerror = (event: Event) => {
      console.log('[WS] onerror:', event);
      this.ws?.close();
    };
  }

  private registerUpdateHandler() {
    if (this.handlerRegistered) return;
    this.doc.on('update', this.updateHandler);
    this.handlerRegistered = true;
    console.log('[WS] synced, updateHandler registered');
  }

  private unregisterUpdateHandler() {
    if (!this.handlerRegistered) return;
    this.doc.off('update', this.updateHandler);
    this.handlerRegistered = false;
  }

  private handleSync(msg: NonNullable<ReturnType<typeof decodeMessage>>) {
    if (msg.docMessageType === undefined) return;

    switch (msg.docMessageType) {
      case DocMessageType.Step1: {
        if (msg.data) {
          const update = Y.encodeStateAsUpdate(this.doc, msg.data);
          const encoded = encodeSyncStep2Message(this.roomName, update);
          this.ws?.send(encoded);
        }
        break;
      }
      case DocMessageType.Step2:
      case DocMessageType.Update: {
        if (msg.data && msg.data.length > 0) {
          console.log('[WS] applying remote update, len:', msg.data.length);
          Y.applyUpdate(this.doc, msg.data, this);
        }
        if (!this.synced) {
          this.synced = true;
          this.registerUpdateHandler();
          // Server has acknowledged our state — safe to clear the
          // unsynced cache since the server now has our latest edits.
          this.clearUnsyncedCache();
        }
        break;
      }
    }
  }

  private sendStep1() {
    const sv = Y.encodeStateVector(this.doc);
    const encoded = encodeSyncStep1Message(this.roomName, sv);
    console.log('[WS] sending Step1, len:', encoded.length);
    this.ws?.send(encoded);
  }

  private sendUpdate(update: Uint8Array) {
    if (!this.ws) {
      console.log('[WS] sendUpdate skipped, ws is null');
      return;
    }
    if (this.ws.readyState !== WebSocket.OPEN) {
      console.log('[WS] sendUpdate skipped, ws state:', this.ws.readyState, 'url:', this.ws.url);
      return;
    }
    const encoded = encodeUpdateMessage(this.roomName, update);
    console.log('[WS] sending update, len:', encoded.length, 'url:', this.ws.url);
    this.ws.send(encoded);
    console.log('[WS] update sent, ws state:', this.ws.readyState);
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, 3000);
  }

  disconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.unregisterUpdateHandler();
    this.ws?.close();
    this.ws = null;
    this.synced = false;
  }

  destroy() {
    this.disconnect();
  }
}
