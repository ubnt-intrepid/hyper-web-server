extern crate hyper;

use std::path::{Path, PathBuf};
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri;

fn main() {
    let bind = "0.0.0.0";
    let port = 8080;
    let address = format!("{}:{}", bind, port);

    let s = Server::http(address).expect("failed to create HTTP server");
    s.handle(handler).expect("failed to handle");
}

struct MyRequest {
    uri: String,
    path: PathBuf,
}

impl MyRequest {
    fn new(uri: String) -> MyRequest {
        let path = std::env::current_dir().unwrap().join(&uri[1..]);
        MyRequest {
            uri: uri,
            path: path,
        }
    }
}

fn handler(req: Request, res: Response) {
    match req.uri {
        RequestUri::AbsolutePath(ref s) if s == "/favicon.ico" => (),
        RequestUri::AbsolutePath(s) => {
            let req = MyRequest::new(s);
            if req.path.is_file() {
                file_handler(req, res)
            } else {
                dir_handler(req, res)
            }
        }
        _ => (),
    }
}

fn file_handler(req: MyRequest, mut res: Response) {
    let mut body = Vec::new();
    read_content(&req.path, &mut body)
        .and_then(|_| {
            res.headers_mut()
               .set(hyper::header::ContentType::plaintext());
            res.send(&body)
        })
        .unwrap_or(())
}

fn dir_handler(req: MyRequest, mut res: Response) {
    let to_li = |e| {
        let p = format!("{}/{}", &req.uri[1..], e);
        format!("<li><a href=\"{}\">{}</a></li>", p, e)
    };

    read_entries(&req.path)
        .and_then(|entries| {
            let entries = if req.uri != "/" {
                vec![
                    format!("<li><a href=\"{}\">..</a></li>",
                            PathBuf::from(&req.uri).parent().unwrap().display()),
                ]
                    .into_iter()
                    .chain(entries.into_iter().map(to_li))
                    .fold(String::new(), |mut acc, s| {
                        acc.push_str(&s);
                        acc
                    })
            } else {
                entries.into_iter()
                       .map(to_li)
                       .fold(String::new(), |mut acc, s| {
                    acc.push_str(&s);
                    acc
                })
            };

            let body = format!(r#"
    <html>
    <head>
      <title>{}</title>
    </head>
    <body>
      <ul>{}</ul>
    </body>
    </html>
    "#,
                               req.uri,
                               entries);
            res.headers_mut().set(hyper::header::ContentType::html());
            res.send(body.as_bytes())
        })
        .unwrap_or(())
}


fn read_content<P, W>(path: P, w: &mut W) -> std::io::Result<()>
    where P: AsRef<Path>,
          W: std::io::Write
{
    std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .and_then(|mut f| std::io::copy(&mut f, w))
        .map(|_| ())
}

fn read_entries<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<String>> {
    let entries = std::fs::read_dir(path)?;
    let entries =
        entries.flat_map(Result::ok)
               .map(|e| e.file_name().to_string_lossy().into_owned())
               .collect();
    Ok(entries)
}
