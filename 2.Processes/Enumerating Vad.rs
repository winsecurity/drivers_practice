
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


#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
struct rtl_balanced_node{
    left: u64,
    right: u64,
    parentvalue: u64
}





pub fn traverse(rootnode: rtl_balanced_node){
    unsafe{

          // Vad - 56 61 64 20
        // VadS - 56 61 64 53


        if rootnode.left!=0{
            DbgPrint("left node: %I64x\n\0".as_ptr(), rootnode.left as c_ulonglong);
            
            // enumerating the node mmvad or mmvad_short
            
            
            vadcount +=1;
            traverse(*(rootnode.clone().left as *mut rtl_balanced_node));
        }

        if rootnode.right!=0{
            DbgPrint("right node: %I64x\n\0".as_ptr(), rootnode.right as c_ulonglong);
            vadcount +=1;
            traverse(*(rootnode.clone().right as *mut rtl_balanced_node));

        }

    }
}

static mut vadcount:u32 = 0;

#[no_mangle]
pub extern "system" fn driver_entry(_driver: &mut DRIVER_OBJECT,
     _: *const UNICODE_STRING) -> u32 {
    unsafe {

        
        //PsSetCreateProcessNotifyRoutine(processcreationcallback as *mut c_void, 0);
    
        let mut eprocess:u64 = 0;
        PsLookupProcessByProcessId(9168 as HANDLE, &mut eprocess as *mut _ as *mut c_void);

        if eprocess==0{
            return 0;
        }
    
        // eprocess+0x440 = uniqueprocessid (8 bytes)
        // eprocess+0x5a8 = imagefilename (15bytes)
        // eprocess+0x448 = activeprocesslinks (_list_entry)
        // eprocess+0x5e0 = threadlisthead (_list_entry)
        // ethread+0x4e8 = threadlistentry
        // ethread+0x478 = Cid = (processid, threadid)

        // eprocess+7d8 = vadroot
        let mut firstpid:u64 = core::ptr::read((eprocess+0x440) as *const u64);
        
        /*let mut vadroot:u64 = 0;
        let mmcopy = MM_COPY_ADDRESS{address:(vadroot + 0x7d8) as *mut c_void};
        let mut byteswritten = 0;
        let res = MmCopyMemory(&mut vadroot as *mut _ as *mut c_void, 
            mmcopy, 8, 0x2, &mut byteswritten);
        
        DbgPrint("result: %I64x, vadroot: %I64x\n\0".as_ptr(),
        res as c_uint, vadroot as c_ulonglong);       */


        let vadroot = core::ptr::read((eprocess+0x7d8) as *const u64);
        DbgPrint("vadroot: %I64x\n\0".as_ptr(), vadroot as c_ulonglong);

        
        let rootnode = *(vadroot  as *mut rtl_balanced_node);
        
        // preorder traversal left
        //traverseleft(rootnode.clone());
        /*let mut tempnode = rootnode;
        let mut backtrack = 0;
        let mut parentnode = core::mem::zeroed::<rtl_balanced_node>();

        let mut isvisited = 2;*/

        // Vad - 56 61 64 20
        // VadS - 56 61 64 53


       
        traverse(rootnode);
        DbgPrint("total vadcount: %I64d\n\0".as_ptr(), vadcount as c_int);
       
        //DbgPrint("left: %I64x\t right: %I64x\t parent: %I64d\n\0".as_ptr(),
        //rootnode.left as c_ulonglong,rootnode.right as c_ulonglong, rootnode.parentvalue as c_ulonglong );   
       

        
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

