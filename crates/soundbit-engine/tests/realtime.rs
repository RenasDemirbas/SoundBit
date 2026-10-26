//! `Engine::process` must never allocate: allocation can block on a lock inside the allocator,
//! which causes audible dropouts on the audio thread.
//!
//! This test binary installs a counting global allocator. It is the one place outside `dsp` and
//! `audio-io` where `unsafe` is allowed, because `GlobalAlloc` is an unsafe trait.

#![allow(unsafe_code, reason = "implementing GlobalAlloc requires unsafe")]
#![allow(
    clippy::expect_used,
    reason = "tests panic on unexpected errors by design"
)]

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use common::{shaped_song, showcase_song};
use soundbit_engine::{Engine, Schedule};

thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

/// Forwards to the system allocator and counts allocations made on a tracking thread.
struct CountingAllocator;

fn record_allocation() {
    // `try_with` because the allocator may run while thread-locals are being torn down.
    let tracking = TRACKING.try_with(Cell::get).unwrap_or(false);
    if tracking {
        let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
    }
}

// SAFETY: every method forwards to `System` with the caller's arguments unchanged, so the
// `GlobalAlloc` contract is upheld by the system allocator.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // SAFETY: forwarded unchanged; the caller upholds `alloc`'s contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        // SAFETY: forwarded unchanged; the caller upholds `alloc_zeroed`'s contract.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation();
        // SAFETY: forwarded unchanged; the caller upholds `realloc`'s contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarded unchanged; the caller upholds `dealloc`'s contract.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[test]
fn process_never_allocates() {
    for song in [showcase_song(), shaped_song()] {
        let schedule = Schedule::compile(&song, 48_000).expect("valid rate");
        let mut engine = Engine::new(schedule);
        engine.enable_metering();
        let mut block = vec![0.0_f32; 2 * 256];

        let count = allocations_during(|| {
            while !engine.is_finished() {
                engine.process(&mut block);
            }
        });
        assert_eq!(
            count,
            0,
            "Engine::process allocated for {}",
            song.meta().title
        );
    }
}

#[test]
fn counter_detects_allocations() {
    TRACKING.with(|tracking| tracking.set(true));
    let before = ALLOCATIONS.with(Cell::get);
    let boxed = std::hint::black_box(Box::new(42_u64));
    let after = ALLOCATIONS.with(Cell::get);
    TRACKING.with(|tracking| tracking.set(false));
    drop(boxed);
    assert!(after > before, "the counting allocator is not active");
}

/// Runs `body` with allocation tracking on and returns how many allocations it made.
fn allocations_during(body: impl FnOnce()) -> usize {
    let before = ALLOCATIONS.with(Cell::get);
    TRACKING.with(|tracking| tracking.set(true));
    body();
    TRACKING.with(|tracking| tracking.set(false));
    ALLOCATIONS.with(Cell::get) - before
}

#[test]
fn live_player_never_allocates_on_the_audio_side() {
    let schedule = || Schedule::compile(&showcase_song(), 48_000).expect("valid rate");
    let (mut control, mut rt) = soundbit_engine::realtime::player();
    let mut block = vec![0.0_f32; 2 * 256];

    // Building engines and schedules happens on the control side and may allocate.
    control.start(schedule(), 0, true).expect("queue has room");
    let update = schedule();
    let count = allocations_during(|| {
        for _ in 0..50 {
            rt.process(&mut block);
        }
    });
    assert_eq!(count, 0, "starting playback allocated on the audio thread");

    control.update(update).expect("queue has room");
    control.stop().expect("queue has room");
    let count = allocations_during(|| {
        for _ in 0..50 {
            rt.process(&mut block);
        }
    });
    assert_eq!(
        count, 0,
        "updating or stopping allocated on the audio thread"
    );
    control.collect_garbage();
}
