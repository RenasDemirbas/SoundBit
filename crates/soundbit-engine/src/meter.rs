//! Level metering per track, for mix reports after offline rendering.

/// Level of one track's contribution to the output, before the master soft clipper.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TrackPower {
    /// Largest absolute sample value.
    pub peak: f32,
    /// Mean of the squared samples over both channels; its square root is the RMS level.
    pub mean_square: f64,
}

/// Running sums for every track slot. Preallocated, so recording never allocates.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Meters {
    peaks: Vec<f32>,
    sums: Vec<f64>,
    samples: u64,
}

impl Meters {
    pub(crate) fn new(slots: usize) -> Self {
        Self {
            peaks: vec![0.0; slots],
            sums: vec![0.0; slots],
            samples: 0,
        }
    }

    // RT-SAFE
    /// Adds one stereo frame of track `index`. Track 0 counts the frames.
    pub(crate) fn record(&mut self, index: usize, left: f32, right: f32) {
        if index == 0 {
            self.samples += 2;
        }
        if let (Some(peak), Some(sum)) = (self.peaks.get_mut(index), self.sums.get_mut(index)) {
            *peak = peak.max(left.abs()).max(right.abs());
            *sum += f64::from(left) * f64::from(left) + f64::from(right) * f64::from(right);
        }
    }

    /// The levels of the first `tracks` tracks.
    pub(crate) fn power(&self, tracks: usize) -> Vec<TrackPower> {
        let samples = self.samples.max(1) as f64;
        self.peaks
            .iter()
            .zip(&self.sums)
            .take(tracks)
            .map(|(&peak, &sum)| TrackPower {
                peak,
                mean_square: sum / samples,
            })
            .collect()
    }
}
