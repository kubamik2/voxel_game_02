struct CameraUniform {
    view_projection: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
    @location(1) texture_index: u32,
    @location(2) @interpolate(perspective) light: f32
}

struct ModelQuad {
    vertex_positions: array<vec3<f32>, 4>,
    normal: vec3f,
    uv: array<vec2f, 4>
}

struct Face {
    lighting: array<u32, 4>,
    block_position: array<u32, 3>,
    texture_index: u32,
    quad_index: u32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<storage, read> face_buffer: array<vec2u>;
@group(2) @binding(0) var<storage, read> quad_buffer: array<ModelQuad>;
@group(4) @binding(0) var<uniform> translation: vec2i;

const TEXTURE_ATLAS_WIDTH: u32 = 256;
const TEXTURE_ATLAS_HEIGHT: u32 = 256;
fn construct_face(vertex_index: u32) -> Face {
    var face: Face;
    let face_raw_data = face_buffer[vertex_index >> 2u];
    var lighting: array<u32, 4>;

    lighting[0] = face_raw_data.x & 15u;
    lighting[1] = (face_raw_data.x >> 4u) & 15u;
    lighting[2] = (face_raw_data.x >> 8u) & 15u;
    lighting[3] = (face_raw_data.x >> 12u) & 15u;
    face.lighting = lighting;

    var block_position: array<u32, 3>;
    block_position[0] = (face_raw_data.x >> 16u) & 31u;
    block_position[1] = (face_raw_data.x >> (16u + 5u) & 31u);
    block_position[2] = (face_raw_data.x >> (16u + 10u) & 31u);
    face.block_position = block_position;

    face.texture_index = (face_raw_data.x >> (16u + 15u)) | ((face_raw_data.y & 32767u) << 1u);
    face.quad_index = ((face_raw_data.y >> 15u) & 65535u);
    
    return(face);
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var face = construct_face(i);
    let quad = quad_buffer[face.quad_index];
    
    let i_mod_4 = i % 4u;

    var vertex_positions = quad.vertex_positions;
    var vertex = vertex_positions[i_mod_4] + vec3f(f32(face.block_position[0]), f32(face.block_position[1]), f32(face.block_position[2]));

    vertex.x += f32(translation.x * 32);
    vertex.z += f32(translation.y * 32);
    vertex.y += f32(instance_index);
    var uvs = quad.uv;
    let uv = uvs[i_mod_4];

    var out: VertexOutput;

    out.clip_position = camera.view_projection * vec4f(vertex, 1.0);
    out.uv = uv;
    out.texture_index = face.texture_index;
    out.light = f32(face.lighting[i_mod_4] + 2) / 17.0;

    return out;
}

@group(3) @binding(0) var t_diffuse: texture_2d<f32>;
@group(3) @binding(1) var s_diffuse: sampler;
const BLOCK_SIZE: f32 = 1.0 / 16;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let texture_coords = vec2u(in.texture_index % 16u, in.texture_index / 16u);
    let base_uv = vec2f(f32(texture_coords.x) * BLOCK_SIZE, f32(texture_coords.y) * BLOCK_SIZE);
    let uv = base_uv + (in.uv * vec2f(BLOCK_SIZE, BLOCK_SIZE));
    let color = textureSample(t_diffuse, s_diffuse, uv) * vec4(in.light, in.light, in.light, 1.0);
    return color;
}