#define JULIA_C (float2)(-0.8, 0.156)
#define SAMPLES (10)
#define LIMIT (10000)
#define HUE_JUMP (7)
#define SCALE (1)

float2 f(float2 x) {
  return complex_sqr(x) + JULIA_C;
}

__kernel void draw(__global float4 *buff, uint width, uint height) {
  uint id = get_global_id(0);
  int y = id / width;
  int x = id % width;
  uint max_wh = max(width, height);
  x -= width / 2;
  y -= height / 2;
  float3 col = 0;
  float samples = 0;

  for(int xx = -SAMPLES; xx <= SAMPLES; xx++) {
    for(int yy = -SAMPLES; yy <= SAMPLES; yy++) {
      samples += 1;
      float cnt = 0;
      float2 p = (float2)((x + xx / 2.0 / (float)SAMPLES),
                          (y + yy / 2.0 / (float)SAMPLES));
      p /= max_wh / 4 * SCALE;

      for(uint i = 0; i < LIMIT; i++) {
        if(complex_abs2(p) > 4)
            break;
        p = f(p);
        cnt += HUE_JUMP;
        if(cnt > HUE_MAX) cnt -= HUE_MAX;
      }

      col += hueToRgb(cnt);
    }
  }

  buff[id].xyz = col / samples;

}
