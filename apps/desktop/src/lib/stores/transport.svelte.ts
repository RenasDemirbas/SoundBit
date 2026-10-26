// Play/stop and the play head position.

import { backend, errorMessage } from '../ipc/client';
import type { PlaybackState } from '../ipc/types';
import { editor } from './editor.svelte';

class TransportStore {
  playing = $state(false);
  /** Play head in bars (fractional), while playing. */
  position = $state<number | null>(null);
  /** Play head in seconds from the start of the song, while playing. */
  seconds = $state<number | null>(null);
  looping = $state(true);
  private polling = false;

  async play(fromBar = 0): Promise<void> {
    try {
      this.update(await backend.play(fromBar, this.looping));
      this.poll();
    } catch (error) {
      editor.notify(errorMessage(error), true);
    }
  }

  async stop(): Promise<void> {
    try {
      this.update(await backend.stop());
    } catch (error) {
      editor.notify(errorMessage(error), true);
    }
  }

  async toggle(): Promise<void> {
    await (this.playing ? this.stop() : this.play());
  }

  private update(state: PlaybackState): void {
    this.playing = state.playing;
    this.position = state.position_bars;
    this.seconds = state.position_seconds;
  }

  /** Follows the play head once per frame while playing. */
  private poll(): void {
    if (this.polling) return;
    this.polling = true;
    const step = async (): Promise<void> => {
      try {
        this.update(await backend.playbackState());
      } catch {
        this.playing = false;
      }
      if (this.playing) {
        requestAnimationFrame(() => void step());
      } else {
        this.polling = false;
        this.position = null;
        this.seconds = null;
      }
    };
    void step();
  }
}

export const transport = new TransportStore();
