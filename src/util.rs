use crate::*;

// blech
// Returns in the range [-1.0, 1.0]
// multiply/offset as needed
pub fn rand_uniform_f32(rng: &mut EntropyComponent<ChaCha8Rng>) -> f32 {
    ((rng.next_u64() & 0xffffffff) as f64 / (0x7fffffff as f64) - 1.0) as f32
}
