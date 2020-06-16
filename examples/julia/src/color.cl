#define HUE_MAX (360.0)

float3 hueToRgb(float hue) {
  float3 rgb;

  if(hue >= HUE_MAX) hue = 0.0;
  hue /= 60.0;

  float ff = hue - (uint)hue;
  float t = ff;
  float q = 1.0 - ff;

  switch((uint)hue) {
  case 0:
    rgb.x = 1.0;
    rgb.y = t;
    rgb.z = 0.0;
    break;

  case 1:
    rgb.x = q;
    rgb.y = 1.0;
    rgb.z = 0.0;
    break;

  case 2:
    rgb.x = 0.0;
    rgb.y = 1.0;
    rgb.z = t;
    break;

  case 3:
    rgb.x = 0.0;
    rgb.y = q;
    rgb.z = 1.0;
    break;

  case 4:
    rgb.x = t;
    rgb.y = 0.0;
    rgb.z = 1.0;
    break;

  case 5:
  default:
    rgb.x = 1.0;
    rgb.y = 0.0;
    rgb.z = q;
    break;
  }

  return rgb;
}
