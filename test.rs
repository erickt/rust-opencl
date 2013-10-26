extern mod OpenCL;

use std::mem;
use std::rt::io;
use std::rt::io::file;
use std::rt::io::extensions::ReaderUtil;
use std::str;



fn main()
{
	let ker = file::open(&std::path::Path::new("./test.ocl"), io::Open, io::Read).read_to_end();
    let ker = str::from_utf8(ker);

	let vec_a: &[int] = &[0, 1, 2, -3, 4, 5, 6, 7];
	let vec_b: &[int] = &[-7, -6, 5, -4, 0, -1, 2, 3];
	let vec_c: &[int] = &[0, 0, 0, 0, 0, 0, 0, 0];

	let ctx = OpenCL::hl::create_compute_context();

	println!("{:?}", ctx.device_name());

	let A : OpenCL::hl::CLBuffer<int> = ctx.ctx.create_buffer(vec_a.len(), OpenCL::CL::CL_MEM_READ_ONLY);
	let B : OpenCL::hl::CLBuffer<int> = ctx.ctx.create_buffer(vec_a.len(), OpenCL::CL::CL_MEM_READ_ONLY);
	let C : OpenCL::hl::CLBuffer<int> = ctx.ctx.create_buffer(vec_a.len(), OpenCL::CL::CL_MEM_WRITE_ONLY);

	ctx.q.write_buffer(&A, 0, vec_a, ());
	ctx.q.write_buffer(&B, 0, vec_b, ());

	let program = ctx.create_program_from_source(ker);

	program.build(ctx.device);

	let kernel = program.create_kernel("vector_add");

	kernel.set_arg(0, &A);
	kernel.set_arg(1, &B);
	kernel.set_arg(2, &C);

	OpenCL::hl::enqueue_nd_range_kernel(&ctx.q, &kernel, 1, 0, 8, 8);

	ctx.q.read_buffer(&C, 0, vec_c, ());

	println!("	{:?}", vec_a);
	println!("+	{:?}", vec_b);
	println!("=	{:?}", vec_c);
}
