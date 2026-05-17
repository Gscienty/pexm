import * as Y from 'yjs';
import {
  encodeSyncStep1Message,
  encodeSyncStep2Message,
  encodeUpdateMessage,
  decodeMessage,
  MessageType,
  DocMessageType,
} from './protocol';

export class DocWebSocketProvider {
  private ws: WebSocket | null = null;
  private doc: Y.Doc;
  private roomName: string;
  private url: string;
  private synced: boolean = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private updateHandler: (update: Uint8Array, origin: any) => void;

  constructor(
    doc: Y.Doc,
    roomName: string,
    options: { baseUrl?: string } = {},
  ) {
    this.doc = doc;
    this.roomName = roomName;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = options.baseUrl || window.location.host;
    this.url = `${protocol}//${host}/api/websocket/doc/${encodeURIComponent(roomName)}`;

    this.updateHandler = (update: Uint8Array, origin: any) => {
      console.log('[WS] update event fired, origin:', origin, 'update len:', update.length);
      if (origin === this) {
        console.log('[WS] update skipped (self-origin)');
        return;
      }
      this.sendUpdate(update);
    };

    this.connect();
  }

  private connect() {
    if (this.ws) {
      this.ws.close();
    }
    this.doc.off('update', this.updateHandler);
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
          this.doc.on('update', this.updateHandler);
          console.log('[WS] synced, updateHandler registered');
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
    this.doc.off('update', this.updateHandler);
    this.ws?.close();
    this.ws = null;
    this.synced = false;
  }

  destroy() {
    this.disconnect();
  }
}