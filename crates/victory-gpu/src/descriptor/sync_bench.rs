//! Contention between the heap sync and title threads writing views
//! (#3425: "title threads waiting to write views", 74-105 ms a frame). One
//! thread plays the submitting thread: each frame it rewrites records in
//! `SYNC_BENCH_PAGES` (200) random 64 KiB pages of a placed, write-watched
//! view heap of `SYNC_BENCH_SLOTS` (2^20) slots, as the title copies
//! descriptors in as bytes (among the slots the host heap has, or all of
//! them with `SYNC_BENCH_PAST_CAP=1`), and runs one [`sync_heaps`]. Meanwhile
//! `SYNC_BENCH_THREADS` (5) title threads each write a view
//! (`CreateShaderResourceView` of a placed buffer, through [`record`] and
//! [`mirror`]) every `SYNC_BENCH_WORK_US` (20) microseconds of other work,
//! into pages of their own.
//!
//! Prints the sync time, and for the title threads the time per frame in
//! their view writes and how much of it is waiting (each call's time past
//! the median of a phase with no sync running), medians over
//! `SYNC_BENCH_FRAMES` (30) frames after three.
//!
//! Checks, the run fails otherwise: after each sync, every view a title
//! thread had written before it started (and not written over since) is the
//! slot's entry in the shader-visible view addresses; after the last frame,
//! with the title threads stopped and one more sync, every slot's entry is
//! the view its record names.
//!
//! Run with `cargo test -p victory-gpu --release sync_contention_bench --
//! --ignored --nocapture --test-threads=1`.

use super::*;
use crate::object::S_OK;
use std::sync::atomic::{
    AtomicBool, AtomicU64, AtomicUsize,
    Ordering::{Relaxed, SeqCst},
};
use std::time::{Duration, Instant};

fn env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn spin(d: Duration) {
    let until = Instant::now() + d;
    while Instant::now() < until {
        std::hint::spin_loop();
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// A placed buffer view's record (32 bytes, as the title's) of 16 4-byte
/// elements at `address`, written by the device method.
fn buffer_view(address: u64, at: u64) {
    let mut resource = [0u32; 14];
    (resource[0], resource[4]) = (1, 64);
    let mut view = [0u32; 16];
    (view[1], view[2], view[6], view[7]) = (
        1,
        windows::Win32::Graphics::Direct3D12::D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        16,
        4,
    );
    (view[10], view[11]) = (address as u32, (address >> 32) as u32);
    create_placed_srv(std::ptr::null(), &resource, &view, at);
}

#[repr(C)]
struct Extended(HeapDesc, Placement);

#[test]
#[ignore = "a benchmark: needs a D3D12 adapter and a quiet machine"]
fn sync_contention_bench() {
    if crate::host::host().is_none() {
        eprintln!("sync_contention_bench: no host D3D12 device");
        return;
    }
    let slots = env("SYNC_BENCH_SLOTS", 1 << 20);
    let pages = env("SYNC_BENCH_PAGES", 200);
    let changed = env("SYNC_BENCH_CHANGED", 50);
    let threads = env("SYNC_BENCH_THREADS", 5);
    let work = Duration::from_micros(env("SYNC_BENCH_WORK_US", 20) as u64);
    let frames = env("SYNC_BENCH_FRAMES", 30);
    let own_pages = env("SYNC_BENCH_THREAD_PAGES", 4);
    // Pages past a capped host heap are written (and synced) too, as the
    // title's are: SYNC_BENCH_PAST_CAP=1.
    let past_cap = env("SYNC_BENCH_PAST_CAP", 0) != 0;
    const WARM: usize = 3;
    const WINDOW: usize = 36 << 10;
    let page = crate::watch::PAGE;
    let per_page = page / 32;

    let memory = crate::watch::tests::pool(slots * 32 / page);
    // The buffer the views view: 1 MiB of title memory with a host copy.
    const BUFFER: usize = 16;
    let buffer = crate::watch::tests::pool(BUFFER);
    crate::shadow::register(buffer, buffer as u64, (BUFFER * page) as u64);
    let addresses = (BUFFER * page / 16 - 16) as u64;
    let address = |k: u64| buffer as u64 + 16 * (k % addresses);

    let ext = Extended(
        HeapDesc {
            kind: VIEWS_KIND,
            count: slots as u32,
            flags: SHADER_VISIBLE,
            nodes: 1,
        },
        Placement {
            address: memory as u64,
            size: (slots * 32) as u64,
        },
    );
    let mut h: This = std::ptr::null();
    assert_eq!(
        create_heap(std::ptr::null(), &ext.0, &HEAP_IIDS[0], &mut h),
        S_OK
    );
    let host = heap(h).host().expect("a host view heap");
    assert!(host.view_addresses().is_some(), "view addresses");
    let mirrored = (host.count() as usize).min(slots);
    let heap_pages = mirrored / per_page;
    assert!(
        heap_pages > threads * own_pages + 16,
        "a heap too small for the title threads' pages"
    );
    let shared_pages = heap_pages - threads * own_pages;
    // The pages the submitting thread rewrites: all but the title threads'.
    let choices = if past_cap {
        slots / per_page
    } else {
        heap_pages
    } - threads * own_pages;
    let shared = |r: usize| {
        if r < shared_pages {
            r
        } else {
            r + threads * own_pages
        }
    };

    // Records the submitting thread copies in: one in eight a null shader
    // resource view, one in eight a constant buffer view (no view address),
    // the rest buffer views at distinct addresses.
    let pool: Vec<([u8; 32], u64)> = (0..65_536u64)
        .map(|k| {
            let mut slot = [0u8; 32];
            let at = slot.as_mut_ptr() as u64;
            let entry = match k % 8 {
                0 => {
                    create_srv(std::ptr::null(), std::ptr::null(), std::ptr::null(), at);
                    0
                }
                4 => {
                    create_cbv(std::ptr::null(), &[(k as u32) << 8, 6, 256, 0], at);
                    0
                }
                _ => {
                    buffer_view(address(k * 7), at);
                    address(k * 7)
                }
            };
            (slot, entry)
        })
        .collect();
    // The view address each slot should end with.
    let mut expected = vec![0u64; mirrored];
    let put = |expected: &mut [u64], slot: usize, record: usize| {
        let (bytes, entry) = &pool[record % pool.len()];
        // SAFETY: `slot` is inside the heap.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), (memory + slot * 32) as *mut u8, 32)
        };
        if slot < expected.len() {
            expected[slot] = *entry;
        }
    };
    for slot in 0..slots {
        put(&mut expected, slot, slot.wrapping_mul(7));
    }
    let started = Instant::now();
    sync_heaps();
    let first_sync = started.elapsed();
    let wrong = (0..mirrored)
        .filter(|&s| host.address_at(s as u32) != Some(expected[s]))
        .count();
    assert_eq!(wrong, 0, "slots without their view after the first sync");

    // Title threads: thread `t` writes slot `own(t) + n % own_slots` with
    // its `n`th view, at `title_address(t, n)`.
    let own_slots = own_pages * per_page;
    let own = |t: usize| (shared_pages + t * own_pages) * per_page;
    let title_address = |t: usize, n: u64| address(n * 5 + t as u64 * 7919);
    let frame_now = AtomicUsize::new(usize::MAX);
    let stop = AtomicBool::new(false);
    let began: Vec<AtomicU64> = (0..threads).map(|_| AtomicU64::new(0)).collect();
    let done: Vec<AtomicU64> = (0..threads).map(|_| AtomicU64::new(0)).collect();

    let (mut sync_ms, mut dirty_ms) = (Vec::new(), Vec::new());
    let (mut lost, mut lost_shown) = (0usize, 0usize);
    let samples: Vec<Vec<(usize, u32)>> = std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let (frame_now, stop, began, done) = (&frame_now, &stop, &began[t], &done[t]);
                s.spawn(move || {
                    let mut samples = Vec::with_capacity(1 << 20);
                    let mut n = 0u64;
                    while !stop.load(Relaxed) {
                        let frame = frame_now.load(Relaxed);
                        let slot = own(t) + (n % own_slots as u64) as usize;
                        began.store(n + 1, SeqCst);
                        let at = Instant::now();
                        buffer_view(title_address(t, n), (memory + slot * 32) as u64);
                        let took = at.elapsed().as_nanos().min(u32::MAX as u128) as u32;
                        done.store(n + 1, SeqCst);
                        n += 1;
                        samples.push((frame, took));
                        spin(work);
                    }
                    samples
                })
            })
            .collect();
        // A stop that also holds when this thread panics (the workers spin).
        struct Stop<'a>(&'a AtomicBool);
        impl Drop for Stop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Relaxed);
            }
        }
        let stopping = Stop(&stop);

        // No sync running: the uncontended cost of a view write.
        std::thread::sleep(Duration::from_millis(300));
        let mut next = 0x9E37_79B9_7F4A_7C15u64;
        let mut rand = move || {
            next ^= next << 13;
            next ^= next >> 7;
            next ^= next << 17;
            next as usize
        };
        let mut checked = vec![0u64; threads];
        for frame in 0..frames + WARM {
            frame_now.store(frame, Relaxed);
            let at = Instant::now();
            let mut chosen: Vec<usize> = (0..pages).map(|_| shared(rand() % choices)).collect();
            chosen.sort_unstable();
            chosen.dedup();
            for &p in &chosen {
                let first = p * per_page;
                // The window's bytes again, as a title copying its table in.
                // SAFETY: inside the page, inside the heap.
                unsafe {
                    let at = (memory + first * 32) as *mut u8;
                    for k in (0..WINDOW).step_by(4096) {
                        at.add(k).write_volatile(at.add(k).read_volatile());
                    }
                }
                for _ in 0..changed {
                    put(&mut expected, first + rand() % (WINDOW / 32), rand());
                }
            }
            let dirtied = at.elapsed();
            let written: Vec<u64> = done.iter().map(|d| d.load(SeqCst)).collect();
            let at = Instant::now();
            sync_heaps();
            let synced = at.elapsed();
            // Every view written before the sync is there after it, unless
            // the thread has started writing that slot again since.
            for t in 0..threads {
                let upto = written[t];
                for n in checked[t].max(upto.saturating_sub(own_slots as u64))..upto {
                    let slot = own(t) + (n % own_slots as u64) as usize;
                    let entry = host.address_at(slot as u32);
                    if began[t].load(SeqCst) <= n + own_slots as u64
                        && entry != Some(title_address(t, n))
                    {
                        lost += 1;
                        if lost_shown < 5 {
                            lost_shown += 1;
                            eprintln!(
                                "frame {frame}: thread {t} view {n} at slot {slot} is {entry:#x?}, not {:#x}",
                                title_address(t, n)
                            );
                        }
                    }
                }
                checked[t] = upto;
            }
            if frame >= WARM {
                sync_ms.push(synced.as_secs_f64() * 1e3);
                dirty_ms.push(dirtied.as_secs_f64() * 1e3);
            }
        }
        frame_now.store(usize::MAX, Relaxed);
        drop(stopping);
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    });

    // Quiet now: one more sync, and every slot holds its last view.
    sync_heaps();
    for t in 0..threads {
        let n_end = done[t].load(SeqCst);
        for n in n_end.saturating_sub(own_slots as u64)..n_end {
            expected[own(t) + (n % own_slots as u64) as usize] = title_address(t, n);
        }
    }
    let stale: Vec<usize> = (0..mirrored)
        .filter(|&s| host.address_at(s as u32) != Some(expected[s]))
        .collect();

    // The title threads' calls: the uncontended median, then per frame.
    let mut base: Vec<f64> = samples
        .iter()
        .flatten()
        .filter(|&&(f, _)| f == usize::MAX)
        .map(|&(_, ns)| f64::from(ns))
        .collect();
    let base_ns = median(&mut base);
    let mut per_frame = vec![(0u64, 0f64, 0f64); frames];
    let mut all: Vec<f64> = Vec::new();
    for &(f, ns) in samples.iter().flatten() {
        if f == usize::MAX || f < WARM {
            continue;
        }
        let row = &mut per_frame[f - WARM];
        let ns = f64::from(ns);
        row.0 += 1;
        row.1 += ns / 1e6;
        row.2 += (ns - base_ns).max(0.0) / 1e6;
        all.push(ns);
    }
    let mut calls: Vec<f64> = per_frame.iter().map(|r| r.0 as f64).collect();
    let mut in_calls: Vec<f64> = per_frame.iter().map(|r| r.1).collect();
    let mut waits: Vec<f64> = per_frame.iter().map(|r| r.2).collect();
    all.sort_by(f64::total_cmp);
    let pct = |p: f64| {
        all.get(((all.len() as f64 * p) as usize).min(all.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0.0)
            / 1e3
    };
    let (sync_min, sync_max) = (
        sync_ms.iter().copied().fold(f64::MAX, f64::min),
        sync_ms.iter().copied().fold(0.0, f64::max),
    );
    println!(
        "sync_contention_bench: {mirrored} slots, {pages} pages x {changed} records, {threads} title threads every {} us; \
         median of {frames} frames: sync {:.2} ms (min {:.2}, max {:.2}), dirtying {:.2} ms; title threads per frame: \
         {:.0} view writes, {:.2} ms in them, {:.2} ms waiting (uncontended write {:.2} us); per write p50 {:.2} us, \
         p99 {:.1} us, max {:.1} ms; first sync {:.0} ms; lost after a sync {lost}, stale at the end {}",
        work.as_micros(),
        median(&mut sync_ms),
        sync_min,
        sync_max,
        median(&mut dirty_ms),
        median(&mut calls),
        median(&mut in_calls),
        median(&mut waits),
        base_ns / 1e3,
        pct(0.5),
        pct(0.99),
        all.last().copied().unwrap_or(0.0) / 1e6,
        first_sync.as_secs_f64() * 1e3,
        stale.len(),
    );
    for &s in stale.iter().take(5) {
        eprintln!(
            "stale: slot {s} is {:#x?}, not {:#x}",
            host.address_at(s as u32),
            expected[s]
        );
    }
    assert_eq!(lost, 0, "views written before a sync missing after it");
    assert!(stale.is_empty(), "slots without their last view");
    assert_eq!(release(h), 0);
    crate::shadow::unregister(buffer, buffer as u64);
    crate::watch::tests::free(buffer);
    crate::watch::tests::free(memory);
}

/// One heap shaped as the title's world heap for the slot-level benches
/// below: a placed, write-watched view heap of `slots` slots, every slot
/// holding a record from `pool` and made once, a 1 MiB buffer the views
/// view, and the records the benches copy in.
struct Shape {
    h: This,
    memory: usize,
    buffer: usize,
    pool: Vec<([u8; 32], u64)>,
    expected: Vec<u64>,
    mirrored: usize,
}

impl Shape {
    fn new(slots: usize) -> Shape {
        let page = crate::watch::PAGE;
        let memory = crate::watch::tests::pool(slots * 32 / page);
        const BUFFER: usize = 16;
        let buffer = crate::watch::tests::pool(BUFFER);
        crate::shadow::register(buffer, buffer as u64, (BUFFER * page) as u64);
        let addresses = (BUFFER * page / 16 - 16) as u64;
        let address = |k: u64| buffer as u64 + 16 * (k % addresses);
        let ext = Extended(
            HeapDesc {
                kind: VIEWS_KIND,
                count: slots as u32,
                flags: SHADER_VISIBLE,
                nodes: 1,
            },
            Placement {
                address: memory as u64,
                size: (slots * 32) as u64,
            },
        );
        let mut h: This = std::ptr::null();
        assert_eq!(create_heap(std::ptr::null(), &ext.0, &HEAP_IIDS[0], &mut h), S_OK);
        let host = heap(h).host().expect("a host view heap");
        assert!(host.view_addresses().is_some(), "view addresses");
        let mirrored = (host.count() as usize).min(slots);
        // One in eight a null shader resource view, one in eight a constant
        // buffer view, the rest buffer views at distinct addresses.
        let pool: Vec<([u8; 32], u64)> = (0..65_536u64)
            .map(|k| {
                let mut slot = [0u8; 32];
                let at = slot.as_mut_ptr() as u64;
                let entry = match k % 8 {
                    0 => {
                        create_srv(std::ptr::null(), std::ptr::null(), std::ptr::null(), at);
                        0
                    }
                    4 => {
                        create_cbv(std::ptr::null(), &[(k as u32) << 8, 6, 256, 0], at);
                        0
                    }
                    _ => {
                        buffer_view(address(k * 7), at);
                        address(k * 7)
                    }
                };
                (slot, entry)
            })
            .collect();
        let mut shape = Shape { h, memory, buffer, pool, expected: vec![0; mirrored], mirrored };
        for slot in 0..slots {
            shape.put(slot, slot.wrapping_mul(7));
        }
        sync_heaps();
        assert_eq!(shape.wrong(), 0, "slots without their view after the first sync");
        shape
    }

    fn host(&self) -> &host::HostHeap {
        heap(self.h).host().expect("a host view heap")
    }

    /// The record bytes in `slot`.
    fn bytes(&self, slot: usize) -> [u8; 32] {
        // SAFETY: `slot` is inside the heap.
        unsafe { ((self.memory + slot * 32) as *const [u8; 32]).read_unaligned() }
    }

    /// Copies pool record `record` into `slot` as the title does (bytes).
    fn put(&mut self, slot: usize, record: usize) {
        let (bytes, entry) = &self.pool[record % self.pool.len()];
        // SAFETY: `slot` is inside the heap.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), (self.memory + slot * 32) as *mut u8, 32) };
        if slot < self.expected.len() {
            self.expected[slot] = *entry;
        }
    }

    /// Slots whose view address is not their record's.
    fn wrong(&self) -> usize {
        let host = self.host();
        (0..self.mirrored)
            .filter(|&s| host.address_at(s as u32) != Some(self.expected[s]))
            .count()
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        assert_eq!(release(self.h), 0);
        crate::shadow::unregister(self.buffer, self.buffer as u64);
        crate::watch::tests::free(self.buffer);
        crate::watch::tests::free(self.memory);
    }
}

/// What [`with_title_threads`] measured: the title threads' waiting per
/// frame (each call's time past the median with no frame running), median
/// over the frames from `warm`; the uncontended call; and each written
/// slot's last view address.
struct Title {
    wait_ms: f64,
    base_us: f64,
    last: Vec<(usize, u64)>,
}

/// `threads` title threads writing a buffer view every `work` into slots
/// of their own (`own` each, from slot `first` of the heap at `memory`,
/// viewing the buffer at `buffer`) while `body` runs. `body` gets a frame
/// setter: `Some(frame)` while a frame's timed part runs, `None` otherwise.
#[allow(clippy::too_many_arguments)] // a bench's knobs
fn with_title_threads<R>(
    memory: usize,
    buffer: usize,
    threads: usize,
    first: usize,
    own: usize,
    work: Duration,
    warm: usize,
    body: impl FnOnce(&dyn Fn(Option<usize>)) -> R,
) -> (R, Title) {
    let buffer = buffer as u64;
    let title_address = move |t: usize, n: u64| buffer + 16 * ((n * 5 + t as u64 * 7919) % 60_000);
    let frame_now = AtomicUsize::new(usize::MAX);
    let stop = AtomicBool::new(false);
    let (r, samples, written) = std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let (frame_now, stop) = (&frame_now, &stop);
                s.spawn(move || {
                    let mut samples: Vec<(usize, u32)> = Vec::with_capacity(1 << 20);
                    let mut n = 0u64;
                    while !stop.load(Relaxed) {
                        let frame = frame_now.load(Relaxed);
                        let slot = first + t * own + (n % own as u64) as usize;
                        let at = Instant::now();
                        buffer_view(title_address(t, n), (memory + slot * 32) as u64);
                        samples.push((frame, at.elapsed().as_nanos().min(u32::MAX as u128) as u32));
                        n += 1;
                        spin(work);
                    }
                    (samples, n)
                })
            })
            .collect();
        struct Stop<'a>(&'a AtomicBool);
        impl Drop for Stop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Relaxed);
            }
        }
        let stopping = Stop(&stop);
        if threads > 0 {
            std::thread::sleep(Duration::from_millis(200));
        }
        let r = body(&|f: Option<usize>| frame_now.store(f.unwrap_or(usize::MAX), Relaxed));
        drop(stopping);
        let mut all = Vec::new();
        let mut written = Vec::new();
        for (t, w) in workers.into_iter().enumerate() {
            let (samples, n) = w.join().unwrap();
            all.extend(samples);
            written.push((t, n));
        }
        (r, all, written)
    });
    let mut base: Vec<f64> = samples.iter().filter(|s| s.0 == usize::MAX).map(|s| f64::from(s.1)).collect();
    let base_ns = median(&mut base);
    let frames = samples.iter().filter(|s| s.0 != usize::MAX).map(|s| s.0 + 1).max().unwrap_or(0);
    let mut waits = vec![0f64; frames];
    for &(f, ns) in &samples {
        if f != usize::MAX {
            waits[f] += (f64::from(ns) - base_ns).max(0.0) / 1e6;
        }
    }
    let mut waits: Vec<f64> = waits.into_iter().skip(warm).collect();
    let last = written
        .iter()
        .flat_map(|&(t, n)| {
            (n.saturating_sub(own as u64)..n)
                .map(move |k| (first + t * own + (k % own as u64) as usize, title_address(t, k)))
        })
        .collect();
    (r, Title { wait_ms: median(&mut waits), base_us: base_ns / 1e3, last })
}

fn xorshift(mut next: u64) -> impl FnMut() -> usize {
    move || {
        next ^= next << 13;
        next ^= next >> 7;
        next ^= next << 17;
        next as usize
    }
}

fn spread(v: &[f64]) -> (f64, f64) {
    (v.iter().copied().fold(f64::MAX, f64::min), v.iter().copied().fold(0.0, f64::max))
}

/// `HostHeap::sync_slots` in the shape #3425 reports (18.1 ms/frame for
/// ~384,000 slots on an RTX 5090): each frame `SYNC_SLOTS_SLOTS` (393,216 =
/// 192 pages) slots of a 2^20-slot view heap are in pages written since the
/// last sync, `SYNC_SLOTS_STALE` records among them changed, and one
/// `sync_heaps` runs (`SYNC_SLOTS_DIRECT=1`: `sync_slots` of each page
/// instead, without the write-watch walk). A row for each stale count in
/// `SYNC_SLOTS_STALE` (comma list, default 0,9600,96000), each without and
/// with `SYNC_BENCH_THREADS` (5) title threads writing views every 20 us.
/// Checks the changed slots after each sync and every slot at the end.
///
/// `cargo test -p victory-gpu --release sync_slots_bench -- --ignored
/// --nocapture --test-threads=1`
#[test]
#[ignore = "a benchmark: needs a D3D12 adapter and a quiet machine"]
fn sync_slots_bench() {
    if crate::host::host().is_none() {
        eprintln!("sync_slots_bench: no host D3D12 device");
        return;
    }
    let slots = env("SYNC_BENCH_SLOTS", 1 << 20);
    let dirty = env("SYNC_SLOTS_SLOTS", 393_216);
    let frames = env("SYNC_BENCH_FRAMES", 20);
    let threads = env("SYNC_BENCH_THREADS", 5);
    let work = Duration::from_micros(env("SYNC_BENCH_WORK_US", 20) as u64);
    let direct = env("SYNC_SLOTS_DIRECT", 0) != 0;
    let stales: Vec<usize> = std::env::var("SYNC_SLOTS_STALE")
        .unwrap_or_else(|_| "0,9600,96000".into())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    const WARM: usize = 3;
    const OWN_PAGES: usize = 4;
    let per_page = crate::watch::PAGE / 32;
    let mut shape = Shape::new(slots);
    let heap_pages = shape.mirrored / per_page;
    let shared_pages = heap_pages - threads * OWN_PAGES;
    let pages = (dirty / per_page).min(shared_pages);
    let first_own = shared_pages * per_page;
    let start = shape.memory as u64;
    let (memory, buffer) = (shape.memory, shape.buffer);
    let mut rand = xorshift(0x9E37_79B9_7F4A_7C15);
    let what = if direct { "sync_slots" } else { "sync_heaps" };
    let mut lost = 0usize;
    for &stale in &stales {
        for n_threads in [0, threads] {
            let ((mut sync_ms, mut seen), title) =
                with_title_threads(memory, buffer, n_threads, first_own, OWN_PAGES * per_page, work, WARM, |frame| {
                    let (mut sync_ms, mut seen) = (Vec::new(), Vec::new());
                    for f in 0..frames + WARM {
                        // Distinct random pages among the shared ones.
                        let mut chosen: Vec<usize> = Vec::with_capacity(pages);
                        let mut taken = vec![false; shared_pages];
                        while chosen.len() < pages {
                            let p = rand() % shared_pages;
                            if !std::mem::replace(&mut taken[p], true) {
                                chosen.push(p);
                            }
                        }
                        chosen.sort_unstable();
                        // Every chosen page written (the title copying its
                        // tables in), `stale` records among them changed.
                        for &p in &chosen {
                            // SAFETY: inside the heap.
                            unsafe {
                                let at = (memory + p * per_page * 32) as *mut u8;
                                for k in (0..crate::watch::PAGE).step_by(4096) {
                                    at.add(k).write_volatile(at.add(k).read_volatile());
                                }
                            }
                        }
                        let mut changed = Vec::with_capacity(stale);
                        for _ in 0..stale {
                            let slot = chosen[rand() % pages] * per_page + rand() % per_page;
                            let mut r = rand();
                            while shape.pool[r % shape.pool.len()].0 == shape.bytes(slot) {
                                r = rand();
                            }
                            shape.put(slot, r);
                            changed.push(slot);
                        }
                        frame(Some(f));
                        let at = Instant::now();
                        if direct {
                            let host = shape.host();
                            for &p in &chosen {
                                host.sync_slots((p * per_page) as u32, ((p + 1) * per_page) as u32, start, 32);
                            }
                        } else {
                            sync_heaps();
                        }
                        let took = at.elapsed();
                        frame(None);
                        changed.sort_unstable();
                        changed.dedup();
                        let host = shape.host();
                        lost += changed
                            .iter()
                            .filter(|&&s| host.address_at(s as u32) != Some(shape.expected[s]))
                            .count();
                        if f >= WARM {
                            sync_ms.push(took.as_secs_f64() * 1e3);
                            seen.push(changed.len() as f64);
                        }
                    }
                    (sync_ms, seen)
                });
            for &(s, a) in &title.last {
                shape.expected[s] = a;
            }
            let (min, max) = spread(&sync_ms);
            println!(
                "sync_slots_bench: {what}, {} slots/frame in {pages} pages, {:.0} changed, {n_threads} title threads: \
                 median {:.2} ms (min {min:.2}, max {max:.2}); title wait {:.2} ms/frame (uncontended write {:.2} us); lost {lost}",
                pages * per_page,
                median(&mut seen),
                median(&mut sync_ms),
                title.wait_ms,
                title.base_us,
            );
        }
    }
    sync_heaps();
    let stale_end = shape.wrong();
    println!("sync_slots_bench: lost after a sync {lost}, stale at the end {stale_end}");
    assert_eq!(lost, 0, "changed slots without their view after the sync");
    assert_eq!(stale_end, 0, "slots without their last view");
}

/// The title's descriptor copies (`CopyDescriptorsSimple` into its
/// shader-visible heap; borosf's "heap slots the title copied", 493,568
/// slots/frame on Arc, 36.6 ms): each frame `SYNC_COPY_SLOTS` (393,216)
/// slots copied in runs of `SYNC_COPY_RUN` (comma list, default 1,8,64,1024)
/// to random places of a 2^20-slot view heap, from random records
/// (`SYNC_COPY_SAME=1`: the records the slots hold already). Rows without
/// and with `SYNC_BENCH_THREADS` (5) title threads writing views. Checks
/// every slot after each row.
///
/// `cargo test -p victory-gpu --release copy_simple_bench -- --ignored
/// --nocapture --test-threads=1`
#[test]
#[ignore = "a benchmark: needs a D3D12 adapter and a quiet machine"]
fn copy_simple_bench() {
    if crate::host::host().is_none() {
        eprintln!("copy_simple_bench: no host D3D12 device");
        return;
    }
    let slots = env("SYNC_BENCH_SLOTS", 1 << 20);
    let total = env("SYNC_COPY_SLOTS", 393_216);
    let frames = env("SYNC_BENCH_FRAMES", 10);
    let threads = env("SYNC_BENCH_THREADS", 5);
    let work = Duration::from_micros(env("SYNC_BENCH_WORK_US", 20) as u64);
    let same = env("SYNC_COPY_SAME", 0) != 0;
    let runs: Vec<usize> = std::env::var("SYNC_COPY_RUN")
        .unwrap_or_else(|_| "1,8,64,1024".into())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    const WARM: usize = 2;
    const OWN_PAGES: usize = 4;
    let per_page = crate::watch::PAGE / 32;
    let mut shape = Shape::new(slots);
    let first_own = (shape.mirrored / per_page - threads * OWN_PAGES) * per_page;
    let (memory, buffer) = (shape.memory, shape.buffer);
    let mut rand = xorshift(0x2545_F491_4F6C_DD1D);
    for &run in &runs {
        for n_threads in [0, threads] {
            let mut source = vec![[0u8; 32]; total];
            let ((mut copy_ms, copied), title) =
                with_title_threads(memory, buffer, n_threads, first_own, OWN_PAGES * per_page, work, WARM, |frame| {
                    let mut copy_ms = Vec::new();
                    let mut placed: Vec<(usize, usize)> = Vec::with_capacity(total);
                    for f in 0..frames + WARM {
                        // Where each run goes (below the title threads'
                        // pages), and the records it carries.
                        placed.clear();
                        let mut k = 0;
                        while k < total {
                            let n = run.min(total - k);
                            let at = rand() % (first_own - n);
                            for j in 0..n {
                                if same {
                                    source[k + j] = shape.bytes(at + j);
                                    placed.push((at + j, usize::MAX));
                                } else {
                                    let record = rand();
                                    source[k + j] = shape.pool[record % shape.pool.len()].0;
                                    placed.push((at + j, record));
                                }
                            }
                            k += n;
                        }
                        frame(Some(f));
                        let started = Instant::now();
                        let mut k = 0;
                        while k < total {
                            let n = run.min(total - k);
                            copy_simple(
                                std::ptr::null(),
                                n as u32,
                                (memory + placed[k].0 * 32) as u64,
                                source[k..].as_ptr() as u64,
                                VIEWS_KIND,
                            );
                            k += n;
                        }
                        let took = started.elapsed();
                        frame(None);
                        if f >= WARM {
                            copy_ms.push(took.as_secs_f64() * 1e3);
                        }
                        for &(slot, record) in placed.iter().filter(|p| p.1 != usize::MAX) {
                            shape.expected[slot] = shape.pool[record % shape.pool.len()].1;
                        }
                    }
                    (copy_ms, placed.len())
                });
            for &(s, a) in &title.last {
                shape.expected[s] = a;
            }
            // The copies make the slots themselves: no sync before the check.
            let wrong = shape.wrong();
            let (min, max) = spread(&copy_ms);
            println!(
                "copy_simple_bench: {copied} slots/frame in runs of {run}{}, {n_threads} title threads: \
                 copies median {:.2} ms (min {min:.2}, max {max:.2}); title wait {:.2} ms/frame (uncontended write {:.2} us); wrong {wrong}",
                if same { " (same records)" } else { "" },
                median(&mut copy_ms),
                title.wait_ms,
                title.base_us,
            );
            assert_eq!(wrong, 0, "slots without the view copied there");
        }
    }
    sync_heaps();
}
