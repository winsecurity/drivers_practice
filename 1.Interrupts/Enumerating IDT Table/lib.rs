#![no_std]
#![allow(unused_imports)]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

use winapi::km::wdm::DRIVER_OBJECT;
use winapi::shared::basetsd::PSIZE_T;
use winapi::shared::ntdef::*;
use winapi::ctypes::*;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::vc::vcruntime::size_t;

#[derive(Copy,Clone)]
#[repr(C)]
#[repr(packed)]
pub struct MM_COPY_ADDRESS{
    address: *mut c_void
}


#[link(name = "ntoskrnl")]
extern "C"{
    pub fn DbgPrint(format: *const u8, ...) -> NTSTATUS;
    pub fn MmCopyMemory(targetaddress: *mut c_void,
    sourceaddress: MM_COPY_ADDRESS, 
    numberofbytes: usize,
    flags: u32, byteswritten: *mut usize) -> i32; 
    pub fn MmIsAddressValid(virtualaddress: *mut c_void) -> u8;
}



#[derive(Clone,Copy)]
#[repr(C)]
#[repr(packed)]
struct idtr{
    limit: i16,
    registervalue: i64
}


#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
struct idtentry64{
    offsetlow: u16,
    selector: u16,
    reserveddpltype: u16,
    offsetmiddle: u16,
    offsethigh: u32,
    reserved1: u32
}




#[no_mangle]
pub extern "system" fn driver_entry(_driver: &mut DRIVER_OBJECT,
     _: *const UNICODE_STRING) -> u32 {
    unsafe {

       
 
        //DbgPrint("hello from rust\0".as_ptr() );
       
        let mut idt = idtr{limit:0, registervalue:0};
        let mut ap = &mut idt as *mut _ as *mut u64;
        asm!(
            "sidt  [{0}]",
            out(reg) ap  
           
        );
       
        for k in 0..256{

       

            let isvalid = MmIsAddressValid((idt.registervalue as usize + (k as usize*16)) as *mut c_void);
            if isvalid==1{
                // these strings will be in continuous
               // DbgPrint("%I64x\0".as_ptr(), idt.registervalue);
               // DbgPrint("address is valid\n\0".as_ptr());


                let addr = MM_COPY_ADDRESS{address:(idt.registervalue as usize + (k as usize*16)) as *mut c_void};
                let mut byteswritten = 0;
                let mut contents:[u8;16] = [0;16];
                let res = MmCopyMemory(&mut contents as *mut _ as *mut c_void, 
                    addr, 
                    16, 
                    0x2, // MM_COPY_MEMORY_VIRTUAL 0x2
                    &mut byteswritten);

                //DbgPrint("return value from mmcopymemory: %I32x\0".as_ptr(), res);
            // DbgPrint("number of bytes copied: %I64d\0".as_ptr(),byteswritten);   

                let idtentry = *(&mut contents as *mut _ as *mut idtentry64);
                DbgPrint("%I64x%I64x%I64x\n\0".as_ptr(),
                idtentry.offsethigh as c_uint,
                idtentry.offsetmiddle as c_uint,
                idtentry.offsetlow as c_uint);

                //DbgPrint("\0".as_ptr());

            }
            else{
                DbgPrint("not a valid address\n\0".as_ptr());
            }
        //DbgPrint("%d\n".as_ptr(), idt.limit as c_int);
        //DbgPrint("%lld\n".as_ptr(), idt.registervalue );

        //DbgPrint("%u\0".as_ptr(),ap as *const u8);
        /*for i in 0..a.len(){
            DbgPrint("%#010x\n".as_ptr(),
            a[i]  as c_uint);
        }*/
    }
        
 
       
    }

    0
}





#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 {
    unimplemented!()
}
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
