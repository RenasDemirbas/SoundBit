// The only module that talks to the Rust backend. Stores call these functions; components
// never call Tauri directly.

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { Action, EditorState, Environment, PlaybackState, PresetInfo } from './types';

export const backend = {
  getState: () => invoke<EditorState>('get_state'),
  getEnvironment: () => invoke<Environment>('get_environment'),
  edit: (action: Action) => invoke<EditorState>('edit', { action }),
  undo: () => invoke<EditorState>('undo'),
  redo: () => invoke<EditorState>('redo'),
  newSong: (title: string) => invoke<EditorState>('new_song', { title }),
  openProject: (path: string) => invoke<EditorState>('open_project', { path }),
  saveProject: (path: string | null) => invoke<EditorState>('save_project', { path }),
  exportWav: (path: string) => invoke<string>('export_wav', { path }),
  play: (fromBar: number, looping: boolean) => invoke<PlaybackState>('play', { fromBar, looping }),
  stop: () => invoke<PlaybackState>('stop'),
  playbackState: () => invoke<PlaybackState>('playback_state'),
  listPresets: () => invoke<PresetInfo[]>('list_presets'),
  /** Called whenever the song changes, including edits made by an AI agent over MCP. */
  onSongChanged: (handler: () => void): Promise<UnlistenFn> => listen('song-changed', handler),
};

/** Turns a rejected backend call into a message for the user. */
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
