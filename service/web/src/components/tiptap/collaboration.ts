import { Extension } from '@tiptap/core';
import * as Y from 'yjs';
import { ySyncPlugin, yUndoPlugin } from 'y-prosemirror';
import { DocWebSocketProvider } from '../../lib/ws-provider';

export interface CollaborationOptions {
  docName: string;
  wsUrl?: string;
}

export const Collaboration = Extension.create<CollaborationOptions>({
  name: 'collaboration',

  addOptions() {
    return {
      docName: '',
      wsUrl: '',
    };
  },

  addStorage() {
    const doc = new Y.Doc();
    const yXmlFragment = doc.getXmlFragment('prosemirror');
    return {
      doc,
      provider: null as DocWebSocketProvider | null,
      yXmlFragment,
    };
  },

  onCreate() {
    const provider = new DocWebSocketProvider(
      this.storage.doc as Y.Doc,
      this.options.docName,
      {
        baseUrl: this.options.wsUrl || undefined,
      },
    );
    this.storage.provider = provider;
  },

  onDestroy() {
    const provider = this.storage.provider as DocWebSocketProvider | undefined;
    provider?.destroy();
    const doc = this.storage.doc as Y.Doc | undefined;
    doc?.destroy();
  },

  addProseMirrorPlugins() {
    const yXmlFragment = this.storage.yXmlFragment as Y.XmlFragment;
    return [ySyncPlugin(yXmlFragment), yUndoPlugin()];
  },
});