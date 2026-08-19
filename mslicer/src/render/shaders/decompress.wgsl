var<push_constant> runs: u32;
@group(0) @binding(0) var<storage, read> compressed: array<u32>;
@group(0) @binding(1) var<storage, read_write> uncompressed: array<atomic<u32>>;

@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {
    if (pos.x >= runs) { return; }

    let idx = pos.x * 3;
    let start = compressed[idx];
    let length = compressed[idx + 1];
    let value = compressed[idx + 2] & 0xFF;

    for (var i = 0u; i < length; i++) {
        let byte_idx = i + start;
        let array_idx = byte_idx / 4;
        let shift = (byte_idx % 4) * 8;

        atomicOr(&uncompressed[array_idx], value << shift);
    }
}
