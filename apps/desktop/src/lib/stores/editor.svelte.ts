// The editor state shown by every component, and the user actions that change it.

import { backend, errorMessage } from '../ipc/client';
import { chooseProjectToOpen, chooseProjectToSave, chooseWavToSave, confirmDiscard } from '../ipc/dialogs';
import type { Action, EditorState, Environment, Instrument, Pattern, PresetInfo } from '../ipc/types';

/** How long information messages stay in the status bar. */
const MESSAGE_SECONDS = 5;

export interface Message {
  text: string;
  error: boolean;
}

class EditorStore {
  state = $state<EditorState | null>(null);
  environment = $state<Environment | null>(null);
  presets = $state<PresetInfo[]>([]);
  selectedPattern = $state<number | null>(null);
  selectedInstrument = $state<number | null>(null);
  /** Length in steps of notes added in the pattern editor. */
  noteLength = $state(2);
  message = $state<Message | null>(null);
  private messageId = 0;

  song = $derived(this.state?.song ?? null);
  pattern = $derived<Pattern | null>(this.song?.patterns.find((p) => p.id === this.selectedPattern) ?? null);
  instrument = $derived<Instrument | null>(
    this.song?.instruments.find((i) => i.id === this.selectedInstrument) ?? null,
  );

  async init(): Promise<void> {
    await this.run(async () => {
      this.environment = await backend.getEnvironment();
      this.presets = await backend.listPresets();
      this.show(await backend.getState());
      // Edits from AI agents arrive here too.
      await backend.onSongChanged(() => void this.refresh());
    });
  }

  async refresh(): Promise<void> {
    await this.run(async () => this.show(await backend.getState()));
  }

  async act(action: Action): Promise<void> {
    await this.run(async () => this.show(await backend.edit(action)));
  }

  async undo(): Promise<void> {
    await this.run(async () => this.show(await backend.undo()));
  }

  async redo(): Promise<void> {
    await this.run(async () => this.show(await backend.redo()));
  }

  async newSong(): Promise<void> {
    if (this.state?.modified && !(await confirmDiscard())) return;
    await this.run(async () => this.show(await backend.newSong('Untitled')));
  }

  async open(): Promise<void> {
    if (this.state?.modified && !(await confirmDiscard())) return;
    const path = await chooseProjectToOpen();
    if (path) await this.run(async () => this.show(await backend.openProject(path)));
  }

  /** Saves to the current file, or asks where to save for new songs and "save as". */
  async save(saveAs = false): Promise<void> {
    let path: string | null = null;
    if (saveAs || !this.state?.file) {
      path = await chooseProjectToSave();
      if (!path) return;
    }
    await this.run(async () => {
      this.show(await backend.saveProject(path));
      this.notify(`Saved ${this.state?.file ?? ''}`);
    });
  }

  async exportWav(): Promise<void> {
    const path = await chooseWavToSave();
    if (path) await this.run(async () => this.notify(await backend.exportWav(path)));
  }

  /** Shows a message in the status bar. Information fades after a few seconds; errors stay
   * until the next message. */
  notify(text: string, error = false): void {
    // `$state` proxies the object, so messages are told apart by a counter, not identity.
    const id = ++this.messageId;
    this.message = { text, error };
    if (!error) {
      setTimeout(() => {
        if (this.messageId === id) this.message = null;
      }, MESSAGE_SECONDS * 1000);
    }
  }

  private show(state: EditorState): void {
    this.state = state;
    const { patterns, instruments } = state.song;
    if (!patterns.some((p) => p.id === this.selectedPattern)) {
      this.selectedPattern = patterns[0]?.id ?? null;
    }
    if (!instruments.some((i) => i.id === this.selectedInstrument)) {
      this.selectedInstrument = instruments[0]?.id ?? null;
    }
  }

  private async run(task: () => Promise<void>): Promise<void> {
    try {
      await task();
    } catch (error) {
      this.notify(errorMessage(error), true);
    }
  }
}

export const editor = new EditorStore();
