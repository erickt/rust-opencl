use std::cast::transmute;
use std::libc::{size_t, c_void};
use std::unstable::raw::Slice;

use CL::*;
use mem::Buffer;
use hl::KernelArg;

pub struct Mapping<BUFFER, T>
{
    buffer: BUFFER,
    ptr: *mut T,
}

impl<T, B: Buffer<T>> Mapping<B, T>
{
    // this takes ownership of B on purpose
    pub unsafe fn new(b: B, ptr: *mut T) -> Mapping<B, T>
    {
        Mapping {
            buffer: b,
            ptr: ptr
        }
    }

    pub fn as_slice<'a>(&'a self) -> &'a [T]
    {
        unsafe {
            transmute(Slice {
                data: self.ptr as *T,
                len: self.buffer.len()
            })
        }
    }

    pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [T]
    {
        unsafe {
            transmute(Slice {
                data: self.ptr as *T,
                len: self.buffer.len()
            })
        }
    }
}

impl<T, B: Buffer<T>> Buffer<T> for Mapping<B, T>
{
    unsafe fn id_ptr(&self) -> *cl_mem
    {
        self.buffer.id_ptr()
    }

    fn id(&self) -> cl_mem
    {
        self.buffer.id()
    }

    fn byte_len(&self) -> size_t
    {
        self.buffer.byte_len()
    }

    fn len(&self) -> uint
    {
        self.buffer.len()
    }
}

impl<B: KernelArg, T> KernelArg for Mapping<B, T> {
    fn get_value(&self) -> (size_t, *c_void)
    {
        self.buffer.get_value()
    }
}