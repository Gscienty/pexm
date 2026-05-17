const encoder = new TextEncoder();
const decoder = new TextDecoder();

export enum MessageType {
  Sync = 0,
  Awareness = 1,
  Auth = 2,
  QueryAwareness = 3,
  SyncReply = 4,
  Stateless = 5,
  BroadcastStateless = 6,
  Close = 7,
  SyncStatus = 8,
}

export enum DocMessageType {
  Step1 = 0,
  Step2 = 1,
  Update = 2,
}

export function writeVarUint(value: number): Uint8Array {
  const bytes: number[] = [];
  let v = value;
  while (v >= 0x80) {
    bytes.push((v & 0x7f) | 0x80);
    v >>>= 7;
  }
  bytes.push(v & 0x7f);
  return new Uint8Array(bytes);
}

export function writeVarString(value: string): Uint8Array {
  const encoded = encoder.encode(value);
  const len = writeVarUint(encoded.length);
  const result = new Uint8Array(len.length + encoded.length);
  result.set(len);
  result.set(encoded, len.length);
  return result;
}

export function writeVarBuffer(value: Uint8Array): Uint8Array {
  const len = writeVarUint(value.length);
  const result = new Uint8Array(len.length + value.length);
  result.set(len);
  result.set(value, len.length);
  return result;
}

export function readVarUint(data: Uint8Array, offset: number): [number, number] {
  let result = 0;
  let shift = 0;
  let i = offset;
  while (true) {
    const byte = data[i];
    result |= (byte & 0x7f) << shift;
    i++;
    if ((byte & 0x80) === 0) break;
    shift += 7;
  }
  return [result, i];
}

export function readVarString(data: Uint8Array, offset: number): [string, number] {
  const [len, newOffset] = readVarUint(data, offset);
  const str = decoder.decode(data.slice(newOffset, newOffset + len));
  return [str, newOffset + len];
}

export function readVarBuffer(data: Uint8Array, offset: number): [Uint8Array, number] {
  const [len, newOffset] = readVarUint(data, offset);
  const buf = data.slice(newOffset, newOffset + len);
  return [buf, newOffset + len];
}

export function encodeSyncStep1Message(name: string, stateVector: Uint8Array): Uint8Array {
  const parts: Uint8Array[] = [];
  parts.push(writeVarString(name));
  parts.push(writeVarUint(MessageType.Sync));
  parts.push(writeVarUint(DocMessageType.Step1));
  parts.push(writeVarBuffer(stateVector));
  return concatUint8Arrays(parts);
}

export function encodeSyncStep2Message(name: string, update: Uint8Array): Uint8Array {
  const parts: Uint8Array[] = [];
  parts.push(writeVarString(name));
  parts.push(writeVarUint(MessageType.Sync));
  parts.push(writeVarUint(DocMessageType.Step2));
  parts.push(writeVarBuffer(update));
  return concatUint8Arrays(parts);
}

export function encodeUpdateMessage(name: string, update: Uint8Array): Uint8Array {
  const parts: Uint8Array[] = [];
  parts.push(writeVarString(name));
  parts.push(writeVarUint(MessageType.Sync));
  parts.push(writeVarUint(DocMessageType.Update));
  parts.push(writeVarBuffer(update));
  return concatUint8Arrays(parts);
}

export function encodeAwarenessMessage(name: string, awarenessUpdate: Uint8Array): Uint8Array {
  const parts: Uint8Array[] = [];
  parts.push(writeVarString(name));
  parts.push(writeVarUint(MessageType.Awareness));
  parts.push(writeVarBuffer(awarenessUpdate));
  return concatUint8Arrays(parts);
}

export function encodeQueryAwarenessMessage(name: string): Uint8Array {
  const parts: Uint8Array[] = [];
  parts.push(writeVarString(name));
  parts.push(writeVarUint(MessageType.QueryAwareness));
  return concatUint8Arrays(parts);
}

export interface DecodedMessage {
  name: string;
  messageType: MessageType;
  docMessageType?: DocMessageType;
  data?: Uint8Array;
  syncStatus?: number;
}

export function decodeMessage(data: Uint8Array): DecodedMessage | null {
  try {
    let offset = 0;
    const [name, off1] = readVarString(data, offset);
    offset = off1;
    const [msgType, off2] = readVarUint(data, offset);
    offset = off2;

    const result: DecodedMessage = { name, messageType: msgType as MessageType };

    if (msgType === MessageType.Sync || msgType === MessageType.SyncReply) {
      const [docMsgType, off3] = readVarUint(data, offset);
      offset = off3;
      result.docMessageType = docMsgType as DocMessageType;

      if (docMsgType === DocMessageType.Step1 || docMsgType === DocMessageType.Step2 || docMsgType === DocMessageType.Update) {
        const [buf] = readVarBuffer(data, offset);
        result.data = buf;
      }
    } else if (msgType === MessageType.Awareness) {
      const [buf] = readVarBuffer(data, offset);
      result.data = buf;
    } else if (msgType === MessageType.SyncStatus) {
      const [status] = readVarUint(data, offset);
      result.syncStatus = status;
    }

    return result;
  } catch {
    return null;
  }
}

function concatUint8Arrays(arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((sum, arr) => sum + arr.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }
  return result;
}