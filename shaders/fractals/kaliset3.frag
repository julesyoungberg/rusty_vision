#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform GeneralUniforms {
    vec2 mouse;
    vec2 resolution;
    float time;
    int mouse_down;
};

layout(set = 1, binding = 0) uniform sampler spectrum_sampler;
layout(set = 1, binding = 1) uniform texture2D spectrum;

const vec3 LIGHT_POS = vec3(0.0, 0.0, -1.0);

//@import util/complex_inv
//@import util/complex_mult
//@import util/palette

vec2 complex_inv(in vec2 z);
vec2 complex_mult(in vec2 a, in vec2 b);
vec3 palette(in float t, in vec3 a, in vec3 b, in vec3 c, in vec3 d);

// Based on Alien Tech by Kali
// https://www.shadertoy.com/view/XtX3zj
vec2 formula(in vec2 st) {
    vec2 z = st;

    vec2 c = vec2(-0.6);
    c = vec2(-0.32 + sin(time / 11.0) * 0.05, 0.87);

    float expsmo = 0.0;
    float len = 0.0;
    float orbit_trap = 0.0;

    float angle = time * 0.05;

    const float iterations = 40;
    for (float i = 0.0; i < iterations; i++) {
        // rotation
        //z *= mat2(cos(angle), -sin(angle), sin(angle), cos(angle));

        // original kali equation
        // z = abs(z) / dot(z, z) + c;

        // kali variations
        // z = abs(z) / (z.x * z.y) + c;
        // z = abs(complex_inv(z)) + c;
        // z = complex_mult(abs(z), complex_inv(abs(c))) + c;
        // z = abs(complex_mult(z, complex_inv(c))) + c;
        // z = abs(complex_mult(z, z)) + c;
        // z = complex_inv(complex_mult(complex_mult(z, z), z)) + c;

        // softology variations
        z.x = -abs(z.x);
        // z = complex_mult(z, c) + 1.0 + complex_inv(complex_mult(z, c) + 1.0);
        // z = abs(complex_mult(z, c) + 1.0) + complex_inv(abs(complex_mult(z, c) + 1.0));
        const vec2 cone = vec2(1.0, 0.0);
        vec2 temp = abs(complex_mult(z, c) + cone);
        z = temp + complex_mult(cone, complex_inv(temp));

        float mag = length(z);

        // exponential smoothing
        if (mod(i, 2.0) < 1.0) {
            float prev_len = len;
            len = mag;
            expsmo += exp(-1.0 / abs(len - prev_len));
            orbit_trap = min(orbit_trap, len);
        }
    }

    return vec2(expsmo, orbit_trap);
}

vec3 light(vec2 p, vec3 color) {
    // calculate normals based on horizontal and vertical vectors being z the formula result
    const vec2 d = vec2(0.0, 0.003);
    float d1 = formula(p - d.xy).x - formula(p + d.xy).x;
    float d2 = formula(p - d.yx).x - formula(p + d.yx).x;
    vec3 n1 = vec3(0.0, d.y * 2.0, -d1 * 0.05);
    vec3 n2 = vec3(d.y * 2.0, 0.0, -d2 * 0.05);
    vec3 n = normalize(cross(n1, n2));

    // lighting
    vec3 light_dir = normalize(vec3(p, 0.0) + LIGHT_POS);
    float diff = pow(max(0.0, dot(light_dir, n)), 2.0) + 0.2; // lambertian diffuse + ambient
	vec3 r = reflect(vec3(0.0, 0.0, 1.0), light_dir); // half vector
	float spec = pow(max(0.0, dot(r, n)), 30.0); // specular
  	return diff * color + spec * 0.1;
}

void main() {
    vec2 st = uv;
    float aspect_ratio = resolution.x / resolution.y;
    st.x *= aspect_ratio;
    st *= 2.0;
    
    vec3 color = vec3(0);

    float t = time;
    // float scale = 1.0 + 0.5 * sin(t / 17.0);
    // st *= scale;

    vec2 pix_size = 0.25 / resolution;
    pix_size.x *= aspect_ratio;
    const float aa_samples = 1.0;
    const float aa_sqrt = sqrt(aa_samples);
    float little_lights = 0.0;

    vec2 m = 2.0 * mouse / resolution.y;
    m = m * 0.5 + 0.5;

    for (float aa = 0.0; aa < aa_samples; aa++) {
        vec2 aa_coord = floor(vec2(aa / aa_sqrt, mod(aa, aa_sqrt)));
        vec2 p = st + aa_coord * pix_size;

        vec2 result = formula(p);
        float k = clamp(result.x * 0.06, 0.8, 1.4);
        vec3 col = palette(
            fract(result.x * 0.05), // last_stable / iterations,
            vec3(0.5, 0.5, 0.5), 
            vec3(0.5, 0.5, 0.5),
            vec3(1.0, 1.0, 1.0),
            vec3(0.0, 0.1, 0.2)
        );

        color += light(p, col);
        little_lights += max(0.0, 2.0 - result.y) / 2.0;
    }

    color /= aa_samples;

    // uv shift by light coords
    vec2 luv = st + LIGHT_POS.xy;

    // min amb light + spotlight with falloff * varying intensity
    color *= 0.07 + pow(max(0.0, 2.0 - length(luv) * 0.5), 2.0);

    // yellow lights
    // color += pow(little_lights * 0.12, 15.0) * vec3(1.0, 0.9, 0.3) * (0.8 + sin(time * 5.0 - st.y * 10.0) * 0.6);

    frag_color = vec4(color, 1);
}
