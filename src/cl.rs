use crate::conf::{Arg, BufferType, Computable, Environment};
use rust_gpu_tools::opencl as cl;
use std::collections::HashMap;

struct TypedBuffer {
    pub buffer: cl::Buffer<u8>,
    pub buffer_type: BufferType,
    pub length: usize,
}

impl TypedBuffer {
    pub fn new(
        program: &cl::Program,
        buffer_type: BufferType,
        length: usize,
    ) -> cl::GPUResult<Self> {
        Ok(Self {
            buffer_type,
            buffer: program.create_buffer::<u8>(buffer_type.size_of() * length)?,
            length,
        })
    }
}

pub struct GPU {
    program: cl::Program,
    buffers: HashMap<String, TypedBuffer>,
}

impl GPU {
    pub fn new(source: String) -> cl::GPUResult<Self> {
        let dev = cl::Device::all()?[0].clone();
        Ok(GPU {
            program: cl::Program::from_opencl(dev, &source)?,
            buffers: HashMap::new(),
        })
    }

    pub fn create_buffer(
        &mut self,
        name: String,
        buffer_type: BufferType,
        length: usize,
    ) -> cl::GPUResult<()> {
        self.buffers
            .insert(name, TypedBuffer::new(&self.program, buffer_type, length)?);
        Ok(())
    }

    pub fn read_buffer<T: Clone>(&self, name: String) -> cl::GPUResult<Vec<T>> {
        let buff = self.buffers.get(&name).unwrap();
        let mut as_u8 = vec![0u8; buff.buffer.length()];
        buff.buffer.read_into(&mut as_u8)?;
        Ok(unsafe { std::slice::from_raw_parts(as_u8.as_ptr() as *const T, buff.length).to_vec() })
    }

    pub fn run_kernel(
        &mut self,
        env: &Environment,
        name: String,
        args: Vec<Arg>,
        global_work_size: usize,
    ) -> cl::GPUResult<()> {
        let mut kern = self
            .program
            .create_kernel(&name[..], global_work_size, None);
        for arg in args {
            match arg {
                Arg::Char(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Uchar(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Short(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Ushort(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Int(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Uint(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Long(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Ulong(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Float(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Double(v) => {
                    kern = kern.arg(v.compute(env));
                }
                Arg::Buffer(name) => {
                    let buff = self.buffers.get(&name.compute(env)).unwrap();
                    kern = kern.arg(&buff.buffer);
                }
            }
        }

        kern.run()?;

        Ok(())
    }
}
