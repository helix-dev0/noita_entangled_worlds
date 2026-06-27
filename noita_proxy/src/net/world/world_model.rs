use std::num::NonZeroU16;
use std::sync::Arc;

use bitcode::{Decode, Encode};
use chunk::{Chunk, CompactPixel, Pixel, PixelFlags};
use encoding::{NoitaWorldUpdate, PixelRun, PixelRunner};
use rustc_hash::{FxHashMap, FxHashSet};
use tracing::info;

pub(crate) mod chunk;
pub mod encoding;

pub(crate) const CHUNK_SIZE: usize = 128;

#[derive(Debug, Encode, Decode, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChunkCoord(pub i32, pub i32);

#[derive(Default)]
pub(crate) struct WorldModel {
    chunks: FxHashMap<ChunkCoord, Chunk>,
    /// Tracks chunks which we written to.
    /// This includes any write, not just those that actually changed at least one pixel.
    updated_chunks: FxHashSet<ChunkCoord>,
}

/// Contains full info abount a chunk, RLE encoded.
/// Kinda close to ChunkDelta, but doesn't assume we know anything about the chunk.
#[derive(Debug, Encode, Decode, Clone)]
pub(crate) struct ChunkData {
    pub runs: Vec<PixelRun<CompactPixel>>,
}

/// Contains a diff, only pixels that were updated, for a given chunk.
#[derive(Debug, Encode, Decode, Clone)]
pub(crate) struct ChunkDelta {
    pub chunk_coord: ChunkCoord,
    runs: Arc<Vec<PixelRun<Option<CompactPixel>>>>,
}

impl ChunkData {
    pub(crate) fn make_random() -> Self {
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel(
                Pixel {
                    flags: PixelFlags::Normal,
                    material: (i as u16) % 512,
                }
                .to_compact(),
            )
        }
        let runs = runner.build();
        ChunkData { runs }
    }

    #[cfg(test)]
    pub(crate) fn new(mat: u16) -> Self {
        let mut runner = PixelRunner::new();
        for _ in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel(
                Pixel {
                    flags: PixelFlags::Normal,
                    material: mat,
                }
                .to_compact(),
            )
        }
        let runs = runner.build();
        ChunkData { runs }
    }

    pub(crate) fn apply_to_chunk(&self, chunk: &mut Chunk) {
        let nil = CompactPixel(NonZeroU16::new(4095).unwrap());
        let mut offset = 0;
        for run in &self.runs {
            let len = run.length as usize;
            if run.data != nil {
                chunk.fill_compact_pixels(offset, len, run.data);
            }
            offset += len;
        }
    }
    pub(crate) fn apply_delta(&mut self, delta: ChunkData) {
        let nil = CompactPixel(NonZeroU16::new(4095).unwrap());
        let mut chunk = Chunk::default();
        self.apply_to_chunk(&mut chunk);
        let mut offset = 0;
        for run in delta.runs.iter() {
            let len = run.length as usize;
            if run.data != nil {
                chunk.fill_compact_pixels(offset, len, run.data);
            }
            offset += len;
        }
        *self = chunk.to_chunk_data()
    }
}

/// Appends `count` copies of `value` to an in-progress run-length encoding,
/// merging with the current run exactly as `count` sequential
/// `PixelRunner::put_pixel(value)` calls would (identical run boundaries and
/// lengths). Lets `get_chunk_delta` bulk-emit whole 64-pixel words of unchanged
/// pixels without changing the encoded output.
#[inline]
fn push_run(
    runs: &mut Vec<PixelRun<Option<CompactPixel>>>,
    current: &mut Option<Option<CompactPixel>>,
    run_len: &mut u32,
    value: Option<CompactPixel>,
    count: u32,
) {
    match *current {
        Some(c) if c == value => *run_len += count,
        Some(c) => {
            runs.push(PixelRun {
                length: *run_len,
                data: c,
            });
            *current = Some(value);
            *run_len = count;
        }
        None => {
            *current = Some(value);
            *run_len = count;
        }
    }
}

impl WorldModel {
    fn get_chunk_coords(x: i32, y: i32) -> (ChunkCoord, usize) {
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_y = y.div_euclid(CHUNK_SIZE as i32);
        let x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let offset = x + y * CHUNK_SIZE;
        (ChunkCoord(chunk_x, chunk_y), offset)
    }

    /*fn set_pixel(&mut self, x: i32, y: i32, pixel: Pixel) {
        let (chunk_coord, offset) = Self::get_chunk_coords(x, y);
        let chunk = self.chunks.entry(chunk_coord).or_default();
        let current = chunk.pixel(offset);
        if current != pixel {
            chunk.set_pixel(offset, pixel);
        }
        self.updated_chunks.insert(chunk_coord);
    }*/

    fn get_pixel(&self, x: i32, y: i32) -> Pixel {
        let (chunk_coord, offset) = Self::get_chunk_coords(x, y);
        self.chunks
            .get(&chunk_coord)
            .map(|chunk| chunk.pixel(offset))
            .unwrap_or_default()
    }

    pub fn apply_noita_update(
        &mut self,
        update: &NoitaWorldUpdate,
        changed: &mut FxHashSet<ChunkCoord>,
    ) {
        fn set_pixel(pixel: Pixel, chunk: &mut Chunk, offset: usize) -> bool {
            let current = chunk.pixel(offset);
            if current != pixel {
                chunk.set_pixel(offset, pixel);
                true
            } else {
                false
            }
        }
        let header = &update.header;
        let runs = &update.runs;
        let mut x = 0;
        let mut y = 0;
        let (mut chunk_coord, _) = Self::get_chunk_coords(header.x, header.y);
        let mut chunk = self.chunks.entry(chunk_coord).or_default();
        for run in runs {
            let flags = if run.data.flags > 0 {
                PixelFlags::Fluid
            } else {
                PixelFlags::Normal
            };
            for _ in 0..run.length {
                let xs = header.x + x;
                let ys = header.y + y;
                let (new_chunk_coord, offset) = Self::get_chunk_coords(xs, ys);
                if chunk_coord != new_chunk_coord {
                    chunk_coord = new_chunk_coord;
                    chunk = self.chunks.entry(chunk_coord).or_default();
                }
                if set_pixel(
                    Pixel {
                        material: run.data.material,
                        flags,
                    },
                    chunk,
                    offset,
                ) {
                    self.updated_chunks.insert(chunk_coord);
                    if changed.contains(&chunk_coord) {
                        changed.remove(&chunk_coord);
                    }
                }
                x += 1;
                if x == i32::from(header.w) + 1 {
                    x = 0;
                    y += 1;
                }
            }
        }
    }

    pub fn get_noita_update(&self, x: i32, y: i32, w: u32, h: u32) -> NoitaWorldUpdate {
        assert!(w <= 256);
        assert!(h <= 256);
        let mut runner = PixelRunner::new();
        for j in 0..(h as i32) {
            for i in 0..(w as i32) {
                runner.put_pixel(self.get_pixel(x + i, y + j).to_raw())
            }
        }
        runner.into_noita_update(x, y, (w - 1) as u8, (h - 1) as u8)
    }

    pub fn get_all_noita_updates(&self) -> Vec<Vec<u8>> {
        let mut updates = Vec::new();
        for chunk_coord in &self.updated_chunks {
            let update = self.get_noita_update(
                chunk_coord.0 * (CHUNK_SIZE as i32),
                chunk_coord.1 * (CHUNK_SIZE as i32),
                CHUNK_SIZE as u32,
                CHUNK_SIZE as u32,
            );
            updates.push(update.save());
        }
        updates
    }

    pub(crate) fn apply_chunk_delta(&mut self, delta: &ChunkDelta) {
        self.updated_chunks.insert(delta.chunk_coord);
        let chunk = self.chunks.entry(delta.chunk_coord).or_default();
        let mut offset = 0;
        for run in delta.runs.iter() {
            let len = run.length as usize;
            if let Some(pixel) = run.data {
                chunk.fill_compact_pixels(offset, len, pixel);
            }
            offset += len;
        }
    }

    pub(crate) fn get_chunk_delta(
        &self,
        chunk_coord: ChunkCoord,
        ignore_changed: bool,
    ) -> Option<ChunkDelta> {
        let chunk = self.chunks.get(&chunk_coord)?;
        // Byte-identical to the previous full `PixelRunner` scan of every pixel:
        // the per-pixel value sequence is unchanged (`None` for an unchanged
        // pixel unless `ignore_changed`, otherwise `Some(compact_pixel)`), and
        // `push_run(v, n)` is exactly `n` `PixelRunner::put_pixel(v)` calls, so
        // the emitted runs match. The only difference is that a 64-pixel word
        // with no changed bit is emitted as one bulk `None` run instead of 64
        // individual pushes — skipping the dominant unchanged span. Proven by
        // `tests::get_chunk_delta_matches_reference`.
        const WORDS: usize = (CHUNK_SIZE * CHUNK_SIZE) / 64;
        let mut runs: Vec<PixelRun<Option<CompactPixel>>> = Vec::new();
        let mut current: Option<Option<CompactPixel>> = None;
        let mut run_len: u32 = 0;
        for w in 0..WORDS {
            let word = if ignore_changed {
                u64::MAX
            } else {
                chunk.changed_word(w)
            };
            if word == 0 {
                push_run(&mut runs, &mut current, &mut run_len, None, 64);
            } else {
                let base = w * 64;
                for b in 0usize..64 {
                    let value = if word & (1u64 << b) != 0 {
                        Some(chunk.compact_pixel(base + b))
                    } else {
                        None
                    };
                    push_run(&mut runs, &mut current, &mut run_len, value, 1);
                }
            }
        }
        if run_len > 0 {
            runs.push(PixelRun {
                length: run_len,
                data: current.expect("has current pixel"),
            });
        }
        Some(ChunkDelta {
            chunk_coord,
            runs: Arc::new(runs),
        })
    }

    pub fn updated_chunks(&self) -> &FxHashSet<ChunkCoord> {
        &self.updated_chunks
    }

    pub fn reset_change_tracking(&mut self) {
        for chunk_pos in &self.updated_chunks {
            if let Some(chunk) = self.chunks.get_mut(chunk_pos) {
                chunk.clear_changed();
            }
        }
        self.updated_chunks.clear();
    }

    pub fn reset(&mut self) {
        self.chunks.clear();
        self.updated_chunks.clear();
        info!("World model reset");
    }

    pub(crate) fn apply_chunk_data(&mut self, chunk: ChunkCoord, chunk_data: &ChunkData) {
        self.updated_chunks.insert(chunk);
        let chunk = self.chunks.entry(chunk).or_default();
        chunk_data.apply_to_chunk(chunk);
    }

    pub(crate) fn get_chunk_data(&self, chunk: ChunkCoord) -> Option<ChunkData> {
        let chunk = self.chunks.get(&chunk)?;
        Some(chunk.to_chunk_data())
    }

    pub(crate) fn forget_chunk(&mut self, chunk: ChunkCoord) {
        self.chunks.remove(&chunk);
        self.updated_chunks.remove(&chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny deterministic PRNG so fuzz inputs are reproducible across runs.
    struct Xorshift(u64);
    impl Xorshift {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    /// The original `PixelRunner`-based delta encoding, kept verbatim as an
    /// independent reference for the optimized `get_chunk_delta`.
    fn reference_runs(
        model: &WorldModel,
        coord: ChunkCoord,
        ignore_changed: bool,
    ) -> Vec<PixelRun<Option<CompactPixel>>> {
        let chunk = model.chunks.get(&coord).unwrap();
        let mut runner = PixelRunner::new();
        for i in 0..CHUNK_SIZE * CHUNK_SIZE {
            runner.put_pixel((ignore_changed || chunk.changed(i)).then(|| chunk.compact_pixel(i)));
        }
        runner.build()
    }

    // F07 byte-identity guard: the word-skipping `get_chunk_delta` must emit
    // exactly the same runs (hence the same wire bytes) as the old full scan,
    // for both `ignore_changed` values, over many random pixel/changed layouts
    // — including the unchanged-but-non-default case.
    #[test]
    fn get_chunk_delta_matches_reference() {
        let coord = ChunkCoord(3, -2);
        for seed in 1..=48u64 {
            let mut rng = Xorshift(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
            let mut model = WorldModel::default();
            {
                let chunk = model.chunks.entry(coord).or_default();
                for i in 0..CHUNK_SIZE * CHUNK_SIZE {
                    if rng.next_u64() & 1 == 0 {
                        let material = (rng.next_u64() % 512) as u16;
                        let flags = if rng.next_u64() & 1 == 0 {
                            PixelFlags::Normal
                        } else {
                            PixelFlags::Fluid
                        };
                        chunk.set_pixel(i, Pixel { flags, material });
                    }
                }
                // For some seeds, clear the changed bits then dirty a second
                // sparse subset, leaving non-default pixels that are NOT marked
                // changed (exercises the unchanged-but-non-default path).
                if seed.is_multiple_of(3) {
                    chunk.clear_changed();
                    for i in 0..CHUNK_SIZE * CHUNK_SIZE {
                        if rng.next_u64().is_multiple_of(5) {
                            let material = (rng.next_u64() % 512) as u16;
                            chunk.set_pixel(
                                i,
                                Pixel {
                                    flags: PixelFlags::Normal,
                                    material,
                                },
                            );
                        }
                    }
                }
            }
            for ignore_changed in [false, true] {
                let got = model.get_chunk_delta(coord, ignore_changed).unwrap();
                let want = reference_runs(&model, coord, ignore_changed);
                assert_eq!(
                    *got.runs, want,
                    "runs differ from reference (seed={seed}, ignore_changed={ignore_changed})"
                );
                let total: u32 = got.runs.iter().map(|r| r.length).sum();
                assert_eq!(
                    total as usize,
                    CHUNK_SIZE * CHUNK_SIZE,
                    "run lengths must cover the whole chunk"
                );
            }
        }
    }

    /// Sparse-delta encode benchmark — proves F06 (u64 changed-bitset) + F07
    /// (word-skipping `get_chunk_delta`) speed up the common "few pixels changed
    /// per frame" case versus the preserved full-pixel `PixelRunner` scan
    /// (`reference_runs`, the pre-optimization path). Measurement only.
    ///   cargo test --manifest-path noita_proxy/Cargo.toml --lib \
    ///       test_sparse_delta_perf -- --nocapture
    #[test]
    fn test_sparse_delta_perf() {
        use std::hint::black_box;
        use std::time::Instant;

        const TOTAL: usize = CHUNK_SIZE * CHUNK_SIZE; // 16384 px / 256 u64 words
        let coord = ChunkCoord(0, 0);

        // Write `n` changed pixels (Normal + a small non-default material always
        // differs from the 4095 default, so each set marks its changed bit),
        // placed every `stride` offsets. stride==1 = a contiguous local edit
        // (F07 best case: only ceil(n/64) words non-empty); stride==TOTAL/n =
        // spread across the chunk (the conservative case).
        fn build(coord: ChunkCoord, n: usize, stride: usize) -> WorldModel {
            let mut model = WorldModel::default();
            let chunk = model.chunks.entry(coord).or_default();
            for k in 0..n {
                let i = (k * stride) % TOTAL;
                chunk.set_pixel(
                    i,
                    Pixel {
                        flags: PixelFlags::Normal,
                        material: ((k % 511) + 1) as u16,
                    },
                );
            }
            model
        }
        // 64-px words F07 cannot skip = the work the sparse path actually does.
        fn nonempty_words(model: &WorldModel, coord: ChunkCoord) -> usize {
            let chunk = model.chunks.get(&coord).unwrap();
            (0..TOTAL / 64)
                .filter(|&w| chunk.changed_word(w) != 0)
                .count()
        }

        let iters: u128 = 4096;
        let bench = |model: &WorldModel| {
            let t = Instant::now();
            for _ in 0..iters {
                black_box(model.get_chunk_delta(coord, black_box(false)));
            }
            let sparse = t.elapsed().as_nanos() / iters;
            let t = Instant::now();
            for _ in 0..iters {
                black_box(reference_runs(model, coord, black_box(false)));
            }
            let reference = t.elapsed().as_nanos() / iters;
            (sparse, reference)
        };

        println!("sparse get_chunk_delta vs full-scan reference ({iters} iters, debug):");
        for pct in [1usize, 10, 100] {
            let n = TOTAL * pct / 100;
            let model = build(coord, n, 1); // contiguous = localized edit
            let (sparse, reference) = bench(&model);
            println!(
                "  {pct:>3}% ({n:>5} px, {:>3}/256 words): sparse {sparse:>7} ns | reference {reference:>7} ns | {:>5.1}x",
                nonempty_words(&model, coord),
                reference as f64 / sparse.max(1) as f64,
            );
        }

        // Conservative cross-check: 1% spread across the whole chunk.
        let n = TOTAL / 100;
        let model = build(coord, n, TOTAL / n.max(1));
        let (sparse, reference) = bench(&model);
        println!(
            "  1% spread ({:>3}/256 words): sparse {sparse:>7} ns | reference {reference:>7} ns | {:>5.1}x",
            nonempty_words(&model, coord),
            reference as f64 / sparse.max(1) as f64,
        );
    }
}
