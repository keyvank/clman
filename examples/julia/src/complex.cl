float complex_abs2(float2 a) {
  return a.x * a.x + a.y * a.y;
}

float2 complex_mul(float2 a, float2 b) {
  return (float2)(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

float2 complex_sqr(float2 a) {
  return complex_mul(a, a);
}
