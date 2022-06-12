use self::request::UnicomRequest;

pub mod request;
pub mod response;

#[derive(Debug)]
pub enum UnicomMessage{
    Request{
        id: u64,
        data: UnicomRequest,
    },
    Quit,
}

