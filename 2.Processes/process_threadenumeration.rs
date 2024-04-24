
#![no_std]
#![allow(unused_imports)]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;


use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntpsapi::ZwCurrentProcess;
use winapi::km::wdm::DRIVER_OBJECT;
use winapi::shared::basetsd::PSIZE_T;
use winapi::shared::ntdef::*;
use winapi::ctypes::*;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::vc::vcruntime::size_t;


use ntapi::ntexapi::*;
use ntapi::ntmmapi::*;

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
    pub fn ZwOpenProcess(outhandle: *mut c_void,
    accessmask: u32, objectattrs: *mut OBJECT_ATTRIBUTES,
    pclientid: *mut CLIENT_ID);

    pub fn PsGetCurrentProcessId() -> *mut c_void;
    pub fn PsGetCurrentThreadId() -> *mut c_void;
    pub fn PsLookupProcessByProcessId(pid: HANDLE,peprocess: *mut c_void)-> NTSTATUS;
    pub fn PsSetCreateProcessNotifyRoutine( functionpointer: *mut c_void, toremove: u8) -> NTSTATUS;
    
    //pub fn ZwOpenProcess() -> *mut c_void;
}


#[link(name="ntdll")]
extern "C"{
    pub fn NtClose(handle: *mut c_void) -> NTSTATUS;

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
pub extern "C" fn processcreationcallback(parentid: HANDLE, pid: HANDLE, iscreated: u8){

        // new process has been created
    let mut eprocess:u64 = 0;
        
    let res = unsafe{PsLookupProcessByProcessId(pid, &mut eprocess as *mut _ as *mut c_void)};

    if eprocess!=0{
        let mut pname = unsafe{core::ptr::read((eprocess+0x5a8) as *const [u8;15])};

        if iscreated==1{
           // unsafe{DbgPrint("process created: %s: %I64d\n\0".as_ptr(),
           //  pname.as_mut_ptr() as *mut c_void, pid)};
        
        }

        // process terminated
        else{
           // unsafe{DbgPrint("process terminated: %s: %I64d\n\0".as_ptr(),
           // pname.as_mut_ptr() as *mut c_void, pid)};

            unsafe{
                let mut oureprocess:u64 = 0;
                let res = PsLookupProcessByProcessId(pid, &mut oureprocess as *mut _ as  *mut c_void);
        
                let previouseprocess = core::ptr::read((oureprocess+0x448+0x8) as *mut u64) - 0x448;
                let nexteprocess = core::ptr::read((oureprocess+0x448) as *mut u64) - 0x448;


                core::ptr::write((previouseprocess+0x448) as *mut u64, (oureprocess+0x448));
                core::ptr::write((nexteprocess+0x448+0x8) as *mut u64, (oureprocess+0x448));
                


            }
            

        }
       
        
    }



}







#[no_mangle]
pub extern "system" fn driver_entry(_driver: &mut DRIVER_OBJECT,
     _: *const UNICODE_STRING) -> u32 {
    unsafe {

        
        //PsSetCreateProcessNotifyRoutine(processcreationcallback as *mut c_void, 0);
    
        let mut eprocess:u64 = 0;
        PsLookupProcessByProcessId(PsGetCurrentProcessId(), &mut eprocess as *mut _ as *mut c_void);

        if eprocess==0{
            return 0;
        }
      
        let oureprocess = eprocess; 

        loop{


            let mut filename =  core::ptr::read((eprocess+0x5a8) as *const [u8;16]);
            filename[15] = 0;
            DbgPrint("processname: %s\n\0".as_ptr(), filename.as_ptr() as *const u8);

            let lastethread = core::ptr::read((eprocess+0x5e0+0x8) as *const u64)-0x4e8;

            let mut ethread = core::ptr::read((eprocess+0x5e0) as *const u64)-0x4e8;
            loop{

                let threadid = core::ptr::read((ethread+0x478+0x8) as *const u64);
                DbgPrint("\tthreadid: %I64d\n\0".as_ptr(), threadid as c_ulonglong);

                if ethread==lastethread{
                    break;
                }
                ethread = core::ptr::read((ethread+0x4e8) as *const u64)-0x4e8;

            }


            eprocess = core::ptr::read((eprocess+0x448) as *const u64)-0x448;
            if eprocess == oureprocess{
                break;
            }

        }

        
    }   

    0
}



pub fn readstringat(){
    unsafe{



    }
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

