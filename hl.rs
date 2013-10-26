// Higher level OpenCL wrappers.

use CL;
use CL::*;
use CL::ll::*;
use error::check;
use std::libc;
use std::vec;
use std::str;
use std::rt::io;
use std::rt::io::file;
use std::rt::io::extensions::ReaderUtil;
use std::mem;
use std::cast;
use std::ptr;

enum DeviceType {
      CPU, GPU
}

fn convert_device_type(device: DeviceType) -> cl_device_type {
    match device {
        CPU => CL_DEVICE_TYPE_CPU,
        GPU => CL_DEVICE_TYPE_GPU | CL_DEVICE_TYPE_ACCELERATOR
    }
}

struct Platform {
    id: cl_platform_id
}

impl Platform {
    #[fixed_stack_segment] #[inline(never)]
    fn get_devices_internal(&self, dtype: cl_device_type) -> ~[Device]
    {
        unsafe
        {
            let num_devices = 0;
            
            info!("Looking for devices matching {:?}", dtype);
            
            clGetDeviceIDs(self.id, dtype, 0, ptr::null(), 
                           ptr::to_unsafe_ptr(&num_devices));

            let ids = vec::from_elem(num_devices as uint, 0 as cl_device_id);
            do ids.as_imm_buf |ids, len| {
                clGetDeviceIDs(self.id, dtype, len as cl_uint,
                               ids, ptr::to_unsafe_ptr(&num_devices));
            };

            do ids.map |id| { Device {id: *id }}
        }
    }

    pub fn get_devices(&self) -> ~[Device]
    {
        self.get_devices_internal(CL_DEVICE_TYPE_ALL)
    }

    pub fn get_devices_by_types(&self, types: &[DeviceType]) -> ~[Device]
    {
        let dtype = 0;
        for &t in types.iter() {
          dtype != convert_device_type(t);
        }

        self.get_devices_internal(dtype)
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn name(&self) -> ~str
    {
        unsafe {
            let mut size = 0;

            clGetPlatformInfo(self.id,
                            CL_PLATFORM_NAME,
                            0,
                            ptr::null(),
                            ptr::to_mut_unsafe_ptr(&mut size));

            let name = " ".repeat(size as uint);

            do name.as_imm_buf |p, len| {
            clGetPlatformInfo(self.id,
                              CL_PLATFORM_NAME,
                              len as libc::size_t,
                              p as *libc::c_void,
                              ptr::to_mut_unsafe_ptr(&mut size));
            };

            name
        }
    }
}

#[fixed_stack_segment] #[inline(never)]
pub fn get_platforms() -> ~[Platform]
{
    let num_platforms = 0;

    unsafe
    {
        let status = clGetPlatformIDs(0,
                                      ptr::null(),
                                      ptr::to_unsafe_ptr(&num_platforms));
        check(status, "could not get platform count.");

        let ids = vec::from_elem(num_platforms as uint, 0 as cl_platform_id);

        do ids.as_imm_buf |ids, len| {
            let status = clGetPlatformIDs(len as cl_uint,
                                          ids,
                                          ptr::to_unsafe_ptr(&num_platforms));
            check(status, "could not get platforms.");
        };

        do ids.map |id| { Platform { id: *id } }
    }
}

struct Device {
    id: cl_device_id
}

impl Device {
    #[fixed_stack_segment] #[inline(never)]
    pub fn name(&self) -> ~str { unsafe {
        let size = 0;
        let status = clGetDeviceInfo(
            self.id,
            CL_DEVICE_NAME,
            0,
            ptr::null(),
            ptr::to_unsafe_ptr(&size));
        check(status, "Could not determine name length");

        let buf = vec::from_elem(size as uint, 0);

        do buf.as_imm_buf |p, len| {
            let status = clGetDeviceInfo(
                self.id,
                CL_DEVICE_NAME,
                len as libc::size_t,
                p as *libc::c_void,
                ptr::null());
            check(status, "Could not get device name");

            str::raw::from_c_str(p as *i8)
        }
    } }

    #[fixed_stack_segment] #[inline(never)]
	pub fn computeUnits(&self) -> uint {
		unsafe {
			let mut ct: uint = 0;
            let status = clGetDeviceInfo(
                self.id,
                CL_DEVICE_MAX_COMPUTE_UNITS,
                8,
                ptr::to_mut_unsafe_ptr(&mut ct) as *libc::c_void,
                ptr::null());
            check(status, "Could not get number of device compute units.");
			return ct;
		}
	}


    #[fixed_stack_segment] #[inline(never)]
    pub fn create_context(&self) -> Context
    {
        unsafe
        {
            // TODO: Support for multiple devices
            let errcode = 0;

            // TODO: Proper error messages
            let ctx = clCreateContext(ptr::null(),
                                      1,
                                      ptr::to_unsafe_ptr(&self.id),
                                      cast::transmute(ptr::null::<&fn ()>()),
                                      ptr::null(),
                                      ptr::to_unsafe_ptr(&errcode));

            check(errcode, "Failed to create opencl context!");

            Context { ctx: ctx }
        }
    }
}



struct Context {
    ctx: cl_context,
}

impl Context {
    #[fixed_stack_segment] #[inline(never)]
    pub fn create_buffer<T>(&self, size: uint, flags: cl_mem_flags) -> CLBuffer<T>
    {
        unsafe {
            let status = 0;
            let buf = clCreateBuffer(self.ctx,
                                     flags,
                                     (size*mem::size_of::<T>()) as libc::size_t ,
                                     ptr::null(),
                                     ptr::to_unsafe_ptr(&status));
            check(status, "Could not allocate buffer");
            CLBuffer{cl_buffer: buf}
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn create_buffer_from_vec<T>(&self, vec: &[T]) -> CLBuffer<T>
    {
        unsafe {
            do vec.as_imm_buf |p, len| {
                let status = 0;
                let buf = clCreateBuffer(self.ctx,
                                         CL_MEM_READ_WRITE | CL_MEM_COPY_HOST_PTR,
                                         (len*mem::size_of::<T>()) as libc::size_t ,
                                         p as *libc::c_void,
                                         ptr::to_unsafe_ptr(&status));
                check(status, "Could not allocate buffer");
                CLBuffer{cl_buffer: buf}
            }
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn create_command_queue(&self, device: &Device) -> CommandQueue
    {
        unsafe
        {
            let errcode = 0;
            
            let cqueue = clCreateCommandQueue(self.ctx, device.id, 0,
                                              ptr::to_unsafe_ptr(&errcode));
            
            check(errcode, "Failed to create command queue!");
            
            CommandQueue {
                cqueue: cqueue
            }
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn create_program_from_source(&self, src: &str) -> Program
    {
        unsafe
        {
            do src.to_c_str().with_ref |src| {
                let status = CL_SUCCESS as cl_int;
                let program = clCreateProgramWithSource(
                    self.ctx,
                    1,
                    ptr::to_unsafe_ptr(&src),
                    ptr::null(),
                    ptr::to_unsafe_ptr(&status));
                check(status, "Could not create program");

                Program { prg: program }
            }
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn create_program_from_binary(&self, bin: &str, device: &Device) -> Program {
        do bin.to_c_str().with_ref |src| {
            let status = CL_SUCCESS as cl_int;
            let len = bin.len() as libc::size_t;
            let program = unsafe {
                clCreateProgramWithBinary(
                    self.ctx,
                    1,
                    ptr::to_unsafe_ptr(&device.id),
                    ptr::to_unsafe_ptr(&len),
                    ptr::to_unsafe_ptr(&src) as **libc::c_uchar,
                    ptr::null(),
                    ptr::to_unsafe_ptr(&status))
            };
            check(status, "Could not create program");

            Program {prg: program}
        }
    }
}

impl Drop for Context
{
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            clReleaseContext(self.ctx);
        }
    }
}


trait Buffer<T> {
    fn id(&self) -> cl_mem;

    #[fixed_stack_segment] #[inline(never)]
    fn byte_len(&self) -> libc::size_t 
    {
        unsafe {
            let size : libc::size_t = 0;
            let err = clGetMemObjectInfo(self.id(),
                                         CL_MEM_SIZE,
                                         mem::size_of::<libc::size_t>() as libc::size_t,
                                         ptr::to_unsafe_ptr(&size) as *libc::c_void,
                                         ptr::null());

            check(err, "Failed to read memory size");
            size
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    fn len(&self) -> uint 
    {
        self.byte_len() as uint / mem::size_of::<T>()
    }
}

pub struct CLBuffer<T> {
    cl_buffer: cl_mem
}

#[unsafe_destructor]
impl<T> Drop for CLBuffer<T> {
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            clReleaseMemObject(self.cl_buffer);
        }
    }
}

impl<T> Buffer<T> for CLBuffer<T> {
    fn id(&self) -> cl_mem 
    {
        self.cl_buffer
    }
}

impl<T> KernelArg for CLBuffer<T> {
    fn get_value(&self) -> (libc::size_t, *libc::c_void)
    {
        (mem::size_of::<cl_mem>() as libc::size_t,
         ptr::to_unsafe_ptr(&self.cl_buffer) as *libc::c_void)
    }
} 

pub struct CommandQueue {
    cqueue: cl_command_queue
}

impl CommandQueue
{
    #[fixed_stack_segment] #[inline(never)]
    pub fn enqueue_async_kernel<I: KernelIndex, E: EventList>(&self, k: &Kernel, global: I, local: Option<I>, wait_on: E)
        -> Event
    {
        unsafe
        {
            do wait_on.as_event_list |event, event_count| {
                let e: cl_event = ptr::null();
                let status = clEnqueueNDRangeKernel(
                    self.cqueue,
                    k.kernel,
                    KernelIndex::num_dimensions(None::<I>),
                    ptr::null(),
                    global.get_ptr(),
                    match local {
                        Some(ref l) => l.get_ptr(),
                        None => ptr::null()
                    },
                    event_count,
                    event,
                    ptr::to_unsafe_ptr(&e));
                check(status, "Error enqueuing kernel.");
                Event { event: e }
            }
        }
    }

    pub fn read<T, B: Buffer<T>, E: EventList>(&self, mem: &B, event: E) -> ~[T]
    {
        let mut v: ~[T] = vec::with_capacity(mem.len());
        unsafe {
            vec::raw::set_len(&mut v, mem.len());
        }
        self.read_buffer(mem, 0, v, event);
        v
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn write_buffer<T, B: Buffer<T>, E: EventList>(&self, mem: &B, offset: uint, write: &[T], event: E)
    {
        unsafe {
            do event.as_event_list |evt, evt_len| {
                do write.as_imm_buf |p, len| {
                    let err = clEnqueueWriteBuffer(self.cqueue,
                                                   mem.id(),
                                                   CL_TRUE,
                                                   (offset * mem::size_of::<T>()) as libc::size_t,
                                                   (len * mem::size_of::<T>()) as libc::size_t,
                                                   p as *libc::c_void,
                                                   evt_len,
                                                   evt,
                                                   ptr::null());

                    check(err, "Failed to write buffer");
                }
            }
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn read_buffer<T, B: Buffer<T>, E: EventList>(&self, mem: &B, offset: uint, read: &mut [T], event: E)
    {
        unsafe {
            do event.as_event_list |evt, evt_len| {
                do read.as_imm_buf |p, len| {
                    let err = clEnqueueReadBuffer(self.cqueue,
                                                  mem.id(),
                                                  CL_TRUE,
                                                  (offset * mem::size_of::<T>()) as libc::size_t,
                                                  (len * mem::size_of::<T>()) as libc::size_t,
                                                  p as *libc::c_void,
                                                  evt_len,
                                                  evt,
                                                  ptr::null());

                    check(err, "Failed to read buffer");
                }
            }
        }
    }

    // this is unsafe since freeing of write after enqueueing
    // can cause undefined behavior
    #[fixed_stack_segment] #[inline(never)]
    pub unsafe fn write_buffer_async<T, B: Buffer<T>, E: EventList>(&self, mem: &B, offset: uint, write: &[T], event: E) -> Event
    {
        do event.as_event_list |evt, evt_len| {
            do write.as_imm_buf |p, len| {
                let e: cl_event = ptr::null();
                let err = clEnqueueWriteBuffer(self.cqueue,
                                               mem.id(),
                                               CL_FALSE,
                                               (offset * mem::size_of::<T>()) as libc::size_t,
                                               (len * mem::size_of::<T>()) as libc::size_t,
                                               p as *libc::c_void,
                                               evt_len,
                                               evt,
                                               ptr::to_unsafe_ptr(&e));

                check(err, "Failed to write buffer");
                Event{event: e}
            }
        }
    }

    // this is unsafe since freeing of write after enqueueing
    // can cause undefined behavior
    #[fixed_stack_segment] #[inline(never)]
    pub unsafe fn read_buffer_async<T, B: Buffer<T>, E: EventList>(&self, mem: &B, offset: uint, read: &mut [T], event: E) -> Event
    {
        do event.as_event_list |evt, evt_len| {
            do read.as_imm_buf |p, len| {
                let e: cl_event = ptr::null();
                let err = clEnqueueReadBuffer(self.cqueue,
                                              mem.id(),
                                              CL_FALSE,
                                              (offset * mem::size_of::<T>()) as libc::size_t,
                                              (len * mem::size_of::<T>()) as libc::size_t,
                                              p as *libc::c_void,
                                              evt_len,
                                              evt,
                                              ptr::to_unsafe_ptr(&e));

                check(err, "Failed to read buffer");
                Event{event: e}
            }
        }
    }
}

impl Drop for CommandQueue
{
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            clReleaseCommandQueue(self.cqueue);
        }
    }
}


pub struct Program
{
    prg: cl_program,
}

impl Drop for Program
{
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            clReleaseProgram(self.prg);
        }
    }
}

impl Program
{
    pub fn build(&self, device: Device) -> Result<(), ~str> {
        build_program(self, device)
    }

    pub fn create_kernel(&self, name: &str) -> Kernel {
        create_kernel(self, name)
    }
}

// TODO: Support multiple devices
#[fixed_stack_segment] #[inline(never)]
pub fn create_program_with_binary(ctx: & Context, device: Device,
                                  binary_path: & Path) -> Program
{
    unsafe
    {
        let errcode = 0;
        let mut file = file::open(binary_path, io::Open, io::Read);
        let binary = file.read_to_end();
        let program = do binary.to_c_str().with_ref |kernel_binary| {
            clCreateProgramWithBinary(ctx.ctx, 1, ptr::to_unsafe_ptr(&device.id),
                                      ptr::to_unsafe_ptr(&(binary.len() + 1)) as *libc::size_t,
                                      ptr::to_unsafe_ptr(&kernel_binary) as **libc::c_uchar,
                                      ptr::null(),
                                      ptr::to_unsafe_ptr(&errcode))
        };

        check(errcode, "Failed to create open cl program with binary!");

        Program {
            prg: program,
        }
    }
}

#[fixed_stack_segment] #[inline(never)]
pub fn build_program(program: & Program, device: Device) -> Result<(), ~str>
{
    unsafe
    {
        let ret = clBuildProgram(program.prg, 1, ptr::to_unsafe_ptr(&device.id),
                                 ptr::null(),
                                 cast::transmute(ptr::null::<&fn ()>()),
                                 ptr::null());
        if ret == CL_SUCCESS as cl_int {
            Ok(())
        }
        else {
            let size = 0 as libc::size_t;
            let status = clGetProgramBuildInfo(
                program.prg,
                device.id,
                CL_PROGRAM_BUILD_LOG,
                0,
                ptr::null(),
                ptr::to_unsafe_ptr(&size));
            check(status, "Could not get build log");

            let buf = vec::from_elem(size as uint, 0u8);
            do buf.as_imm_buf |p, len| {
                let status = clGetProgramBuildInfo(
                    program.prg,
                    device.id,
                    CL_PROGRAM_BUILD_LOG,
                    len as libc::size_t,
                    p as *libc::c_void,
                    ptr::null());
                check(status, "Could not get build log");

                Err(str::raw::from_c_str(p as *libc::c_char))
            }
        }
    }
}

pub struct Kernel {
    kernel: cl_kernel,
}

impl Drop for Kernel
{
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            clReleaseKernel(self.kernel);
        }
    }
}

impl Kernel {
    pub fn set_arg<T: KernelArg>(&self, i: uint, x: &T)
    {
        set_kernel_arg(self, i as CL::cl_uint, x)
    }
}

#[fixed_stack_segment] #[inline(never)]
pub fn create_kernel(program: & Program, kernel: & str) -> Kernel
{
    unsafe {
        let errcode = 0;
        // let bytes = str::to_bytes(kernel);
        do kernel.to_c_str().with_ref |str_ptr|
        {
            let kernel = clCreateKernel(program.prg,
                                        str_ptr,
                                        ptr::to_unsafe_ptr(&errcode));

            check(errcode, "Failed to create kernel!");

            Kernel {
                kernel: kernel,
            }
        }
    }
}

pub trait KernelArg {
  fn get_value(&self) -> (libc::size_t, *libc::c_void);
}

macro_rules! scalar_kernel_arg (
    ($t:ty) => (impl KernelArg for $t {
        fn get_value(&self) -> (libc::size_t, *libc::c_void) {
            (mem::size_of::<$t>() as libc::size_t,
             ptr::to_unsafe_ptr(self) as *libc::c_void)
        }
    })
)

scalar_kernel_arg!(int)
scalar_kernel_arg!(uint)
scalar_kernel_arg!(u32)
scalar_kernel_arg!(u64)
scalar_kernel_arg!(i32)
scalar_kernel_arg!(i64)
scalar_kernel_arg!(f32)
scalar_kernel_arg!(f64)

#[fixed_stack_segment] #[inline(never)]
pub fn set_kernel_arg<T: KernelArg>(kernel: & Kernel,
                                    position: cl_uint,
                                    arg: &T)
{
    unsafe
    {
        let (size, p) = arg.get_value();
        let ret = clSetKernelArg(kernel.kernel, position,
                                 size,
                                 p);

        check(ret, "Failed to set kernel arg!");
    }
}

#[fixed_stack_segment] #[inline(never)]
pub fn enqueue_nd_range_kernel(cqueue: & CommandQueue, kernel: & Kernel, work_dim: cl_uint,
                               _global_work_offset: int, global_work_size: int,
                               local_work_size: int)
{
  unsafe
    {
      let ret = clEnqueueNDRangeKernel(cqueue.cqueue, kernel.kernel, work_dim,
                                       // ptr::to_unsafe_ptr(&global_work_offset) as *libc::size_t,
                                       ptr::null(),
                                       ptr::to_unsafe_ptr(&global_work_size) as *libc::size_t,
                                       ptr::to_unsafe_ptr(&local_work_size) as *libc::size_t,
                                       0, ptr::null(), ptr::null());
      check(ret, "Failed to enqueue nd range kernel!");
  }
}

pub struct Event
{
    event: cl_event,
}

impl Drop for Event
{
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            clReleaseEvent(self.event);
        }
    }
}

trait EventList {
    fn as_event_list<T>(&self, &fn(*cl_event, cl_uint) -> T) -> T;

    #[fixed_stack_segment] #[inline(never)]
    fn wait(&self) {
        do self.as_event_list |p, len| {
            unsafe {
                let status = clWaitForEvents(len, p);
                check(status, "Error waiting for event(s)");
            }
        }
    }
}

impl<'self> EventList for &'self Event {
    fn as_event_list<T>(&self, f: &fn(*cl_event, cl_uint) -> T) -> T
    {
        f(ptr::to_unsafe_ptr(&self.event), 1 as cl_uint)
    }
}

impl EventList for Event {
    fn as_event_list<T>(&self, f: &fn(*cl_event, cl_uint) -> T) -> T
    {
        f(ptr::to_unsafe_ptr(&self.event), 1 as cl_uint)
    }
}

impl<T: EventList> EventList for Option<T> {
    fn as_event_list<T>(&self, f: &fn(*cl_event, cl_uint) -> T) -> T
    {
        match *self {
            None => f(ptr::null(), 0),
            Some(ref s) => s.as_event_list(f)
        }
    }
}

impl<'self> EventList for &'self [Event] {
    fn as_event_list<T>(&self, f: &fn(*cl_event, cl_uint) -> T) -> T
    {
        /* this is wasteful */
        let events = self.iter().map(|event| event.event).to_owned_vec();

        do events.as_imm_buf |p, len| {
            f(p as **libc::c_void, len as cl_uint)
        }
    }
}

/* this seems VERY hackey */
impl EventList for () {
    fn as_event_list<T>(&self, f: &fn(*cl_event, cl_uint) -> T) -> T
    {
        f(ptr::null(), 0)
    }
}

trait KernelIndex
{
    fn num_dimensions(dummy_self: Option<Self>) -> cl_uint;
    fn get_ptr(&self) -> *libc::size_t;
}

impl KernelIndex for int
{
    fn num_dimensions(_: Option<int>) -> cl_uint { 1 }

    fn get_ptr(&self) -> *libc::size_t
    {
        ptr::to_unsafe_ptr(self) as *libc::size_t
    }
}

impl KernelIndex for (int, int) {
    fn num_dimensions(_: Option<(int, int)>) -> cl_uint { 2 }

    fn get_ptr(&self) -> *libc::size_t {
        ptr::to_unsafe_ptr(self) as *libc::size_t
    }
}

impl KernelIndex for uint
{
    fn num_dimensions(_: Option<uint>) -> cl_uint { 1 }

    fn get_ptr(&self) -> *libc::size_t {
        ptr::to_unsafe_ptr(self) as *libc::size_t
    }
}

impl KernelIndex for (uint, uint)
{
    fn num_dimensions(_: Option<(uint, uint)>) -> cl_uint { 2 }

    fn get_ptr(&self) -> *libc::size_t {
        ptr::to_unsafe_ptr(self) as *libc::size_t
    }
}

#[cfg(test)]
mod test {
    use CL::*;
    use hl::*;
    use vector::Vector;
    use std::rt::io;
    use std::io;
    use util;

    macro_rules! expect (
        ($test: expr, $expected: expr) => ({
            let test     = $test;
            let expected = $expected;
            if test != expected {
                fail!(format!("Test failure in {:s}: expected {:?}, got {:?}",
                           stringify!($test),
                           expected, test))
            }
        })
    )

      #[test]
    fn program_build() {
        let src = "__kernel void test(__global int *i) { \
                   *i += 1; \
                   }";
        let (_, device, ctx, _) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);
        prog.build(device);
    }

    #[test]
    fn simple_kernel() {
        let src = "__kernel void test(__global int *i) { \
                   *i += 1; \
                   }";
        let (_, device, ctx, queue) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);
        prog.build(device);

        let k = prog.create_kernel("test");
        let v = ctx.create_buffer_from_vec([1]);
        
        k.set_arg(0, &v);

        enqueue_nd_range_kernel(
            &queue,
            &k,
            1, 0, 1, 1);

        let v = queue.read(&v, ());

        expect!(v[0], 2);
    }

    #[test]
    fn add_k() {
        let src = "__kernel void test(__global int *i, long int k) { \
                   *i += k; \
                   }";
        let (_, device, ctx, queue) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);
        prog.build(device);

        let k = prog.create_kernel("test");
        
        let v = ctx.create_buffer_from_vec([1]);
        
        k.set_arg(0, &v);
        k.set_arg(1, &42);

        enqueue_nd_range_kernel(
              &queue,
              &k,
              1, 0, 1, 1);

        let v = queue.read(&v, ());

        expect!(v[0], 43);
  }

    #[test]
    fn simple_kernel_index() {
        let src = "__kernel void test(__global int *i) { \
                   *i += 1; \
                   }";
        let (_, device, ctx, queue) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);
        prog.build(device);

        let k = prog.create_kernel("test");

        let v = ctx.create_buffer_from_vec([1]);
      
        k.set_arg(0, &v);

        queue.enqueue_async_kernel(&k, 1, None, ()).wait();
      
        let v = queue.read(&v, ());

        expect!(v[0], 2);
    }

    #[test]
    fn chain_kernel_event() {
        let src = "__kernel void test(__global int *i) { \
                   *i += 1; \
                   }";
        let (_, device, ctx, queue) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);
        prog.build(device);

        let k = prog.create_kernel("test");
        let v = ctx.create_buffer_from_vec([1]);
      
        k.set_arg(0, &v);

        let mut e : Option<Event> = None;
        for _ in range(0, 8) {
            e = Some(queue.enqueue_async_kernel(&k, 1, None, e));
        }
        e.wait();
      
        let v = queue.read(&v, ());

        expect!(v[0], 9);
    }

    #[test]
    fn chain_kernel_event_list() {
        let src = "__kernel void inc(__global int *i) { \
                   *i += 1; \
                   } \
                   __kernel void add(__global int *a, __global int *b, __global int *c) { \
                   *c = *a + *b; \
                   }";
        let (_, device, ctx, queue) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);
        prog.build(device);

        let k_incA = prog.create_kernel("inc");
        let k_incB = prog.create_kernel("inc");
        let k_add = prog.create_kernel("add");
        
        let a = ctx.create_buffer_from_vec([1]);
        let b = ctx.create_buffer_from_vec([1]);
        let c = ctx.create_buffer_from_vec([1]);
      
        k_incA.set_arg(0, &a);
        k_incB.set_arg(0, &b);
        let event_list = &[
            queue.enqueue_async_kernel(&k_incA, 1, None, ()),
            queue.enqueue_async_kernel(&k_incB, 1, None, ()),
        ];

        k_add.set_arg(0, &a);
        k_add.set_arg(1, &b);
        k_add.set_arg(2, &c);
        let event = queue.enqueue_async_kernel(&k_add, 1, None, event_list);
      
        let v = queue.read(&c, event);

        expect!(v[0], 4);
    }

    #[test]
    fn kernel_2d()
    {
        let src = "__kernel void test(__global long int *N) { \
                   int i = get_global_id(0); \
                   int j = get_global_id(1); \
                   int s = get_global_size(0); \
                   N[i * s + j] = i * j;
}";
        let (_, device, ctx, queue) = util::create_compute_context().unwrap();
        let prog = ctx.create_program_from_source(src);

        match prog.build(device) {
            Ok(()) => (),
            Err(build_log) => {
                println!("Error building program:\n");
                println!("{:s}", build_log);
                fail!("");
            }
        }

        let k = prog.create_kernel("test");
        
        let v = ctx.create_buffer_from_vec([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        
        k.set_arg(0, &v);

        queue.enqueue_async_kernel(&k, (3, 3), None, ()).wait();
        
        let v = queue.read(&v, ());
        
        expect!(v, ~[0, 0, 0, 0, 1, 2, 0, 2, 4]);
    }

    #[test]
    fn memory_read_write()
    {
        let (_, _, ctx, queue) = util::create_compute_context().unwrap();
        let buffer : CLBuffer<int> = ctx.create_buffer(8, CL_MEM_READ_ONLY);

        let input = ~[0, 1, 2, 3, 4, 5, 6, 7];
        let mut output = ~[0, 0, 0, 0, 0, 0, 0, 0];

        queue.write_buffer(&buffer, 0, input, ());
        queue.read_buffer(&buffer, 0, output, ());

        expect!(input, output);
    }

    #[test]
    fn memory_read_vec()
    {
        let input = ~[0, 1, 2, 3, 4, 5, 6, 7];
        let (_, _, ctx, queue) = util::create_compute_context().unwrap();
        let buffer : CLBuffer<int> = ctx.create_buffer_from_vec(input);
        let output = queue.read(&buffer, ());
        expect!(input, output);
    }
}
