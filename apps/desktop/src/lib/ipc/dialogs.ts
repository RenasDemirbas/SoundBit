// Native file and confirmation dialogs.

import { ask, open, save } from '@tauri-apps/plugin-dialog';

const PROJECT_FILTER = { name: 'SoundBit project', extensions: ['sbproj'] };
const WAV_FILTER = { name: 'WAV audio', extensions: ['wav'] };

export async function chooseProjectToOpen(): Promise<string | null> {
  const path = await open({ multiple: false, directory: false, filters: [PROJECT_FILTER] });
  return typeof path === 'string' ? path : null;
}

export function chooseProjectToSave(): Promise<string | null> {
  return save({ filters: [PROJECT_FILTER], defaultPath: 'song.sbproj' });
}

export function chooseWavToSave(): Promise<string | null> {
  return save({ filters: [WAV_FILTER], defaultPath: 'song.wav' });
}

export function confirmDiscard(): Promise<boolean> {
  return ask('The song has unsaved changes. Discard them?', { title: 'SoundBit', kind: 'warning' });
}
