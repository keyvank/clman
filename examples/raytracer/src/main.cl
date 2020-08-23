#define SPHERE_COUNT (4)
#define LIGHT_COUNT (2)

typedef struct {
  float3 pos;
  float3 emit;
} light;

typedef struct {
  bool intersected;
  float3 pos;
  float3 col;
  ray next;
} intersection;

typedef struct {
  float3 pos;
  float r;
  float3 col;
  float checker;
} sphere;

float intersect(ray _ray, sphere _sphere) {
  float tca = dot(_sphere.pos-_ray.pos,_ray.dir);
  if(tca > 0) {
    float d2 = dot(_sphere.pos - _ray.pos, _sphere.pos - _ray.pos) - tca * tca;
    if(d2 <= _sphere.r * _sphere.r) {
      float thc = sqrt(_sphere.r * _sphere.r - d2);
      float dist = min(tca - thc, tca + thc);
      if(dist > 0) {
        return dist;
      }
    }
  }
  return 0.0;
}

intersection cast_ray(ray r) {
  sphere spheres[4];
  spheres[0].pos = (float3)(0.0f, 20.0f, 70.0f);
  spheres[0].r = 20.0f;
  spheres[0].col = RED;
  spheres[0].checker = 0.0f;
  spheres[1].pos = (float3)(-15.0f, 8.0f, 40.0f);
  spheres[1].r = 8.0f;
  spheres[1].col = GREEN;
  spheres[1].checker = 0.0f;
  spheres[2].pos = (float3)(15.0f, 12.0f, 40.0f);
  spheres[2].r = 12.0f;
  spheres[2].col = BLUE;
  spheres[2].checker = 0.0f;
  spheres[3].pos = (float3)(0.0f, -1000000.0f, 0.0f);
  spheres[3].r = 1000000.0f;
  spheres[3].col = WHITE;
  spheres[3].checker = 20.0f;

  int ind = -1;
  float dist = 0.0f;
  for(uint i = 0; i < SPHERE_COUNT; i++) {
    float isect = intersect(r, spheres[i]);
    if(isect && (isect < dist || ind < 0)) {
      ind = i;
      dist = isect;
    }
  }

  if(ind >= 0) {
    sphere s = spheres[ind];
    float3 pos = r.pos + r.dir * dist;
    intersection isect;
    isect.pos = pos;
    isect.intersected = true;
    isect.col = s.col;
    isect.next = (ray){pos, normalize(pos - s.pos)};
    if(s.checker > 0.0f) {
      int ind = floor(pos.x / s.checker) + floor(pos.z / s.checker);
      if(ind % 2 == 0) {
        isect.col = BLACK;
      }
    }
    return isect;
  }

  intersection isect;
  isect.intersected = false;
  return isect;
}

float3 trace_ray(ray r) {
  light lights[2];
  lights[0].pos = (float3)(100.0f, 100.0f, 100.0f);
  lights[0].emit = WHITE;
  lights[1].pos = (float3)(-100.0f, 100.0f, -100.0f);
  lights[1].emit = WHITE*0.5f;

  uint count = 0;
  intersection isects[10];
  isects[0] = cast_ray(r);
  for(uint i = 1; i < 10; i++) {
    if(isects[i - 1].intersected) {
      count++;
      isects[i] = cast_ray(isects[i - 1].next);
    }
    else
      break;
  }
  float3 ret = BLACK;
  if(count > 0) {
    for(int i = count - 1; i >= 0; i--) {
      ret *= 0.1f;
      float l = 0.0f;
      for(int j =0;j < LIGHT_COUNT; j++) {
        float3 ldir = normalize(lights[j].pos - isects[i].pos);
        ray tolight = (ray){isects[i].pos+ldir*0.001f, ldir};
        l += cast_ray(tolight).intersected ? 0.0f : fmax(dot(tolight.dir, isects[i].next.dir), 0.0f);
      }
      l += 0.1f;
      ret += isects[i].col * l;
    }
  }
  return ret;
}

__kernel void render(uint width, uint height, __global float4 *img) {
  int index = get_global_id(0);
  int x = index % width;
  int y = index / width;
  float2 pos = (float2)((float)x / width, (float)y / height);
  camera cam = camera_create((float3)(0.0f, 50.0f, -50.0f), UP, normalize((float3)(0.0f, -0.2f, 1.0f)));
  img[y * width + x].xyz = trace_ray(get_ray(&cam, pos));
}
