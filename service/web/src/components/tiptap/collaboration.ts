import { Extension } from '@tiptap/core';
import * as Y from 'yjs';
import { ySyncPlugin, yUndoPlugin } from 'y-prosemirror';
import { DocWebSocketProvider } from '../../lib/ws-provider';

export interface CollaborationOptions {
  docName: string;
  wsUrl?: string;
  /** Base64-encoded initial document content (yjs update binary). */
  initialContent?: string;
}

export const Collaboration = Extension.create<CollaborationOptions>({
  name: 'collaboration',

  addOptions() {
    return {
      docName: '',
      wsUrl: '',
      initialContent: '',
    };
  },

  addStorage() {
    const doc = new Y.Doc();

    // Apply initial content before ySyncPlugin binds ProseMirror to the
    // Y.XmlFragment. This ensures the fragment is populated when the
    // editor first renders, preventing an empty-doc update from overwriting
    // the server-side content.
    const initialContent = this.options.initialContent;
    if (initialContent) {
      try {
        const binary = Uint8Array.from(atob(initialContent), (c) => c.charCodeAt(0));
        if (binary.length > 0) {
          Y.applyUpdate(doc, binary);
        }
      } catch (e) {
        console.warn('[Collaboration] Failed to apply initial content:', e);
      }
    }

    // Restore any unsynced local edits from sessionStorage.
    // This covers the case where the user edits and refreshes before
    // the WS connection has finished syncing. Applied AFTER initialContent
    // so that local edits (which are newer) take precedence via CRDT merge.
    const unsynced = DocWebSocketProvider.loadUnsyncedCache(this.options.docName);
    if (unsynced) {
      try {
        Y.applyUpdate(doc, unsynced);
        console.log('[Collaboration] Restored unsynced cache, len:', unsynced.length);
      } catch (e) {
        console.warn('[Collaboration] Failed to apply unsynced cache:', e);
      }
    }

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