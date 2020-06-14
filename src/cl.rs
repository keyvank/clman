use crate::conf::{Arg, BufferType};
use ocl;
use std::any::Any;
use std::collections::HashMap;

pub struct GPU {
    pro_que: ocl::ProQue,
    buffers: HashMap<String, Box<dyn GenericBuffer>>,
}

struct TypedBuffer<T: ocl::OclPrm> {
    pub buffer: ocl::Buffer<T>,
    pub buffer_type: BufferType,
}

pub trait GenericBuffer {
    fn get_type(&self) -> BufferType;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: ocl::OclPrm> GenericBuffer for TypedBuffer<T> {
    fn get_type(&self) -> BufferType {
        self.buffer_type
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<T: ocl::OclPrm> TypedBuffer<T> {
    pub fn new(
        pro_que: &mut ocl::ProQue,
        buffer_type: BufferType,
        length: usize,
    ) -> ocl::Result<Self> {
        Ok(Self {
            buffer_type,
            buffer: ocl::Buffer::<T>::builder()
                .queue(pro_que.queue().clone())
                .flags(ocl::MemFlags::new().read_write())
                .len(length)
                .build()?,
        })
    }
}

macro_rules! expand_downcast {
    ($builder:expr, $buffer:expr, $actual_type:ty) => {{
        $builder.arg(
            &$buffer
                .as_any()
                .downcast_ref::<TypedBuffer<$actual_type>>()
                .unwrap()
                .buffer,
        );
    }};
}

macro_rules! expand_upcast {
    ($pro_que:expr, $actual_type:ty, $type_val: expr, $len:expr) => {{
        Box::new(TypedBuffer::<$actual_type>::new(
            &mut $pro_que,
            $type_val,
            $len,
        )?)
    }};
}

macro_rules! expand_reader {
    ($buffer:expr, $type:ident) => {{
        let buff = &$buffer
            .as_any()
            .downcast_ref::<TypedBuffer<$type>>()
            .unwrap()
            .buffer;
        let len = buff.len();
        let mut rd = vec![$type::default(); len];
        buff.read(&mut rd).enq()?;
        unsafe {
            std::slice::from_raw_parts(
                rd.as_ptr() as *const () as *const u8,
                len * std::mem::size_of::<$type>(),
            )
            .to_vec()
        }
    }};
}

impl GPU {
    pub fn new(source: String) -> ocl::Result<Self> {
        Ok(GPU {
            pro_que: ocl::ProQue::builder().src(source).dims(1).build()?,
            buffers: HashMap::new(),
        })
    }

    pub fn create_buffer(
        &mut self,
        name: String,
        buffer_type: BufferType,
        length: usize,
    ) -> ocl::Result<()> {
        let buff: Box<dyn GenericBuffer> = match buffer_type {
            BufferType::Int => expand_upcast!(self.pro_que, i32, BufferType::Int, length),
            BufferType::Uint => expand_upcast!(self.pro_que, u32, BufferType::Uint, length),
            BufferType::Float => expand_upcast!(self.pro_que, f32, BufferType::Float, length),
            BufferType::Double => expand_upcast!(self.pro_que, f64, BufferType::Double, length),
        };
        self.buffers.insert(name, buff);
        Ok(())
    }

    pub fn read_buffer(&self, name: String) -> ocl::Result<Vec<u8>> {
        let buff = self.buffers.get(&name).unwrap();
        Ok(match buff.get_type() {
            BufferType::Uint => expand_reader!(buff, u32),
            BufferType::Int => expand_reader!(buff, i32),
            BufferType::Float => expand_reader!(buff, f32),
            BufferType::Double => expand_reader!(buff, f64),
        })
    }

    pub fn run_kernel(
        &mut self,
        name: String,
        mut args: Vec<Arg>,
        global_work_size: usize,
    ) -> ocl::Result<()> {
        let mut builder = self.pro_que.kernel_builder(&name[..]);
        builder.global_work_size([global_work_size]);

        for arg in args.iter_mut() {
            match arg {
                Arg::Char(v) => {
                    builder.arg(*v);
                }
                Arg::Uchar(v) => {
                    builder.arg(*v);
                }
                Arg::Short(v) => {
                    builder.arg(*v);
                }
                Arg::Ushort(v) => {
                    builder.arg(*v);
                }
                Arg::Int(v) => {
                    builder.arg(*v);
                }
                Arg::Uint(v) => {
                    builder.arg(*v);
                }
                Arg::Long(v) => {
                    builder.arg(*v);
                }
                Arg::Ulong(v) => {
                    builder.arg(*v);
                }
                Arg::Float(v) => {
                    builder.arg(*v);
                }
                Arg::Double(v) => {
                    builder.arg(*v);
                }
                Arg::Buffer(name) => {
                    let buff = self.buffers.get(name).unwrap();
                    match buff.get_type() {
                        BufferType::Int => expand_downcast!(builder, buff, i32),
                        BufferType::Uint => expand_downcast!(builder, buff, u32),
                        BufferType::Float => expand_downcast!(builder, buff, f32),
                        BufferType::Double => expand_downcast!(builder, buff, f64),
                    }
                }
            }
        }

        let kernel = builder.build()?;

        unsafe {
            kernel.enq()?;
        }

        Ok(())
    }
}
