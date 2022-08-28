//! A simple, lightweight actor framework
use tracing::error;

#[macro_use]
mod macros;
pub mod request;
pub mod resolve;

pub use request::{Message, Request, Resolve, ResolveRequest, Wrapper};
pub use resolve::{Agent, Handle};

/// An error encountered while interacting with an Actor
#[derive(Debug, thiserror::Error)]
pub enum Error<E> {
    /// An error was returned from the Actor while resolving the [`Request`]
    #[error("error while running resolver")]
    Resolve(E),

    /// The channel for communicating with the Actor is closed
    #[error("attempt to send on closed channel to the Actor")]
    SendOnClosedChannel,

    /// The channel for receiving the response from the Actor is closed
    #[error("actor closed the response channel")]
    ActorGone,
}

pub type Result<R> = std::result::Result<<R as Request>::Output, Error<<R as Request>::Error>>;

#[cfg(test)]
mod tests {
    use super::*;

    struct Add(usize, usize);
    request!(Add => usize);

    struct Concat(&'static str, &'static str);
    request!(Concat => String);

    struct Failure(bool);
    request!(Failure => &'static str, &'static str);

    struct TestActor;

    impl Resolve<Add> for TestActor {
        fn resolve(&mut self, req: Add) -> request::Result<Add> {
            Ok(req.0 + req.1)
        }
    }

    impl Resolve<Concat> for TestActor {
        fn resolve(&mut self, req: Concat) -> request::Result<Concat> {
            Ok(format!("{}{}", req.0, req.1))
        }
    }

    impl Resolve<Failure> for TestActor {
        fn resolve(&mut self, req: Failure) -> request::Result<Failure> {
            if req.0 {
                Err("failed")
            } else {
                Ok("succeeded")
            }
        }
    }

    request_set!(enum Example; trait ResolveExample => [Add, Concat, Failure]);

    #[test]
    fn requests_work() {
        let (agent, _) = ResolveExample::run_threaded(TestActor);

        let res = agent.send(Add(1, 2)).unwrap();
        assert_eq!(res, 3);

        let res = agent.send(Concat("hello,", " world!")).unwrap();
        assert_eq!(res, "hello, world!");
    }

    #[test]
    fn errors_are_returned_correctly() {
        let (agent, _) = ResolveExample::run_threaded(TestActor);

        let res = agent.send(Failure(false));
        assert!(res.is_ok());

        let res = agent.send(Failure(true));
        assert!(matches!(res, Err(Error::Resolve("failed"))));
    }

    #[test]
    fn send_to_actor_after_shutdown_is_an_error() {
        let (agent, thread_handle) = ResolveExample::run_threaded(TestActor);
        let cloned = agent.clone();

        agent.shutdown();
        thread_handle.join().unwrap();
        let res = cloned.send(Add(1, 2));

        assert!(matches!(res, Err(Error::SendOnClosedChannel)));
    }

    request_set!(enum Example2; trait ResolveExample2 => [Add, Concat]);
    subset!(Example > Example2);

    #[test]
    fn generic_handles_work() {
        fn generic1<H: Handle<Example2>>(h: &H) -> Result<Add> {
            h.handle(Add(1, 2))
        }

        fn generic2<H: Handle<Example2>>(h: &H) -> Result<Concat> {
            h.handle(Concat("hello,", " world!"))
        }

        let (agent, _) = ResolveExample::run_threaded(TestActor);

        let res = generic1(&agent).expect("to resolve");
        assert_eq!(res, 3);

        let res = generic2(&agent).expect("to resolve");
        assert_eq!(res, "hello, world!");
    }

    #[test]
    fn generic_resolvers_work() {
        fn generic<H>(mut h: H)
        where
            H: ResolveRequest<Add> + ResolveRequest<Concat>,
        {
            let n = h.resolve_request(Add(1, 2)).unwrap();
            assert_eq!(n, 3);

            let s = h.resolve_request(Concat("hello,", " world!")).unwrap();
            assert_eq!(s, "hello, world!");
        }

        let (agent, _) = ResolveExample::run_threaded(TestActor);

        generic(agent);
    }
}
