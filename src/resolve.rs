use crate::{
    request::{Request, ResolveRequest, Wrappable, Wrapper},
    Error, Result,
};
use crossbeam_channel::{Receiver, Sender};

/// An handle for submitting Requests to an actor for processing
#[derive(Debug)]
pub struct Agent<T> {
    pub(crate) tx: Sender<Wrapper<T>>,
}

// NOTE: implemented by hand to avoid requiring T to be Clone
impl<T> Clone for Agent<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<T> Agent<T> {
    /// Send a request to the Actor for it to resolve and return the result
    pub fn send<R>(&self, req: R) -> Result<R>
    where
        R: Wrappable<T>,
    {
        let (wrapped, rx) = req.into_wrapped_pair();
        self.send_inner::<R>(wrapped, rx)
    }

    /// Shut down the associated Actor.
    ///
    /// This is a no-op if the actor has already been shut down.
    pub fn shutdown(self) {
        if self.tx.send(Wrapper::ShutDown).is_err() {
            // channel already closed
        }
    }

    fn send_inner<R>(&self, wrapped: T, rx: Receiver<Result<R>>) -> Result<R>
    where
        R: Request + 'static,
    {
        if self.tx.send(Wrapper::Request(wrapped)).is_err() {
            return Err(Error::SendOnClosedChannel);
        }

        match rx.recv() {
            Ok(res) => res,
            Err(_) => Err(Error::ActorGone),
        }
    }
}

impl<R, T> ResolveRequest<R> for Agent<T>
where
    R: Wrappable<T>,
{
    fn resolve_request(&mut self, req: R) -> Result<R> {
        self.send(req)
    }
}

pub trait Handle<U> {
    /// Send a request to the Actor for it to resolve and return the result
    fn handle<R>(&self, req: R) -> Result<R>
    where
        R: Wrappable<U>,
    {
        let (wrapped, rx) = req.into_wrapped_pair();
        self.map_and_resolve::<R>(wrapped, rx)
    }

    fn map_and_resolve<R>(&self, wrapped: U, rx: Receiver<Result<R>>) -> Result<R>
    where
        R: Request + 'static;
}

impl<T, U> Handle<U> for Agent<T>
where
    U: Into<T>,
{
    fn map_and_resolve<R>(&self, wrapped: U, rx: Receiver<Result<R>>) -> Result<R>
    where
        R: Request + 'static,
    {
        self.send_inner::<R>(wrapped.into(), rx)
    }
}
