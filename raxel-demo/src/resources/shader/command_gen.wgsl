struct DrawIndirect {
    /// The number of vertices to draw.
    vertex_count: u32,
    /// The number of instances to draw.
    instance_count: u32,
    /// The Index of the first vertex to draw.
    base_vertex: u32,
    /// The instance ID of the first instance to draw.
    base_instance: u32,
}

@group(0)
@binding(0)
var<storage, read_write> count: atomic<u32>;

@group(0)
@binding(1)
var<storage, read_write> indirect_buffer: array<DrawIndirect>;

@group(0)
@binding(2)
var<storage, read> vertex_count_buffer: array<u32>;

@compute
@workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = (global_id.x << 6u) | (global_id.y << 3u) | (global_id.z << 0u);

    atomicAdd(&count, 1u);
    var command: DrawIndirect;
    command.vertex_count = vertex_count_buffer[idx];
    command.instance_count = 1u;
    command.base_vertex = (1179648u * idx);
    command.base_instance = idx;
    indirect_buffer[idx] = command;
}
