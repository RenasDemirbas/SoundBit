//! 16-bit PCM WAV export.

use std::io::Write;

use crate::FormatError;

const BITS_PER_SAMPLE: u16 = 16;
const BYTES_PER_SAMPLE: u16 = BITS_PER_SAMPLE / 8;
const PCM_FORMAT: u16 = 1;
/// Size of the RIFF header fields that precede the sample data.
const HEADER_LEN: u32 = 36;

/// Writes interleaved samples in `[-1, 1]` as a 16-bit PCM WAV file. Values outside the range
/// are clipped.
///
/// # Errors
///
/// Returns [`FormatError::AudioTooLong`] if the data exceeds the 4 GiB WAV limit and
/// [`FormatError::Io`] if writing fails.
pub fn write_wav<W: Write>(
    mut writer: W,
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
) -> Result<(), FormatError> {
    let data_len = samples
        .len()
        .checked_mul(usize::from(BYTES_PER_SAMPLE))
        .and_then(|len| u32::try_from(len).ok())
        .filter(|len| len.checked_add(HEADER_LEN).is_some())
        .ok_or(FormatError::AudioTooLong)?;
    let block_align = channels * BYTES_PER_SAMPLE;
    let byte_rate = sample_rate * u32::from(block_align);

    let mut bytes = Vec::with_capacity(44 + samples.len() * usize::from(BYTES_PER_SAMPLE));
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(HEADER_LEN + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&PCM_FORMAT.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&to_pcm16(*sample).to_le_bytes());
    }
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "the value is clamped to the i16 range first"
)]
fn to_pcm16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(samples: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_wav(&mut bytes, samples, 2, 48_000).expect("writes to memory");
        bytes
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 bytes"))
    }

    #[test]
    fn writes_a_valid_header() {
        let bytes = wav(&[0.0, 0.5, -0.5, 1.0]);
        assert_eq!(bytes.len(), 44 + 8);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u32_at(&bytes, 4), 36 + 8);
        assert_eq!(&bytes[8..16], b"WAVEfmt ");
        assert_eq!(u32_at(&bytes, 24), 48_000);
        assert_eq!(u32_at(&bytes, 28), 48_000 * 4);
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32_at(&bytes, 40), 8);
    }

    #[test]
    fn quantizes_and_clips_samples() {
        assert_eq!(to_pcm16(0.0), 0);
        assert_eq!(to_pcm16(1.0), i16::MAX);
        assert_eq!(to_pcm16(-1.0), -i16::MAX);
        assert_eq!(to_pcm16(2.0), i16::MAX);
        assert_eq!(to_pcm16(0.5), 16_384);
    }
}
