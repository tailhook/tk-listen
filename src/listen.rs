pub struct Listen<S> {
    stream: S,
}

pub fn new<S>(stream: S, limit: usize) -> Listen<S> {
    Listen {
        stream: stream,
    }
}
