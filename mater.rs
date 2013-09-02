use std::vec;
use std::str;
use std::rt::uv::{Loop, AllocCallback, vec_from_uv_buf, vec_to_uv_buf};
use std::rt::uv::net::TcpWatcher;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::hashmap::HashMap;
use std::from_str::FromStr;


struct Response {
    content: ~str,
    status: uint,
}

impl Response {
    fn new(content: ~str, status: uint) -> Response {
        return Response { 
            content: content, 
            status: status,
        };
    }

    fn into_bytes(self) -> ~[u8] {

        let message = match self.status {
            100 => "CONTINUE",
            101 => "SWITCHING PROTOCOLS",
            102 => "PROCESSING",
            200 => "OK",
            201 => "CREATED",
            202 => "ACCEPTED",
            203 => "NON-AUTHORITATIVE INFORMATION",
            204 => "NO CONTENT",
            205 => "RESET CONTENT",
            206 => "PARTIAL CONTENT",
            207 => "MULTI-STATUS",
            208 => "ALREADY REPORTED",
            226 => "IM USED",
            300 => "MULTIPLE CHOICES",
            301 => "MOVED PERMANENTLY",
            302 => "FOUND",
            303 => "SEE OTHER",
            304 => "NOT MODIFIED",
            305 => "USE PROXY",
            306 => "RESERVED",
            307 => "TEMPORARY REDIRECT",
            400 => "BAD REQUEST",
            401 => "UNAUTHORIZED",
            402 => "PAYMENT REQUIRED",
            403 => "FORBIDDEN",
            404 => "NOT FOUND",
            405 => "METHOD NOT ALLOWED",
            406 => "NOT ACCEPTABLE",
            407 => "PROXY AUTHENTICATION REQUIRED",
            408 => "REQUEST TIMEOUT",
            409 => "CONFLICT",
            410 => "GONE",
            411 => "LENGTH REQUIRED",
            412 => "PRECONDITION FAILED",
            413 => "REQUEST ENTITY TOO LARGE",
            414 => "REQUEST-URI TOO LONG",
            415 => "UNSUPPORTED MEDIA TYPE",
            416 => "REQUESTED RANGE NOT SATISFIABLE",
            417 => "EXPECTATION FAILED",
            418 => "I'M A TEAPOT",
            422 => "UNPROCESSABLE ENTITY",
            423 => "LOCKED",
            424 => "FAILED DEPENDENCY",
            426 => "UPGRADE REQUIRED",
            428 => "PRECONDITION REQUIRED",
            429 => "TOO MANY REQUESTS",
            431 => "REQUEST HEADER FIELDS TOO LARGE",
            500 => "INTERNAL SERVER ERROR",
            501 => "NOT IMPLEMENTED",
            502 => "BAD GATEWAY",
            503 => "SERVICE UNAVAILABLE",
            504 => "GATEWAY TIMEOUT",
            505 => "HTTP VERSION NOT SUPPORTED",
            506 => "VARIANT ALSO NEGOTIATES",
            507 => "INSUFFICIENT STORAGE",
            508 => "LOOP DETECTED",
            510 => "NOT EXTENDED",
            511 => "NETWORK AUTHENTICATION REQUIRED",
            _   => "UNKNOWN"
        };

        let response_segments = [
            fmt!("HTTP/1.1 %u %s", self.status, message),
            fmt!("Content-Length: %u", self.content.len()),
            ~"",
            self.content.clone(),
            ~"",
        ];

        return response_segments.connect("\n").into_bytes();
    }
}

struct Request { 
    content: ~str,
    path: ~str,
    method: ~str,
    headers: HashMap<~str,~str>
}

impl Request {
    fn new(body: ~str) -> Request { 
        let mut path = ~"/";
        let mut method = ~"GET";
        let mut content = ~"";
        let mut headers: HashMap<~str,~str> = HashMap::new();
            
        for (line_number, line_content) in body.split_str_iter("\r\n\r\n").enumerate() {
            match line_number {
                0 => {
                    for (header_line, header_content) in line_content.split_str_iter("\r\n")
                        .enumerate() {
                        match header_line {
                            0 => {
                                let first_line: ~[&str] = header_content.split_iter(' ')
                                    .collect();
                                method = first_line[0].to_owned();
                                path = first_line[1].to_owned();
                            }
                            _ => {
                                let header: ~[&str] = header_content.split_str_iter(": ").collect();
                                headers.insert(header[0].to_owned(), 
                                               header[1].to_owned());
                            }
                        }
                    }
                }
                1 => {
                    content = line_content.to_owned();
                }
                _ => {}
            }
        }
        return Request { 
            content: content, 
            path: path, 
            method: method,
            headers: headers
        };
    }
}

struct View {
    path: ~str
}

impl View {
    fn new(path: ~str) -> View {
        return View {
            path: path
        }
    }
}

struct Application {
    loop_: Loop,
    socket: SocketAddr,
    // views: ~[~str]
}

impl Application {
    fn new(address: ~str) -> Application { 
        let address: Option<SocketAddr> = FromStr::from_str(address);
        
        let socket: SocketAddr = match address {
            Some(ref socket_address) => *socket_address,
            None => SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8000 }
        };

	return Application { 
	    socket: socket, 
            loop_: Loop::new(),
            // views: [View::new(~"/"), View::new(~"/test")],
            // views: [~"1234"]
        };
    }
    
    fn run(&self) { 
        let mut loop_ = self.loop_;
        
        let mut server_tcp_watcher = { TcpWatcher::new(&mut loop_) };
        server_tcp_watcher.bind(self.socket);

        let loop_ = loop_;

        do server_tcp_watcher.listen |mut server_stream_watcher, _ | {
            let mut loop_ = loop_;
            
            let client_tcp_watcher = TcpWatcher::new(&mut loop_);
            let mut client_tcp_watcher = client_tcp_watcher.as_stream();
            server_stream_watcher.accept(client_tcp_watcher);
            
            let alloc: AllocCallback = |size| {
                vec_to_uv_buf(vec::from_elem(size, 0u8))
            };
            
            do client_tcp_watcher.read_start(alloc) |mut stream_watcher, _ , buf, _ | {

                let buf = vec_from_uv_buf(buf);
                match buf {
                    Some(ref m) => {
                        let request_str = str::from_bytes(*m);
                        let request = Request::new(request_str.clone());

                        // []
                        [1,2,3,4].each |m| {
                            println("1");
                        };
                        
                        do stream_watcher.write(vec_to_uv_buf(Response::new(~"Hello World!", 200).into_bytes()))
                            |watcher, _ | {
                            watcher.close(||());
                        }

                    },
                    None => ()
                }
            }
        }
        
        let mut loop_ = loop_;
        loop_.run();
        loop_.close();
    }
}

fn main() {
    let application = Application::new(~"127.0.0.1:8000");
    application.run();
}
