use crossbeam_channel::Sender;

pub type Result<R> = std::result::Result<<R as Request>::Output, <R as Request>::Error>;

/// A request that can be handled by an Actor. For an Actor to be able to
/// resolve a given [`Request`] it must implement [`Resolve`] for it.
pub trait Request: Sized + Send {
    /// The happy path response type
    type Output: Send;
    /// The error path response type
    type Error: Send;
}

/// Register the ability for an Actor to resolve a certain type of [`Request`]
pub trait Resolve<R: Request> {
    /// Resolve a request using this Actor
    fn resolve(&mut self, req: R) -> Result<R>;
}

pub trait ResolveRequest<R: Request> {
    fn resolve_request(&mut self, req: R) -> crate::Result<R>;
}

// impl<R, T> ResolveRequest<R> for T
// where
//     R: Request,
//     T: Resolve<R>,
// {
//     fn resolve_request(&mut self, req: &mut R) -> crate::Result<R> {
//         match self.resolve(req) {
//             Ok(res) => Ok(res),
//             Err(e) => Err(crate::Error::Resolve(e)),
//         }
//     }
// }

/// A message for an actor
#[derive(Debug)]
pub struct Message<R: Request> {
    /// The [`Request`] being sent
    pub req: R,
    /// A channel for sending back the response
    pub tx: Sender<crate::Result<R>>,
}

/// Message wrapper for a given request type
#[derive(Debug)]
pub enum Wrapper<T> {
    /// A [`Request`] for processing
    Request(T),
    /// Signal that the owner of the communication channel being used should shut down
    ShutDown,
}
