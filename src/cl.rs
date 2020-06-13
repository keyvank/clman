use crate::conf::BufferType;
use ocl;
use std::any::Any;

pub struct GPU {
    pro_que: ocl::ProQue,
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

pub enum KernelArgument<'a> {
    Int(i32),
    Uint(u32),
    Float(f32),
    Double(f64),
    Buffer(&'a Box<dyn GenericBuffer>),
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

impl GPU {
    pub fn new(source: String) -> ocl::Result<Self> {
        Ok(GPU {
            pro_que: ocl::ProQue::builder().src(source).dims(1).build()?,
        })
    }

    pub fn create_buffer(
        &mut self,
        buffer_type: BufferType,
        length: usize,
    ) -> ocl::Result<Box<dyn GenericBuffer>> {
        Ok(match buffer_type {
            BufferType::Int => expand_upcast!(self.pro_que, i32, BufferType::Int, length),
            BufferType::Uint => expand_upcast!(self.pro_que, u32, BufferType::Uint, length),
            BufferType::Float => expand_upcast!(self.pro_que, f32, BufferType::Float, length),
            BufferType::Double => expand_upcast!(self.pro_que, f64, BufferType::Double, length),
        })
    }

    pub fn run_kernel(&mut self, name: String, mut args: Vec<KernelArgument>) -> ocl::Result<()> {
        let mut builder = self.pro_que.kernel_builder(&name[..]);

        for arg in args.iter_mut() {
            match arg {
                KernelArgument::Int(v) => {
                    builder.arg(*v);
                }
                KernelArgument::Uint(v) => {
                    builder.arg(*v);
                }
                KernelArgument::Float(v) => {
                    builder.arg(*v);
                }
                KernelArgument::Double(v) => {
                    builder.arg(*v);
                }
                KernelArgument::Buffer(buff) => match buff.get_type() {
                    BufferType::Int => expand_downcast!(builder, buff, i32),
                    BufferType::Uint => expand_downcast!(builder, buff, u32),
                    BufferType::Float => expand_downcast!(builder, buff, f32),
                    BufferType::Double => expand_downcast!(builder, buff, f64),
                },
            }
        }

        let kernel = builder.build()?;

        unsafe {
            kernel.enq()?;
        }

        Ok(())
    }
}
