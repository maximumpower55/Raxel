struct CameraUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(1)
@binding(0)
var<uniform> camera_uniforms: CameraUniforms;

@group(0)
@binding(0)
var<storage, read> faces: array<u32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) tex_id: i32,
    @location(2) light: f32,
}

@vertex
fn vert(@builtin(instance_index) instance_index: u32, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var vertex_pos_lookup: array<array<vec3<f32>, 6>, 6> = array<array<vec3<f32>, 6>, 6>(
        array<vec3<f32>, 6>( vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0) ), // North
        array<vec3<f32>, 6>( vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 0.0) ), // West
        array<vec3<f32>, 6>( vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, 0.0) ), // Down
        array<vec3<f32>, 6>( vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 1.0) ), // South
        array<vec3<f32>, 6>( vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0) ), // East
        array<vec3<f32>, 6>( vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 0.0) ), // Up
    );
    var texture_coords: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0), vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    var normal_light_lookup: array<f32, 6> = array<f32, 6>(
        0.6,
        0.8,
        0.4,
        0.6,
        0.8,
        1.0,
    );

                //     mesh.push(
                //     (((idx >> 10 & 31) as u32) << 0)
                //         | (((idx >> 5 & 31) as u32) << 5)
                //         | (((idx >> 0 & 31) as u32) << 10)
                //         | ((face.norm as u32) << 15)
                //         | ((face.tex_id as u32) << 18)
                //         | (31 << 25)
                //         | (31 << 30)
                // );

    let face = faces[vertex_index / 6u];
    let corner_index = vertex_index % 6u;

    let face_normal = (face >> 15u) & 7u;
    let face_width = f32((face >> 21u) & 31u) + 1.0;
    let face_height = f32((face >> 26u) & 31u) + 1.0;

    var transformed_pos = vec3<f32>(f32((face >> 10u) & 31u), f32((face >> 5u) & 31u), f32((face >> 0u) & 31u));
    transformed_pos += vertex_pos_lookup[face_normal][corner_index] * vec3<f32>(face_width, face_width, face_height);

    var out: VertexOutput;
    out.position = (camera_uniforms.projection * camera_uniforms.view) * vec4<f32>(transformed_pos + vec3<f32>(f32((instance_index >> 6u) & 7u), f32((instance_index >> 3u) & 7u), f32((instance_index >> 0u) & 7u)) * 32.0, 1.0);
    out.tex_coord = texture_coords[corner_index] * vec2<f32>(face_width, face_height);
    out.tex_id = i32((face >> 18u) & 7u);
    out.light = normal_light_lookup[face_normal];
    return out;
}

@group(2)
@binding(0)
var block_tex_array: texture_2d_array<f32>;
@group(2)
@binding(1)
var block_tex_sampler: sampler;

@fragment
fn frag(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(block_tex_array, block_tex_sampler, vertex.tex_coord, vertex.tex_id) * vertex.light;
}
