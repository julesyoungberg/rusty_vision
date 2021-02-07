#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform GeneralUniforms {
    vec2 resolution;
    float time;
};

layout(set = 1, binding = 0) uniform sampler audioSampler;
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
    float spectralCentroid;
    float spectralComplexity;
    float spectralContrast;
    float tristimulus1;
    float tristimulus2;
    float tristimulus3;
};

layout(set = 2, binding = 0) uniform NoiseUniforms {
    float lacunarity;
    float gain;
    int invert;
    int mirror;
    int octaves;
    int scaleByPrev;
    int sharpen;
    float speed;
};

//@import util/hsv2rgb

vec3 hsv2rgb(vec3 c);

//	Simplex 3D Noise 
//	by Ian McEwan, Ashima Arts
//
vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec4 taylorInvSqrt(vec4 r){return 1.79284291400159 - 0.85373472095314 * r;}

float snoise(vec3 v){ 
    const vec2  C = vec2(0.1666666667, 0.3333333333);
    const vec4  D = vec4(0.0, 0.5, 1.0, 2.0);

    // First corner
    vec3 i = floor(v + dot(v, C.yyy));
    vec3 x0 = v - i + dot(i, C.xxx);

    // Other corners
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min(g.xyz, l.zxy);
    vec3 i2 = max(g.xyz, l.zxy);

    //  x0 = x0 - 0. + 0.0 * C 
    vec3 x1 = x0 - i1 + 1.0 * C.xxx;
    vec3 x2 = x0 - i2 + 2.0 * C.xxx;
    vec3 x3 = x0 - 1. + 3.0 * C.xxx;

    // Permutations
    i = mod(i, 289.0);
    vec4 p = permute(
        permute(
            permute(i.z + vec4(0.0, i1.z, i2.z, 1.0))
            + i.y + vec4(0.0, i1.y, i2.y, 1.0)
        ) 
        + i.x + vec4(0.0, i1.x, i2.x, 1.0)
    );

    // Gradients
    // ( N*N points uniformly over a square, mapped onto an octahedron.)
    float n_ = 0.1428571429; // 1.0/7.0; // N=7
    vec3  ns = n_ * D.wyz - D.xzx;

    vec4 j = p - 49.0 * floor(p * ns.z *ns.z);  //  mod(p,N*N)

    vec4 x_ = floor(j * ns.z);
    vec4 y_ = floor(j - 7.0 * x_);    // mod(j,N)

    vec4 x = x_ * ns.x + ns.yyyy;
    vec4 y = y_ * ns.x + ns.yyyy;
    vec4 h = 1.0 - abs(x) - abs(y);

    vec4 b0 = vec4(x.xy, y.xy);
    vec4 b1 = vec4(x.zw, y.zw);

    vec4 s0 = floor(b0) * 2.0 + 1.0;
    vec4 s1 = floor(b1) * 2.0 + 1.0;
    vec4 sh = -step(h, vec4(0.0));

    vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
    vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww ;

    vec3 p0 = vec3(a0.xy,h.x);
    vec3 p1 = vec3(a0.zw,h.y);
    vec3 p2 = vec3(a1.xy,h.z);
    vec3 p3 = vec3(a1.zw,h.w);

    //Normalise gradients
    vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    // Mix final noise value
    vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
    m = m * m;
    return 42.0 * dot(m * m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

float getNoiseVal(vec3 p) {
    float raw = snoise(p);

    if (mirror == 1) {
        return abs(raw);
    }

    return raw * 0.5 + 0.5;
}

float fbm(vec2 p) {
    float sum = 0.0;
    float freq = 1.0;
    float amp = 0.5;
    float prev = 1.0;
    vec3 v = vec3(p, time * speed);

    #pragma unroll 1
    for (int i = 0; i < octaves; i++) {
        float n = getNoiseVal(v * freq);

        if (invert == 1) {
            n = 1.0 - n;
        }

        if (sharpen == 1) {
            n = n * n;
        }

        sum += n * amp;

        if (scaleByPrev == 1) {
            sum += n * amp * prev;
        }

        prev = n;
        freq *= lacunarity;
        amp *= gain;
    }

    return sum;
}

float pattern(in vec2 p, out vec2 q, out vec2 r) {
    const float scale1 = 2.0;
    const float scale2 = 2.0;
    const vec2 offsetA = vec2(0.0);
    const vec2 offsetB = vec2(0.0);
    const vec2 offsetC = vec2(0.0);
    const vec2 offsetD = vec2(0.0);

    p *= scale1;
    q = vec2(fbm(p + offsetA), fbm(p + offsetB));
    r = vec2(fbm(p + scale2 * q + offsetC), fbm(p + scale2 * q + offsetD));

    return fbm(p + scale2 * r);
}

void main() {
    const vec3 color1 = hsv2rgb(vec3(tristimulus1, 1, 1));
    const vec3 color2 = hsv2rgb(vec3(tristimulus2, 1, 1));
    const vec3 color3 = hsv2rgb(vec3(tristimulus3, 1, 1));
    const vec3 color4 = vec3(spectralCentroid * 0.00001, spectralContrast, spectralComplexity * 0.1);

    vec2 q;
    vec2 r;
    float f = pattern(uv, q, r);

    vec3 color = mix(
        mix(
            mix(color1, color2, f),
            color3,
            length(q) * 0.5
        ),
        color4,
        r.y * 0.5
    );

    frag_color = vec4(color, 1);
}
