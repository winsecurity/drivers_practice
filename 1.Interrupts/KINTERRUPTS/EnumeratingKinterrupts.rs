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
pub struct idtr{
    limit: i16,
    registervalue: i64
}


#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct idtentry64{
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


        let mut prcb:u64 = 0;
        
        asm!(
            "mov {0}, gs:[0x20]",
            out(reg) prcb
            // kprcb + 0x3140 = interruptobject array
        );
        DbgPrint("prcb address: %I64x\n\0".as_ptr(), prcb);
        if MmIsAddressValid(prcb as *mut c_void)==1{
            let interruptobjectarray = prcb + 0x3140;
            for i in 0x80..0x81{

                let mut kinterrupt:u64 = 0;

                let mmcopy = MM_COPY_ADDRESS{address: (interruptobjectarray as usize + (i as usize * 8)) as *mut c_void};
                let mut byteswritten = 0;
                let res = MmCopyMemory(&mut kinterrupt as *mut _ as *mut c_void, 
                    mmcopy, 
                    8, 
                    0x2, //virtual,
                     &mut byteswritten);
                    
                if res==0 && byteswritten == 8{
                    DbgPrint("%I64x\n\0".as_ptr(), kinterrupt);
                }

            }
        }

       
       
        
 
       
    }

    0
}



pub fn readbytesat(addr: *mut c_void, n: u64){
    unsafe{
        // addr contains address to read
        // n contains number of bytes to read

        let mmcopy = MM_COPY_ADDRESS{address: addr};
        //let mut destination:[u8;n] = [0;n];
        

    }
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
