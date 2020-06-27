use crate::conf::{Arg, BufferType, Environment};
use ocl;
use ocl::prm::Float4;
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

macro_rules! expand_arg {
    ($builder:expr, $v:expr) => {{
        $builder.arg($v);
    }};
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
    ($buffer:expr, $type:ident, $target_type:ty) => {{
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
                rd.as_ptr() as *const () as *const $target_type,
                len * std::mem::size_of::<$type>() / std::mem::size_of::<$target_type>(),
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
            BufferType::Char => expand_upcast!(self.pro_que, i8, BufferType::Char, length),
            BufferType::Uchar => expand_upcast!(self.pro_que, u8, BufferType::Uchar, length),
            BufferType::Short => expand_upcast!(self.pro_que, i16, BufferType::Short, length),
            BufferType::Ushort => expand_upcast!(self.pro_que, u16, BufferType::Ushort, length),
            BufferType::Int => expand_upcast!(self.pro_que, i32, BufferType::Int, length),
            BufferType::Uint => expand_upcast!(self.pro_que, u32, BufferType::Uint, length),
            BufferType::Long => expand_upcast!(self.pro_que, i64, BufferType::Long, length),
            BufferType::Ulong => expand_upcast!(self.pro_que, u64, BufferType::Ulong, length),
            BufferType::Float => expand_upcast!(self.pro_que, f32, BufferType::Float, length),
            BufferType::Double => expand_upcast!(self.pro_que, f64, BufferType::Double, length),
            BufferType::Float4 => expand_upcast!(self.pro_que, Float4, BufferType::Float4, length),
        };
        self.buffers.insert(name, buff);
        Ok(())
    }

    pub fn read_buffer<T: Clone>(&self, name: String) -> ocl::Result<Vec<T>> {
        let buff = self.buffers.get(&name).unwrap();
        Ok(match buff.get_type() {
            BufferType::Char => expand_reader!(buff, i8, T),
            BufferType::Uchar => expand_reader!(buff, u8, T),
            BufferType::Short => expand_reader!(buff, i16, T),
            BufferType::Ushort => expand_reader!(buff, u16, T),
            BufferType::Int => expand_reader!(buff, i32, T),
            BufferType::Uint => expand_reader!(buff, u32, T),
            BufferType::Long => expand_reader!(buff, i64, T),
            BufferType::Ulong => expand_reader!(buff, u64, T),
            BufferType::Float => expand_reader!(buff, f32, T),
            BufferType::Double => expand_reader!(buff, f64, T),
            BufferType::Float4 => expand_reader!(buff, Float4, T),
        })
    }

    pub fn run_kernel(
        &mut self,
        env: &Environment,
        name: String,
        mut args: Vec<Arg>,
        global_work_size: usize,
    ) -> ocl::Result<()> {
        let mut builder = self.pro_que.kernel_builder(&name[..]);
        builder.global_work_size([global_work_size]);

        for arg in args.iter_mut() {
            match arg {
                Arg::Char(v) => expand_arg!(builder, v.compute(env)),
                Arg::Uchar(v) => expand_arg!(builder, v.compute(env)),
                Arg::Short(v) => expand_arg!(builder, v.compute(env)),
                Arg::Ushort(v) => expand_arg!(builder, v.compute(env)),
                Arg::Int(v) => expand_arg!(builder, v.compute(env)),
                Arg::Uint(v) => expand_arg!(builder, v.compute(env)),
                Arg::Long(v) => expand_arg!(builder, v.compute(env)),
                Arg::Ulong(v) => expand_arg!(builder, v.compute(env)),
                Arg::Float(v) => expand_arg!(builder, v.compute(env)),
                Arg::Double(v) => expand_arg!(builder, v.compute(env)),
                Arg::Buffer(name) => {
                    let buff = self.buffers.get(name).unwrap();
                    match buff.get_type() {
                        BufferType::Char => expand_arg!(builder, buff, i8),
                        BufferType::Uchar => expand_arg!(builder, buff, u8),
                        BufferType::Short => expand_arg!(builder, buff, i16),
                        BufferType::Ushort => expand_arg!(builder, buff, u16),
                        BufferType::Int => expand_arg!(builder, buff, i32),
                        BufferType::Uint => expand_arg!(builder, buff, u32),
                        BufferType::Long => expand_arg!(builder, buff, i64),
                        BufferType::Ulong => expand_arg!(builder, buff, u64),
                        BufferType::Float => expand_arg!(builder, buff, f32),
                        BufferType::Double => expand_arg!(builder, buff, f64),
                        BufferType::Float4 => expand_arg!(builder, buff, Float4),
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
