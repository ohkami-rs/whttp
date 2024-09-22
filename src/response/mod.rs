mod status;

pub use status::Status;
use crate::Headers;

pub struct Response {
    status:  Status,
    headers: Headers,
}
