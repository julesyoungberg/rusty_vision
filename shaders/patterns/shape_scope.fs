/*{
    "DESCRIPTION": "",
    "CREDIT": "by julesyoungberg",
    "ISFVSN": "2.0",
    "CATEGORIES": [ "GENERATOR" ],
    "INPUTS": []
}*/

// based on Square Kaleidoscope by domorin
// https://www.shadertoy.com/view/3t2XRG

#define PI 3.14159265359

float sdBox(in vec2 p, in vec2 b) {
    vec2 d = abs(p) - b;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float in_square(vec2 uv, float square_size) {
    // Changing this value causes some funky shit
    float a = TIME + length(uv) * PI * sin(TIME * 0.12316);
    float s = sin(a);
    float c = cos(a);
    uv *= mat2(c, -s, s, c);
    return float(uv.x > -square_size && uv.x < square_size &&
                 uv.y > -square_size && uv.y < square_size);
}

float square(in vec2 st, in float size) {
    float a = TIME + length(st / RENDERSIZE) * sin(TIME * 0.2) * PI * 0.5;
    float c = cos(a);
    float s = sin(a);
    mat2 rot = mat2(c, -s, s, c);
    st *= rot;
    return sdBox(st, vec2(size));
}

void main() {
    vec2 st = isf_FragNormCoord * 2.0 - 1.0;
    st.x *= RENDERSIZE.x / RENDERSIZE.y;
    st *= 6.0;

    float a = TIME + length(st) * sin(TIME * 0.5) * 0.2;
    float c = cos(a);
    float s = sin(a);
    mat2 rot = mat2(c, -s, s, c);
    st *= rot;

    vec2 f_st = fract(st);
    f_st -= 0.5;

    float size = 0.3 + sin(TIME * 0.5) * 0.1 * length(st);

    vec3 color = vec3(0.0);
    float dist = 0.0;

    float stp = 0.75;
    for (float y = -stp; y <= stp; y += stp) {
        for (float x = -stp; x <= stp; x += stp) {
            vec2 v = f_st + vec2(x, y);
            dist += square(v, size); // in_square(v, size);
        }
    }

    color = 0.5 +
            sin(TIME + vec3(dist * 0.25, dist * 2.354126, dist * 13.42)) * 0.5;

    gl_FragColor = vec4(color, 1.0);
}
