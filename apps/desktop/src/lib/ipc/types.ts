// Types mirrored from the Rust side. The song uses the project-file JSON shape produced by
// `soundbit-core`; the rest mirrors `src-tauri/src/dto.rs` and `commands/action.rs`.

export type ChipProfile = 'free' | 'nes' | 'game_boy' | 'c64' | 'genesis';

export type ScaleKind =
  | 'chromatic'
  | 'major'
  | 'natural_minor'
  | 'harmonic_minor'
  | 'melodic_minor'
  | 'dorian'
  | 'phrygian'
  | 'lydian'
  | 'mixolydian'
  | 'locrian'
  | 'major_pentatonic'
  | 'minor_pentatonic'
  | 'blues'
  | 'whole_tone';

export type DutyCycle = 'eighth' | 'quarter' | 'half' | 'three_quarters';
export type NoiseMode = 'long' | 'short';

export interface FmOperator {
  multiple: number;
  detune: number;
  total_level: number;
  attack_rate: number;
  decay_rate: number;
  sustain_rate: number;
  sustain_level: number;
  release_rate: number;
}

export type Voice =
  | { type: 'pulse'; duty: DutyCycle }
  | { type: 'triangle' }
  | { type: 'sawtooth' }
  | { type: 'sine' }
  | { type: 'noise'; mode: NoiseMode }
  | { type: 'wavetable'; samples: number[] }
  | { type: 'fm'; algorithm: number; feedback: number; operators: FmOperator[] };

export type VoiceKind = Voice['type'];

export interface Envelope {
  attack: number;
  decay: number;
  sustain: number;
  release: number;
}

/** Pitch glide at note start: `semitones` away, gliding to the note over `time` ms. */
export interface PitchEnvelope {
  semitones: number;
  time: number;
}

/** `depth` in cents, `rate` in tenths of a hertz, `delay` in ms. */
export interface Vibrato {
  depth: number;
  rate: number;
  delay: number;
}

/** `voices` copies, outermost detuned by `detune` cents. */
export interface Unison {
  voices: number;
  detune: number;
}

export type FilterMode = 'off' | 'low_pass' | 'high_pass' | 'band_pass';

/** `cutoff` in Hz, `resonance` in percent, `envelope_depth` in semitones. */
export interface Filter {
  mode: FilterMode;
  cutoff: number;
  resonance: number;
  envelope_depth: number;
}

export interface Instrument {
  id: number;
  name: string;
  voice: Voice;
  envelope: Envelope;
  pitch_envelope: PitchEnvelope;
  vibrato: Vibrato;
  unison: Unison;
  filter: Filter;
  volume: number;
  pan: number;
  reverb_send: number;
}

export interface Note {
  pitch: number;
  start: number;
  length: number;
  velocity: number;
}

export interface Pattern {
  id: number;
  name: string;
  instrument: number;
  notes: Note[];
}

export interface Clip {
  bar: number;
  pattern: number;
}

export interface Track {
  id: number;
  name: string;
  muted: boolean;
  volume: number;
  clips: Clip[];
}

export interface Song {
  meta: { title: string; author: string; description: string };
  timing: { bpm: number; steps_per_beat: number; beats_per_bar: number; swing: number };
  scale: { root: number; kind: ScaleKind };
  chip_profile: ChipProfile;
  reverb: { size: number; damping: number };
  instruments: Instrument[];
  patterns: Pattern[];
  tracks: Track[];
  loop_range: { start: number; end: number } | null;
}

export interface EditorState {
  editor_id: number;
  revision: number;
  song: Song;
  steps_per_bar: number;
  length_bars: number;
  length_seconds: number;
  scale_classes: number[];
  channels: (string | null)[];
  problems: string[];
  undo: string | null;
  redo: string | null;
  file: string | null;
  modified: boolean;
}

export interface Environment {
  audio_device: string | null;
  audio_error: string | null;
  mcp_url: string | null;
  mcp_error: string | null;
  workspace: string;
}

/** A built-in instrument preset, mirrored by `PresetInfo` in `commands/presets.rs`. */
export interface PresetInfo {
  id: string;
  name: string;
  category: string;
  description: string;
  voice: VoiceKind;
}

export interface PlaybackState {
  playing: boolean;
  position_bars: number | null;
  position_seconds: number | null;
}

/** One edit, mirrored by `Action` in `commands/action.rs`. */
export type Action =
  | {
      kind: 'update_song';
      title?: string;
      bpm?: number;
      steps_per_beat?: number;
      beats_per_bar?: number;
      swing?: number;
      chip_profile?: ChipProfile;
      scale_root?: number;
      scale_kind?: ScaleKind;
      reverb_size?: number;
      reverb_damping?: number;
    }
  | { kind: 'add_instrument'; name: string; voice: Voice }
  | { kind: 'add_preset'; preset: string }
  | ({ kind: 'update_instrument'; id: number } & InstrumentChanges)
  | { kind: 'remove_instrument'; id: number }
  | { kind: 'create_pattern'; name: string; instrument: number }
  | { kind: 'update_pattern'; id: number; name?: string; instrument?: number }
  | { kind: 'duplicate_pattern'; id: number }
  | { kind: 'remove_pattern'; id: number }
  | { kind: 'transpose_pattern'; id: number; semitones: number }
  | { kind: 'toggle_note'; pattern: number; start: number; pitch: number; length: number }
  | { kind: 'add_track'; name: string }
  | { kind: 'update_track'; id: number; name?: string; muted?: boolean; volume?: number }
  | { kind: 'remove_track'; id: number }
  | { kind: 'set_clip'; track: number; bar: number; pattern: number | null };

/** Instrument fields the editor can change; groups are sent whole. */
export interface InstrumentChanges {
  name?: string;
  voice?: Voice;
  envelope?: Envelope;
  pitch_envelope?: PitchEnvelope;
  vibrato?: Vibrato;
  unison?: Unison;
  filter?: Filter;
  volume?: number;
  pan?: number;
  reverb_send?: number;
}
