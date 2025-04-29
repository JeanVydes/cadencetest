use std::fmt::Debug;

pub fn trace_err<E: Debug>(msg: &'static str) -> impl Fn(E) -> E {
    move |e| {
        tracing::trace!("{}: {:?}", msg, e);
        e
    }
}