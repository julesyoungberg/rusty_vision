/*{
    "DESCRIPTION": "",
    "CREDIT": "by julesyoungberg",
    "ISFVSN": "2.0",
    "CATEGORIES": [ "GENERATOR" ],
    "INPUTS": [
        {
            "NAME": "fft_texture",
            "TYPE": "audioFFT"
        }
    ]
}*/

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float rand(float n) { return fract(n * 1183.5437 + .42); }

vec2 rand2(vec2 p) {
    return fract(
        sin(vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)))) *
        43758.5453);
}

vec2 get_point(vec2 coord) {
    vec2 point = rand2(coord);
    point = sin(6.2831 * point + TIME * 0.2) * 0.5 + 0.5;
    return point;
}

vec4 voronoi(vec2 p, float scale) {
    vec2 i_st = floor(p);
    vec2 f_st = fract(p);

    float m_dist = scale;
    vec2 m_point;
    vec2 m_coord;
    vec2 m_diff;

    // find the nearest cell center
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 neighbor = vec2(x, y);
            vec2 coord = i_st + neighbor;
            vec2 point = get_point(coord);

            vec2 diff = neighbor + point - f_st;
            float dist = length(diff);

            if (dist < m_dist) {
                m_dist = dist;
                m_point = point;
                m_coord = coord;
                m_diff = diff;
            }
        }
    }

    float m_edge_dist = scale;

    // find the nearest edge
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 neighbor = vec2(x, y);
            vec2 coord = i_st + neighbor;
            if (all(equal(m_coord, coord))) {
                continue;
            }

            vec2 point = get_point(coord);

            vec2 diff = neighbor + point - f_st;
            float dist = length(diff);

            vec2 to_center = (m_diff + diff) * 0.5;
            vec2 cell_diff = normalize(diff - m_diff);
            float edge_dist = dot(to_center, cell_diff);
            m_edge_dist = min(m_edge_dist, edge_dist);
        }
    }

    return vec4(m_coord + m_point, m_dist, m_edge_dist);
}

void main() {
    vec2 st = isf_FragNormCoord * 2.0 - 1.0;
    st.y *= RENDERSIZE.y / RENDERSIZE.x;

    vec3 color = vec3(0.0);

    float density = 1.0;
    float r = length(st);
    float a = atan(st.y, st.x);
    if (r < 0.2) {
        density = 0.3;
    } else if (r < 0.25) {
        density = mix(0.3, 0.9, smoothstep(0.2, 0.25, r));
    } else if (r < 0.35) {
        density = mix(0.9, 0.5, smoothstep(0.25, 0.35, r));
    } else {
        density = 0.5;
        st = mix(st, st * 0.5, smoothstep(0.35, 1.5, r));
    }

    // Scale
    float scale = 8.0 * density;
    st *= scale;

    vec4 val = voronoi(st, scale);
    vec2 point = val.xy;
    float dist = val.z;
    float edge_dist = val.w;

    vec2 gv = fract(st);
    vec2 id = floor(st);
    vec2 relative_point = point - id;
    vec2 cell_uv = gv - relative_point;

    cell_uv = mix(cell_uv, cell_uv * pow(sin(a * 8.0) * 0.5 + 0.5, 2.0) * 3.0,
                  smoothstep(0.35, 0.4, r));

    scale = 5.0 * density;
    cell_uv *= scale;
    vec2 inner_gv = fract(cell_uv);
    vec4 inner_val = voronoi(cell_uv, scale);
    vec2 inner_point = inner_val.xy;
    float inner_dist = inner_val.z;
    float inner_edge_dist = inner_val.w;

    float strength = log(IMG_NORM_PIXEL(fft_texture, vec2(0.1, 0)).x + 1.0);
    color += smoothstep(0.01, 0.02, edge_dist);
    float edge_scale = mix(0.5, 5.0, smoothstep(0.0, 0.5, strength));
    color -= smoothstep(0.02 * edge_scale, 0.01 * edge_scale, inner_edge_dist);

    // Draw cell center
    // color += 1.0 - step(0.02, dist);
    // color += 1.0 - step(0.02, inner_dist);

    // Draw grid
    // color.r += step(.48, gv.x) + step(.48, gv.y);
    // color.r += step(0.98, inner_gv.x) + step(0.98, inner_gv.y);

    gl_FragColor = vec4(color, 1.0);
}
