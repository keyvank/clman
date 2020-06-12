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
            BufferType::Int => Box::new(TypedBuffer::<i32>::new(
                &mut self.pro_que,
                BufferType::Int,
                length,
            )?),
            BufferType::Uint => Box::new(TypedBuffer::<u32>::new(
                &mut self.pro_que,
                BufferType::Uint,
                length,
            )?),
            BufferType::Float => Box::new(TypedBuffer::<f32>::new(
                &mut self.pro_que,
                BufferType::Float,
                length,
            )?),
            BufferType::Double => Box::new(TypedBuffer::<f64>::new(
                &mut self.pro_que,
                BufferType::Double,
                length,
            )?),
        })
    }

    pub fn run_kernel(
        &mut self,
        name: String,
        mut args: Vec<Box<dyn GenericBuffer>>,
    ) -> ocl::Result<()> {
        let mut builder = self.pro_que.kernel_builder(&name[..]);

        for arg in args.iter_mut() {
            match arg.get_type() {
                BufferType::Int => {
                    builder.arg(
                        &mut arg
                            .as_any_mut()
                            .downcast_mut::<TypedBuffer<i32>>()
                            .unwrap()
                            .buffer,
                    );
                }
                BufferType::Uint => {
                    builder.arg(
                        &mut arg
                            .as_any_mut()
                            .downcast_mut::<TypedBuffer<u32>>()
                            .unwrap()
                            .buffer,
                    );
                }
                BufferType::Float => {
                    builder.arg(
                        &mut arg
                            .as_any_mut()
                            .downcast_mut::<TypedBuffer<f32>>()
                            .unwrap()
                            .buffer,
                    );
                }
                BufferType::Double => {
                    builder.arg(
                        &mut arg
                            .as_any_mut()
                            .downcast_mut::<TypedBuffer<f64>>()
                            .unwrap()
                            .buffer,
                    );
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
