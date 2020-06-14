__kernel void fill(__global uint *buff, uint coeff) {
  buff[get_global_id(0)] = get_global_id(0) * coeff;
}

__kernel void sum(__global uint *buff, uint len) {
  uint acc = 0;
  for(uint i = 0; i < len; i++)
    acc += buff[i];
  printf("Hello World!\nSum is: %u\n", acc);
}

__kernel void draw(__global float4 *buff) {
  uint ind = get_global_id(0);
  buff[ind].x = 1.0;
  buff[ind].z = 1.0;
}
