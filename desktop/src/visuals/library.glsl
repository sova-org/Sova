float _luminance(vec3 rgb) {
  const vec3 W = vec3(0.2125, 0.7154, 0.0721);
  return dot(rgb, W);
}

// Simplex 3D Noise — Ian McEwan, Ashima Arts
vec4 permute(vec4 x) {
  return mod(((x * 34.0) + 1.0) * x, 289.0);
}
vec4 taylorInvSqrt(vec4 r) {
  return 1.79284291400159 - 0.85373472095314 * r;
}

float _noise(vec3 v) {
  const vec2 C = vec2(1.0 / 6.0, 1.0 / 3.0);
  const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);
  vec3 i = floor(v + dot(v, C.yyy));
  vec3 x0 = v - i + dot(i, C.xxx);
  vec3 g = step(x0.yzx, x0.xyz);
  vec3 l = 1.0 - g;
  vec3 i1 = min(g.xyz, l.zxy);
  vec3 i2 = max(g.xyz, l.zxy);
  vec3 x1 = x0 - i1 + 1.0 * C.xxx;
  vec3 x2 = x0 - i2 + 2.0 * C.xxx;
  vec3 x3 = x0 - 1.0 + 3.0 * C.xxx;
  i = mod(i, 289.0);
  vec4 p = permute(permute(permute(
    i.z + vec4(0.0, i1.z, i2.z, 1.0))
    + i.y + vec4(0.0, i1.y, i2.y, 1.0))
    + i.x + vec4(0.0, i1.x, i2.x, 1.0));
  float n_ = 1.0 / 7.0;
  vec3 ns = n_ * D.wyz - D.xzx;
  vec4 j = p - 49.0 * floor(p * ns.z * ns.z);
  vec4 x_ = floor(j * ns.z);
  vec4 y_ = floor(j - 7.0 * x_);
  vec4 x = x_ * ns.x + ns.yyyy;
  vec4 y = y_ * ns.x + ns.yyyy;
  vec4 h = 1.0 - abs(x) - abs(y);
  vec4 b0 = vec4(x.xy, y.xy);
  vec4 b1 = vec4(x.zw, y.zw);
  vec4 s0 = floor(b0) * 2.0 + 1.0;
  vec4 s1 = floor(b1) * 2.0 + 1.0;
  vec4 sh = -step(h, vec4(0.0));
  vec4 a0 = b0.xzyw + s0.xzyw * sh.xxyy;
  vec4 a1 = b1.xzyw + s1.xzyw * sh.zzww;
  vec3 p0 = vec3(a0.xy, h.x);
  vec3 p1 = vec3(a0.zw, h.y);
  vec3 p2 = vec3(a1.xy, h.z);
  vec3 p3 = vec3(a1.zw, h.w);
  vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
  p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;
  vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
  m = m * m;
  return 42.0 * dot(m*m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

vec3 _rgbToHsv(vec3 c) {
  vec4 K = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
  vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
  vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
  float d = q.x - min(q.w, q.y);
  float e = 1.0e-10;
  return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 _hsvToRgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec4 osc(vec2 _st, float frequency, float sync, float offset) {
  float r = sin((_st.x - offset/frequency + iTime*sync) * frequency) * 0.5 + 0.5;
  float g = sin((_st.x + iTime*sync) * frequency) * 0.5 + 0.5;
  float b = sin((_st.x + offset/frequency + iTime*sync) * frequency) * 0.5 + 0.5;
  return vec4(r, g, b, 1.0);
}

vec4 noise(vec2 _st, float scale, float offset) {
  return vec4(vec3(_noise(vec3(_st * scale, offset * iTime))), 1.0);
}

vec4 voronoi(vec2 _st, float scale, float speed, float blending) {
  vec3 color = vec3(0.0);
  _st *= scale;
  vec2 i_st = floor(_st);
  vec2 f_st = fract(_st);
  float m_dist = 10.0;
  vec2 m_point;
  for (int j = -1; j <= 1; j++) {
    for (int i = -1; i <= 1; i++) {
      vec2 neighbor = vec2(float(i), float(j));
      vec2 p = i_st + neighbor;
      vec2 point = fract(sin(vec2(dot(p, vec2(127.1,311.7)), dot(p, vec2(269.5,183.3)))) * 43758.5453);
      point = 0.5 + 0.5 * sin(iTime * speed + 6.2831 * point);
      vec2 diff = neighbor + point - f_st;
      float dist = length(diff);
      if (dist < m_dist) {
        m_dist = dist;
        m_point = point;
      }
    }
  }
  color += dot(m_point, vec2(0.3, 0.6));
  color *= 1.0 - blending * m_dist;
  return vec4(color, 1.0);
}

vec4 shape(vec2 _st, float sides, float radius, float smoothing) {
  vec2 st = _st * 2.0 - 1.0;
  float a = atan(st.x, st.y) + 3.1416;
  float r = (2.0 * 3.1416) / sides;
  float d = cos(floor(0.5 + a/r) * r - a) * length(st);
  return vec4(vec3(1.0 - smoothstep(radius, radius + smoothing + 0.0000001, d)), 1.0);
}

vec4 gradient(vec2 _st, float speed) {
  return vec4(_st, sin(iTime * speed), 1.0);
}

vec4 solid(vec2 _st, float r, float g, float b, float a) {
  return vec4(r, g, b, a);
}

vec2 rotate(vec2 _st, float angle, float speed) {
  vec2 xy = _st - vec2(0.5);
  float ang = angle + speed * iTime;
  xy = mat2(cos(ang), -sin(ang), sin(ang), cos(ang)) * xy;
  xy += 0.5;
  return xy;
}

vec2 scale(vec2 _st, float amount, float xMult, float yMult, float offsetX, float offsetY) {
  vec2 xy = _st - vec2(offsetX, offsetY);
  xy *= (1.0 / vec2(amount * xMult, amount * yMult));
  xy += vec2(offsetX, offsetY);
  return xy;
}

vec4 color(vec4 _c0, float r, float g, float b, float a) {
  vec4 c = vec4(r, g, b, a);
  vec4 pos = step(0.0, c);
  return vec4(mix((1.0 - _c0) * abs(c), c * _c0, pos));
}

vec4 invert(vec4 _c0, float amount) {
  return vec4((1.0 - _c0.rgb) * amount + _c0.rgb * (1.0 - amount), _c0.a);
}

vec4 contrast(vec4 _c0, float amount) {
  vec4 c = (_c0 - vec4(0.5)) * vec4(amount) + vec4(0.5);
  return vec4(c.rgb, _c0.a);
}

vec4 brightness(vec4 _c0, float amount) {
  return vec4(_c0.rgb + vec3(amount), _c0.a);
}

vec4 saturate(vec4 _c0, float amount) {
  const vec3 W = vec3(0.2125, 0.7154, 0.0721);
  vec3 intensity = vec3(dot(_c0.rgb, W));
  return vec4(mix(intensity, _c0.rgb, amount), _c0.a);
}

vec4 hue(vec4 _c0, float hue) {
  vec3 c = _rgbToHsv(_c0.rgb);
  c.r += hue;
  return vec4(_hsvToRgb(c), _c0.a);
}

vec4 posterize(vec4 _c0, float bins, float gamma) {
  vec4 c2 = pow(_c0, vec4(gamma));
  c2 *= vec4(bins);
  c2 = floor(c2);
  c2 /= vec4(bins);
  c2 = pow(c2, vec4(1.0/gamma));
  return vec4(c2.xyz, _c0.a);
}

vec4 luma(vec4 _c0, float threshold, float tolerance) {
  float a = smoothstep(threshold-(tolerance+0.0000001), threshold+(tolerance+0.0000001), _luminance(_c0.rgb));
  return vec4(_c0.rgb * a, a);
}

vec4 colorama(vec4 _c0, float amount) {
  vec3 c = _rgbToHsv(_c0.rgb);
  c += vec3(amount);
  c = _hsvToRgb(c);
  c = fract(c);
  return vec4(c, _c0.a);
}

vec2 scroll(vec2 _st, float scrollX, float scrollY, float speedX, float speedY) {
  _st.x += scrollX + iTime * speedX;
  _st.y += scrollY + iTime * speedY;
  return fract(_st);
}

vec2 kaleid(vec2 _st, float nSides) {
  vec2 st = _st - 0.5;
  float r = length(st);
  float a = atan(st.y, st.x);
  float pi = 2.0 * 3.1416;
  a = mod(a, pi / nSides);
  a = abs(a - pi / nSides / 2.0);
  return r * vec2(cos(a), sin(a));
}

vec2 pixelate(vec2 _st, float pixelX, float pixelY) {
  vec2 xy = vec2(pixelX, pixelY);
  return (floor(_st * xy) + 0.5) / xy;
}

vec2 repeat(vec2 _st, float repeatX, float repeatY, float offsetX, float offsetY) {
  vec2 st = _st * vec2(repeatX, repeatY);
  st.x += step(1.0, mod(st.y, 2.0)) * offsetX;
  st.y += step(1.0, mod(st.x, 2.0)) * offsetY;
  return fract(st);
}

vec2 modulate(vec2 _st, vec4 _c0, float amount) {
  return _st + _c0.xy * amount;
}

vec2 modulateScale(vec2 _st, vec4 _c0, float multiple, float offset) {
  vec2 xy = _st - vec2(0.5);
  xy *= (1.0 / vec2(offset + multiple * _c0.r, offset + multiple * _c0.g));
  xy += vec2(0.5);
  return xy;
}

vec2 modulateRotate(vec2 _st, vec4 _c0, float multiple, float offset) {
  vec2 xy = _st - vec2(0.5);
  float angle = offset + _c0.x * multiple;
  xy = mat2(cos(angle), -sin(angle), sin(angle), cos(angle)) * xy;
  xy += 0.5;
  return xy;
}

vec4 add(vec4 _c0, vec4 _c1, float amount) {
  return (_c0 + _c1) * amount + _c0 * (1.0 - amount);
}

vec4 mult(vec4 _c0, vec4 _c1, float amount) {
  return _c0 * (1.0 - amount) + (_c0 * _c1) * amount;
}

vec4 blend(vec4 _c0, vec4 _c1, float amount) {
  return _c0 * (1.0 - amount) + _c1 * amount;
}

vec4 diff(vec4 _c0, vec4 _c1) {
  return vec4(abs(_c0.rgb - _c1.rgb), max(_c0.a, _c1.a));
}

vec4 layer(vec4 _c0, vec4 _c1) {
  return vec4(mix(_c0.rgb, _c1.rgb, _c1.a), clamp(_c0.a + _c1.a, 0.0, 1.0));
}

vec4 mask(vec4 _c0, vec4 _c1) {
  float a = _luminance(_c1.rgb);
  return vec4(_c0.rgb * a, a * _c0.a);
}

vec2 scrollX(vec2 _st, float scrollX, float speed) {
  _st.x += scrollX + iTime * speed;
  return fract(_st);
}

vec2 scrollY(vec2 _st, float scrollY, float speed) {
  _st.y += scrollY + iTime * speed;
  return fract(_st);
}

vec2 repeatX(vec2 _st, float reps, float offset) {
  vec2 st = _st * vec2(reps, 1.0);
  st.y += step(1.0, mod(st.x, 2.0)) * offset;
  return fract(st);
}

vec2 repeatY(vec2 _st, float reps, float offset) {
  vec2 st = _st * vec2(1.0, reps);
  st.x += step(1.0, mod(st.y, 2.0)) * offset;
  return fract(st);
}

vec4 shift(vec4 _c0, float r, float g, float b, float a) {
  return vec4(fract(_c0.r + r), fract(_c0.g + g), fract(_c0.b + b), clamp(_c0.a + a, 0.0, 1.0));
}

vec4 thresh(vec4 _c0, float threshold, float tolerance) {
  float a = smoothstep(threshold - tolerance, threshold + tolerance, _luminance(_c0.rgb));
  return vec4(vec3(a), _c0.a);
}

vec4 r(vec4 _c0, float scale, float offset) {
  return vec4(_c0.r * scale + offset, _c0.g, _c0.b, _c0.a);
}

vec4 g(vec4 _c0, float scale, float offset) {
  return vec4(_c0.r, _c0.g * scale + offset, _c0.b, _c0.a);
}

vec4 b(vec4 _c0, float scale, float offset) {
  return vec4(_c0.r, _c0.g, _c0.b * scale + offset, _c0.a);
}

vec4 sub(vec4 _c0, vec4 _c1, float amount) {
  return (_c0 - _c1) * amount + _c0 * (1.0 - amount);
}

vec2 modulateRepeat(vec2 _st, vec4 _c0, float repeatX, float repeatY, float offsetX, float offsetY) {
  vec2 st = _st * vec2(repeatX, repeatY);
  st.x += _c0.r * offsetX;
  st.y += _c0.g * offsetY;
  return fract(st);
}

vec2 modulateRepeatX(vec2 _st, vec4 _c0, float reps, float offset) {
  vec2 st = _st * vec2(reps, 1.0);
  st.y += _c0.r * offset;
  return fract(st);
}

vec2 modulateRepeatY(vec2 _st, vec4 _c0, float reps, float offset) {
  vec2 st = _st * vec2(1.0, reps);
  st.x += _c0.r * offset;
  return fract(st);
}

vec2 modulateKaleid(vec2 _st, vec4 _c0, float nSides) {
  vec2 st = _st - 0.5;
  float r = length(st);
  float a = atan(st.y, st.x);
  float pi = 2.0 * 3.1416;
  float sides = nSides + _c0.r * nSides;
  a = mod(a, pi / sides);
  a = abs(a - pi / sides / 2.0);
  return r * vec2(cos(a), sin(a));
}

vec2 modulateScrollX(vec2 _st, vec4 _c0, float scrollX, float speed) {
  _st.x += _c0.r * scrollX + iTime * speed;
  return fract(_st);
}

vec2 modulateScrollY(vec2 _st, vec4 _c0, float scrollY, float speed) {
  _st.y += _c0.r * scrollY + iTime * speed;
  return fract(_st);
}

vec2 modulatePixelate(vec2 _st, vec4 _c0, float multiple, float offset) {
  vec2 xy = vec2(_c0.r * multiple + offset, _c0.g * multiple + offset);
  return (floor(_st * xy) + 0.5) / xy;
}

vec2 modulateHue(vec2 _st, vec4 _c0, float amount) {
  return _st + (_c0.gb - 0.5) * amount;
}
