#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 o_color;
layout(location = 2) in vec3 vert_pos;

layout(location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(2.0, -1.0, 1.0);
const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);

const int steps = 2;

const float kA = 0.4;
const float kD = 0.5;
const float kS = 0.05;

const float kRim = 0.716;
const float kRimThresh = 0.05;

void main() {
    vec3 N = normalize(v_normal);
    vec3 L = normalize(LIGHT-vert_pos);
    float nDotL =  dot(N, L);

    // Ambient
    vec3 out_color = kA * o_color;

    // Diffuse
    float diffuse = smoothstep(0.3, 0.32, nDotL);
    //float diffuse = nDotL;
    //diffuse = step(0.3, diffuse);
    //float diffuseToon = max(ceil(diffuse * float(steps)) / float(steps), 0.0);
    //diffuseToon = smoothstep(0.0, 0.01, diffuseToon);
    out_color += kD * o_color * LIGHT_COLOR * diffuse;

    // Specular
    vec3 R = reflect(-L, N);
    vec3 V = normalize(-vert_pos);
    float specAngle = max(dot(R, V), 0.0);
    float specPower = pow(specAngle, 80);
    float specSmooth = smoothstep(0.005, 0.01, specPower);
    out_color += kS * specSmooth;

    // Rimdot
    float rimDot = 1.0 - dot(V, N);
    float rimPower = rimDot * pow(nDotL, kRimThresh);
    rimPower = smoothstep(kRim-0.01, kRim+0.01, rimPower);
    out_color += vec3(0.2) * rimPower;

    f_color = vec4(out_color, 1.0);
}
