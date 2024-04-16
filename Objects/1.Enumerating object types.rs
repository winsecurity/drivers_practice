#[no_mangle]
pub extern "system" fn driver_entry(driverobject: &mut DRIVER_OBJECT,
     registrypath: *const UNICODE_STRING) -> u32 {
    unsafe {

        
        let eprocess = PsGetCurrentProcess();
        if eprocess.is_null(){
            return 0;
        }
        let objtype = ObGetObjectType(eprocess);
        DbgPrint("obgetobjecttype address: %I64x\n\0".as_ptr(), ObGetObjectType as c_ulonglong);
        
        // nt!obgetobjecttype + 0x5f4350 = nt!obtypeindextable
        
        let indextable = ObGetObjectType as u64 +0x5f4350;
        let mut i = 2;
        if core::ptr::read(indextable as *const u64)==0{
            loop{
                let objecttypeaddress = core::ptr::read((indextable+(i*8)) as *const u64);
                let index = core::ptr::read(((objecttypeaddress+0x28) as *const u32));
                DbgPrint("Object type: %wZ, index: %I32d\n\0".as_ptr(),(objecttypeaddress+0x10), index );
                i+=1;
                if core::ptr::read((indextable+(i*8)) as *const u64)==0{
                    break;
                }
            }
        }
         
        //DbgPrint("object type: %wZ\n\0".as_ptr(), (objtype+0x10));
        driverobject.DriverUnload = core::mem::transmute(driver_unload as *mut c_void);

    }   

    0
}
