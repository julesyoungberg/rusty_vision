#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform GeneralUniforms {
    vec2 mouse;
    vec2 resolution;
    float time;
};

layout(set = 1, binding = 0) uniform sampler audio_sampler;
layout(set = 1, binding = 1) uniform texture2D mfccs;
layout(set = 1, binding = 2) uniform texture2D spectrum;
layout(set = 1, binding = 3) uniform AudioUniforms {
    float dissonance;
    float energy;
    float loudness;
    float noisiness;
    float onset;
    float pitch;
    float rms;
    float spectral_centroid;
    float spectral_complexity;
    float spectral_contrast;
    float tristimulus1;
    float tristimulus2;
    float tristimulus3;
};

#define NUM_MFCCS 12

//@import util/rand
//@import util/hsv2rgb

vec3 rand3(vec3 p);
vec3 hsv2rgb(vec3 c);

vec3 get_point(vec3 coord) {
    vec3 point = rand3(coord);
    point = sin(time * 0.5 + 6.2831 * point) * 0.5 + 0.5;
    return point;
}

vec4 voroni(vec3 p, float scale) {
    vec3 i_st = floor(p);
    vec3 f_st = fract(p);

    float m_dist = scale;
    vec3 m_point;
    vec3 m_coord;
    vec3 m_diff;

    // find the nearest cell center
    #pragma unroll
    for (int z = -1; z <= 1; z++) {
        #pragma unroll
        for (int y = -1; y <= 1; y++) {
            #pragma unroll
            for (int x = -1; x <= 1; x++) {
                vec3 neighbor = vec3(x, y, z);
                vec3 coord = i_st + neighbor;
                vec3 point = get_point(coord);

                vec3 diff = neighbor + point - f_st;
                float dist = length(diff);

                if (dist < m_dist) {
                    m_dist = dist;
                    m_point = point;
                    m_coord = coord;
                    m_diff = diff;
                }
            }
        }
    }

    float m_edge_dist = scale;

    // find the nearest edge
    #pragma unroll
    for (int z = -1; z <= 1; z++) {
        #pragma unroll
        for (int y = -1; y <= 1; y++) {
            #pragma unroll
            for (int x = -1; x <= 1; x++) {
                vec3 neighbor = vec3(x, y, z);
                vec3 coord = i_st + neighbor;
                if (all(equal(m_coord, coord))) {
                    continue;
                }

                vec3 point = get_point(coord);

                vec3 diff = neighbor + point - f_st;
                float dist = length(diff);

                vec3 to_center = (m_diff + diff) * 0.5;
                vec3 cell_diff = normalize(diff - m_diff);
                float edge_dist = dot(to_center, cell_diff);
                m_edge_dist = min(m_edge_dist, edge_dist);
            }
        }
    }

    return vec4(m_point, m_edge_dist);
}

void main() {
    vec2 st = uv;
    st.y *= resolution.y / resolution.x;
    st = st * 0.5 + 0.5;

    // Scale
    float scale = 20.0;
    st *= scale;

    vec3 p = vec3(st, time * 0.5);
    vec4 val = voroni(p, scale);
    vec3 m_point = val.xyz;
    float m_edge_dist = val.w;

    // map point to 1d value between 0 and 1
    float point_val = dot(m_point, m_point) * 0.5;
    float intensity = texture(sampler2D(spectrum, audio_sampler), vec2(point_val, 0)).x;

    vec3 color = hsv2rgb(vec3(point_val, 1, 1)).zxy * log(intensity * 10.0);
    color = mix(vec3(0), color, smoothstep(0.05, 0.06, m_edge_dist));

    // Draw cell center
    // color += 1.-step(.02, m_dist);

    // Draw grid
    // color.r += step(.98, f_st.x) + step(.98, f_st.y);

    frag_color = vec4(color, 1.0);
}
