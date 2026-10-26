//! Mapping stereo `f32` frames onto a device's channel layout and sample format.

use cpal::{FromSample, Sample};

// RT-SAFE
/// Writes interleaved stereo frames into a device buffer with `channels` channels: mono
/// devices get the average, stereo devices get left and right, and extra channels are silent.
pub(crate) fn write_frames<T>(stereo: &[f32], out: &mut [T], channels: usize)
where
    T: Sample + FromSample<f32>,
{
    let (pairs, _) = stereo.as_chunks::<2>();
    for (frame, &[left, right]) in out.chunks_mut(channels).zip(pairs) {
        match frame {
            [] => {}
            [mono] => *mono = T::from_sample(f32::midpoint(left, right)),
            [first, second, rest @ ..] => {
                *first = T::from_sample(left);
                *second = T::from_sample(right);
                rest.fill(T::EQUILIBRIUM);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_passes_through() {
        let mut out = [0.0_f32; 4];
        write_frames(&[0.1, 0.2, 0.3, 0.4], &mut out, 2);
        assert_eq!(out, [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn mono_averages_and_surround_pads() {
        let mut mono = [0.0_f32; 2];
        write_frames(&[0.2, 0.4, -1.0, 1.0], &mut mono, 1);
        assert!((mono[0] - 0.3).abs() < 1e-6 && mono[1] == 0.0);

        let mut surround = [9.0_f32; 6];
        write_frames(&[0.5, -0.5, 1.0, -1.0], &mut surround, 3);
        assert_eq!(surround, [0.5, -0.5, 0.0, 1.0, -1.0, 0.0]);
    }

    #[test]
    fn converts_to_integer_formats() {
        let mut out = [0_i16; 2];
        write_frames(&[1.0, -1.0], &mut out, 2);
        assert_eq!(out, [i16::MAX, i16::MIN]);
        let mut unsigned = [0_u16; 2];
        write_frames(&[0.0, 0.0], &mut unsigned, 2);
        assert_eq!(unsigned, [u16::EQUILIBRIUM; 2]);
    }
}
