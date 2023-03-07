#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 o_color;
layout(location = 2) out vec3 vert_pos;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec3 o_color;
} uniforms;

void main() {
    o_color = uniforms.o_color;

    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;

    vec4 v = worldview * vec4(position, 1.0);
    vert_pos = vec3(v) / v.w;

    gl_Position = uniforms.proj * v;
}
