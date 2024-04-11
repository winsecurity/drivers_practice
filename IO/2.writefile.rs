
#![no_std]
#![allow(unused_imports)]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;


use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntioapi::FILE_DEVICE_SECURE_OPEN;
use ntapi::ntpsapi::ZwCurrentProcess;
use winapi::km::wdm::*;
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
    pub fn RtlInitUnicodeString(outunicode:*mut UNICODE_STRING, pu16:*const u16);
    pub fn IoCreateDevice(pdriver: *const DRIVER_OBJECT, extension:u32, dname:*const UNICODE_STRING,
        devicetype:u32, devicecharacteristics:u32, exclusive:u8,pdeviceobject: *mut *mut DEVICE_OBJECT ) -> i32;
    pub fn IoDeleteDevice(pdriverobject: *mut DEVICE_OBJECT);
    pub fn IoCreateSymbolicLink(linkname:*const UNICODE_STRING,devicename: *const UNICODE_STRING) -> i32;
    pub fn IoDeleteSymbolicLink(linkname:*const UNICODE_STRING) -> i32;
    pub fn IofCompleteRequest(pirp: *const IRP, priorityboost:i8 );
    //pub fn MmGetSystemAddressForMdlSafe(mdl: *const c_void, pagepriority:i32) -> *mut c_void;
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






#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
struct myunicodestring{
    size:u16,
    maxsize:u16,
    buffer:usize
}



enum IRPMJ{
    IRP_MJ_CREATE = 0,
    IRP_MJ_CLOSE = 2,
    IRP_MJ_READ = 3,
    IRP_MJ_WRITE = 4,
    IRP_MJ_CLEANUP = 18
}





#[no_mangle]
pub extern "system" fn iocreateclose(deviceobject: &mut DEVICE_OBJECT,irp: &mut IRP) -> i32{
    unsafe{

        let piostack = IoGetCurrentIrpStackLocation(irp);

        let majorfunction = core::ptr::read(piostack as *const u8);

        match majorfunction{
            0 => {
                DbgPrint("someone opened handle to our symbolic link\n\0".as_ptr());
            }
            2=>{
                DbgPrint("someone close handle to our symbolic link\n\0".as_ptr());

            }
            _ => {}
        }

        0

    }
}




#[no_mangle]
pub extern "system" fn iowrite(deviceobject: &mut DEVICE_OBJECT,irp: &mut IRP) -> i32{
    unsafe{

        let piostack = IoGetCurrentIrpStackLocation(irp);

        let mut nbytes=0;
        if !piostack.is_null(){
            nbytes = core::ptr::read((piostack as *mut _ as usize+0x8) as *const u64);

        }

        // reading mdl address
        //let mdl = core::ptr::read((irp as *const _ as usize+0x8) as *const u64);
        // DIRECT IO
        // we get system virtual address for corresponding userprocess's buffer's address
        //let mut startingva = core::ptr::read((mdl + 0x18 ) as *const u64 );
        //startingva += core::ptr::read((mdl + 0x2c ) as *const u32 ) as u64;

        //DbgPrint("contents: %s\n\0".as_ptr(), startingva as *const u8);


        // BUFFERED IO
        //let sysbuffer = core::ptr::read((irp as *mut _ as usize+0x18) as *const u64);

        //DbgPrint("contents: %s\n\0".as_ptr(),sysbuffer as *const u8);


        // irp + 0x30 - iostatus block
        // first member of iostatus block is STATUS
        // +0x8 is Information - number of bytes
        core::ptr::write((irp as *mut _ as usize+0x30) as *mut u64, 0);
        core::ptr::write((irp as *mut _ as usize+0x30+0x8) as *mut u64, nbytes);
        IofCompleteRequest(irp, 0);

        return STATUS_SUCCESS;

    }
}




#[no_mangle]
pub extern "system" fn driver_entry(_driver: &mut DRIVER_OBJECT,
     registrypath: *const UNICODE_STRING) -> u32 {
    unsafe {

        
       _driver.DriverUnload = core::mem::transmute(driver_unload as *mut c_void);
        
        

        let dname = obfstr::wide!("\\Device\\mydevice\0");
        let mut devicename = core::mem::zeroed::<UNICODE_STRING>();
        RtlInitUnicodeString(&mut devicename, dname.as_ptr() as *const u16);

        let mut deviceobject = 0 as *mut DEVICE_OBJECT;
        let res = IoCreateDevice(_driver, 0, 
            &devicename, 
            0x00000022, 
            FILE_DEVICE_SECURE_OPEN , 
            0, 
           &mut deviceobject );

        if res==STATUS_SUCCESS{

            (*deviceobject).Flags |= 0x00000010;
            
            DbgPrint("Device created\n\0".as_ptr());

            let dname1 = obfstr::wide!("\\DosDevices\\mydevice69\0");
            let mut dosname = core::mem::zeroed::<UNICODE_STRING>();
            RtlInitUnicodeString(&mut dosname, dname1.as_ptr() as *const u16);
    
            let res2 = IoCreateSymbolicLink(&dosname, &devicename);
            if res2==STATUS_SUCCESS{
                DbgPrint("Symbolic link created\n\0".as_ptr());


                _driver.MajorFunction[IRPMJ::IRP_MJ_CREATE as usize] = core::mem::transmute(iocreateclose as *mut c_void);
                _driver.MajorFunction[IRPMJ::IRP_MJ_CLOSE as usize] = core::mem::transmute(iocreateclose as *mut c_void);
                _driver.MajorFunction[IRPMJ::IRP_MJ_WRITE as usize] = core::mem::transmute(iowrite as *mut c_void);
                
            }


        }
        
    }   

    0
}



#[no_mangle]
pub extern "system" fn driver_unload(_driver:&mut DRIVER_OBJECT){
    unsafe{
        DbgPrint("Driver unloaded\n\0".as_ptr());

        let dname1 = obfstr::wide!("\\DosDevices\\mydevice69\0");
        let mut dosname = core::mem::zeroed::<UNICODE_STRING>();
        RtlInitUnicodeString(&mut dosname, dname1.as_ptr() as *const u16);
        IoDeleteSymbolicLink(&dosname);

        IoDeleteDevice(_driver.DeviceObject);



    }
}




pub fn unicodetostring(u:&mut UNICODE_STRING) -> [u8;2048] {

    let mut buffer:[u8;2048] = [0;2048];

    for i in 0..u.MaximumLength/2{
        let mut u16byte: u16 = 0;
        let mmcopy = MM_COPY_ADDRESS{address: (u.Buffer as usize + (i as usize*2)) as *mut c_void} ;
        let mut byteswritten = 0;
        let res = unsafe{MmCopyMemory(&mut u16byte as *mut _ as *mut c_void, 
            mmcopy, 
            2, 
            0x2, 
            &mut byteswritten)};

        if res==STATUS_SUCCESS{
            if u16byte==0{
                break;
            }
            buffer[i as usize] = (u16byte&0xFFFF) as u8;  
        }
          
    }
    
    return buffer;
    

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
