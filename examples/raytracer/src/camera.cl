typedef struct {
  float3 pos;
  float3 ul;
  float3 right;
  float3 down;
} camera;

camera camera_create(float3 const _pos, float3 const _up, float3 const _dir) {
  float3 cr = cross(_dir, _up);
  float3 ul = _pos + _dir + _up - cr;
  float3 down = -2 * _up;
  float3 right = 2 * cr;
  return (camera){_pos, ul, right, down};
}

ray get_ray(camera const * const _camera, float2 const _pos) {
  float3 target = _camera->ul + _camera->right * _pos.x + _camera->down * _pos.y;
  return (ray){_camera->pos, normalize(target - _camera->pos)};
}
