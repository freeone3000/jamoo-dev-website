use std::io::Write;
use std::str::FromStr;
use iron::{IronResult, Request, Response};
use iron::headers::{ContentType, Encoding};
use iron::mime::{Mime, SubLevel, TopLevel};
use iron::response::WriteBody;
use crate::negotation::Negotiation;

pub(crate) struct CompressMiddleware {}
impl CompressMiddleware {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
impl iron::middleware::AroundMiddleware for CompressMiddleware {
    fn around(self, handler: Box<dyn iron::Handler>) -> Box<dyn iron::Handler> {
        Box::new(CompressHandler {
            handler,
        })
    }
}

struct CompressHandler {
    handler: Box<dyn iron::Handler>,
}
impl iron::Handler for CompressHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut orig_resp = self.handler.handle(req)?;
        let compress = match orig_resp.headers.iter().find(|header| header.name() == "Content-Type") {
            Some(header) => match header.value::<ContentType>() {
                Some(content_type) => match content_type {
                    ContentType(Mime(TopLevel::Text, _, _)) => true,
                    ContentType(Mime(TopLevel::Application, SubLevel::Javascript, _)) => true,
                    ContentType(Mime(TopLevel::Application, SubLevel::Json, _)) => true,
                    ContentType(Mime(TopLevel::Application, SubLevel::Xml, _)) => true,
                    ContentType(Mime(TopLevel::Application, sl, _)) => sl.as_str() == "xhtml+xml",
                    _ => false,
                }
                None => false,
            },
            None => false,
        };

        // grab the compression via content-negotiation
        if compress {
            if let Some(encoding) = req.headers.iter().find(|h| h.name() == "Accept-Encoding") {
                let negotiation = Negotiation::parse(encoding.value_string());
                if let Some(encoding) = negotiation.best(&["gzip", "deflate"]) {
                    let orig_body = orig_resp.body;
                    orig_resp.headers.set(iron::headers::ContentEncoding(vec![Encoding::from_str(encoding).unwrap()]));

                    let mut buf = Vec::new();
                    CompressedResponse::new(encoding.clone(), orig_body).write_body(&mut buf)
                        .map_err(|_| iron::IronError::new(std::io::Error::new(std::io::ErrorKind::Other, "Error compressing response"), iron::status::InternalServerError))?;
                    orig_resp.headers.set(iron::headers::ContentLength(buf.len() as u64));
                    orig_resp.body = Some(Box::new(buf));
                }
            }
        }
        Ok(orig_resp)
    }
}
struct CompressedResponse {
    encoding: String,
    orig_body: Option<Box<dyn WriteBody>>,
}
impl CompressedResponse {
    fn new(encoding: String, orig_body: Option<Box<dyn WriteBody>>) -> Self {
        Self {
            encoding,
            orig_body,
        }
    }
}
impl WriteBody for CompressedResponse {
    fn write_body(&mut self, res: &mut dyn Write) -> std::io::Result<()> {
        let mut encoded_writer: Box<dyn Write> = match self.encoding.as_str() {
            "gzip" => Box::new(flate2::write::GzEncoder::new(res, flate2::Compression::fast())),
            "deflate" => Box::new(flate2::write::DeflateEncoder::new(res, flate2::Compression::fast())),
            "zlib" => Box::new(flate2::write::ZlibEncoder::new(res, flate2::Compression::fast())),
            _ => panic!("Unknown compression type: {}", self.encoding),
        };
        match &mut self.orig_body {
            Some(orig_body) => orig_body.write_body(&mut encoded_writer),
            None => Ok(()),
        }
    }
}
