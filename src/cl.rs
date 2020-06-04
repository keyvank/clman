use ocl::ProQue;

pub fn run(source: String) -> ocl::Result<()> {
    let pro_que = ProQue::builder().src(source).dims(1).build()?;

    let kernel = pro_que.kernel_builder("main").build()?;

    unsafe {
        kernel.enq()?;
    }

    Ok(())
}
