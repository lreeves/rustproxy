extern crate regex;
extern crate time;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use regex::Regex;
use time::*;

enum Verb {
    GET,
    POST,
    OPTIONS,
    PUT,
    DELETE
}

// Hashmap would be a fit but headers can be duplicated
struct Header {
    key: String,
    value: String
}

struct Request {
    verb: String,
    hostname: String,
    path: String,
    protocol: String,
    client_headers: Vec<Header>
}

fn read_request(stream: &mut TcpStream) -> Request {
    let mut buf: [u8; 1024] = [0; 1024]; // if this is declared mutable here, why below too?
    let resp_bytes = b"HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\n\r\nOK";
    stream.read(&mut buf);

    let client_headers_buf = String::from_utf8_lossy(&mut buf);
    let mut request_iterator = client_headers_buf.split("\r\n");
    let request_line: &str = request_iterator.next().unwrap();

    let request_tokens: Vec<&str> = request_line.split(" ").collect();
    let url: &str = request_tokens[1];
    let version: &str = request_tokens[2];

    let re = Regex::new(r"(\w*?)://(.*?)/(.*)").unwrap();
    let caps = re.captures(url).unwrap();

    let mut request = Request {
        verb: request_tokens[0].to_string(),
        hostname: caps.at(2).unwrap().to_string(),
        path: caps.at(3).unwrap().to_string(),
        protocol: caps.at(1).unwrap().to_string(),
        client_headers: Vec::new()
    };

    // Not a big fan of this; would rather define the request once and somehow point the
    // structures vector at this one.
    for header in request_iterator { // iterator is already past request line
        if header.len() > 0 {
            let tokens: Vec<&str> = header.splitn(2, ": ").collect();
            if tokens.len() == 2 {
                request.client_headers.push(Header {
                    key: tokens[0].to_string(),
                    value: tokens[1].to_string()
                });
            }
        }
    }

    return request;
}

fn log_request(request: &Request) {
    let t = now();
    println!("[{}-{:02}-{:02} {:02}:{:02}:{:02}.{:04}] {} {} \"/{}\"",
             t.tm_year + 1900,
             t.tm_mon + 1,
             t.tm_mday,
             t.tm_hour,
             t.tm_min,
             t.tm_sec,
             t.tm_nsec,
             request.verb,
             request.hostname,
             request.path);
}

fn send_request(request: &Request, stream: &mut TcpStream) {

    // Send actual request
    let request_line = format!("{} /{} HTTP/1.1\r\n", request.verb, request.path);
    stream.write(&request_line.into_bytes());

    // Send all client headers
    for header in request.client_headers.iter() {
        let outbound_header = format!("{}: {}\r\n", header.key, header.value);
        stream.write(&outbound_header.into_bytes());
    }

    stream.write(b"Connection: close\r\n");
    stream.write(b"\r\n");
}

fn handle_client(mut client_stream: TcpStream) {
    let request = read_request(&mut client_stream);
    log_request(&request);

    let address_string = format!("{}:{}", request.hostname, 80);
    let mut server_stream = TcpStream::connect(&*address_string).unwrap();
    send_request(&request, &mut server_stream);

    // Pass through the reads to the client
    let mut content_buffer: Vec<u8> = Vec::new();
    let content_size = server_stream.read_to_end(&mut content_buffer).unwrap();
    client_stream.write(&content_buffer);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3128").unwrap();

    for stream in listener.incoming() {
        match stream {
            Err(_) => { /* connection failed */ }
            Ok(stream) => {
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream)
                });
            }
        }
    }

    // close the socket server
    drop(listener);
}
