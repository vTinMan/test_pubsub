pub mod state_tracker;
use state_tracker::*;

use log::{debug, info};
use async_std::{ task, net::TcpStream, io::ReadExt };
use futures::{ executor::block_on, AsyncWriteExt };
use std::{ time, collections::linked_list::LinkedList };

static PAUSE_INT: u64 = 50;

pub async fn handle_stream(mut stream: TcpStream) {
    task::spawn(async move {
        let res = unpack_request(&stream);
        match res {
            Ok(req_data) => {
                info!("Request accepted {} {}", req_data.method, req_data.path);
                let resp_data = process_request(&req_data);
                let _ = block_on(stream.write_all(resp_data.as_bytes()));
            },
            Err(error_kind) => {
                let err_msg = error_kind.to_string();
                info!("Bad request: {}", err_msg);
                let _ = block_on(stream.write_all(("HTTP/1.1 400 Bad Request\r\n\r\n".to_string()
                                                   + &err_msg).as_bytes()));
            }
        }
    });
}


fn process_request(req_data: &RequestData)-> String {
    if req_data.method == "GET".to_string() {
        wait_for_message(&req_data.path)
    } else {
        handle_message(req_data)
    }
}


fn handle_message(req_data: &RequestData)-> String {
    let session = fetch_session_id(&req_data.path);
    info!("Path session id {} => {}", &req_data.path, session);
    finalize_session(&session, &req_data.message);
    "HTTP/1.1 200 OK\r\n\r\n".to_string()
}


fn wait_for_message(path: &String)-> String {
    let pause_time = time::Duration::from_millis(PAUSE_INT);
    let session_id = fetch_session_id(&path);
    info!("Path session id {} => {}", path, session_id);

    for _ in 0..200 {
        let session = get_session(&session_id);
        match session {
            Some(SessionData{ state: StateKind::Done, message: Some(message), ..}) => {
                return "HTTP/1.1 200 OK\r\n\r\n".to_string() + &message + "\r\n"
            },
            Some(SessionData{ state: StateKind::Done, ..}) => {
                return "HTTP/1.1 200 OK\r\n\r\n".to_string()
            },
            Some(SessionData{ state: StateKind::Pending, ..}) => {
                block_on(task::sleep(pause_time))
            },
            _ => {
                return "HTTP/1.1 422 Unprocessable Entity\r\n\r\n".to_string()
            }
        }
    }
    "HTTP/1.1 408 Request Timeout\r\n\r\n".to_string()
}


struct RequestData {
    method: String,
    path: String,
    message: String
}

fn unpack_request(mut stream: &TcpStream) -> Result<RequestData, String> {
    let mut buf = vec![0; 4096];
    let len = block_on(stream.read(&mut buf)).or(Err("Stream cannot be read"))?;

    let s = std::str::from_utf8(&buf).or(Err("Buffer error"))?;
    let mut lines: LinkedList<String> = s[..len].lines().map(String::from).collect();

    let head_line = lines.pop_front().ok_or("No headline")?;
    let mut req_header_data: LinkedList<String> = head_line.split(" ").map(String::from).collect();
    let method = req_header_data.pop_front().ok_or("No method")?;
    debug!("  Request:");
    debug!("    method: {}", method);
    let path = req_header_data.pop_front().ok_or("No path")?;
    debug!("    path: {}", path);
    let message = lines.pop_back().unwrap_or("".to_string());
    debug!("    message: {}", message);
    for line in &lines { debug!("    {}", line); }

    Ok(RequestData {
        method: method,
        path: path,
        message: message
    })
}
