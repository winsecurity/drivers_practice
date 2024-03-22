pub fn unicodetostring(u:&UNICODE_STRING) -> [u8;1024] {

    let mut buffer:[u8;1024] = [0;1024];

    for i in 0..u.Length/2{
        let mut u16byte: u16 = 0;
        let mmcopy = MM_COPY_ADDRESS{address: (u.Buffer as usize + (i as usize*2)) as *mut c_void} ;
        let mut byteswritten = 0;
        let res = unsafe{MmCopyMemory(&mut u16byte as *mut _ as *mut c_void, 
            mmcopy, 
            2, 
            0x2, 
            &mut byteswritten)};

        buffer[i as usize] = (u16byte&0xFF) as u8;    
    }
    
    return buffer;
    

}
