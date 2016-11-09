//! Request
//!

//use frame::headers::HeaderEntry;

struct Request {
    headers: Vec<String>,
}

impl Request {
    pub fn new() -> Self {
        let mut v = Vec::with_capacity(1);
        v.push("test".to_string());
        Request { headers: v }
    }
}

#[cfg(test)]
mod request_tests {
    use super::Request;

    #[test]
    fn request_create() {
        let req = Request::new();
        assert_eq!("test", req.headers[0]);
    }
}
