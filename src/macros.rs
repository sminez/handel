/// Quickly implement [`Request`] for a given type
#[macro_export]
macro_rules! request {
    ($req:ident => $res:ty) => {
        request!($req => $res, ());
    };

    ($req:ident => $res:ty, $err:ty) => {
        impl $crate::Request for $req {
            type Output = $res;
            type Error = $err;
        }
    };
}

/// Group a set of [`Request`] types together for the purposes of establishing a communication
/// interface.
#[macro_export]
macro_rules! request_set {
    ($vis:vis enum $enum:ident; trait $trait:ident => [ $($req:ident),+ ]) => {
        $vis enum $enum {
            $($req($crate::Message<$req>),)+
        }

        #[allow(dead_code)]
        impl $enum {
            fn from_inner_message<T>(self) -> T
                where T: Sized $(+ From<$crate::Message<$req>>)+ {
                match self {
                    $(
                        $enum::$req(msg) => msg.into(),
                    )+
                }
            }
        }

        $vis trait $trait: Send + Sized + 'static $(+ $crate::Resolve<$req>)+ {
            fn resolve_enum(&mut self, msg: $enum) {
                match msg {
                    $(
                        $enum::$req($crate::Message { req, tx }) => {
                            let res = match self.resolve(req) {
                                Ok(res) => Ok(res),
                                Err(e) => Err($crate::Error::Resolve(e)),
                            };

                            if tx.send(res).is_err() {
                                // The client dropped their end of the channel
                            }
                        },
                    )+
                }
            }

            fn run_threaded(mut self) -> ($crate::Agent<$enum>, std::thread::JoinHandle<()>) {
                let (tx, rx) = crossbeam_channel::unbounded::<$crate::Wrapper<$enum>>();

                let handle = std::thread::spawn(move || loop {
                    let w = match rx.recv() {
                        Ok(w) => w,
                        Err(_) => break, // channel is now disconnected
                    };

                    match w {
                       $crate::Wrapper::Request(req) => self.resolve_enum(req),
                       $crate::Wrapper::ShutDown => break,
                    }
                });

                ($crate::Agent { tx }, handle)
            }
        }

        $(
            impl crate::Wrappable<$enum> for $req {
                fn into_wrapped_pair(self) -> ($enum, crossbeam_channel::Receiver<crate::Result<Self>>) {
                    let (tx, rx) = crossbeam_channel::bounded(1);

                    ($enum::$req($crate::Message { req: self, tx }), rx)
                }
            }

            impl From<$crate::Message<$req>> for $enum {
                fn from(msg: $crate::Message<$req>) -> $enum {
                    $enum::$req(msg)
                }
            }
        )+

        impl<T> $trait for T where T: Send + Sized + 'static $(+ Resolve<$req>)+ {}
    };
}

// Mark one request set as being a subset of another so that Agents for the parent
// can be used in place of the child
#[macro_export]
macro_rules! subset {
    ($parent:ident > $child:ident) => {
        impl From<$child> for $parent {
            fn from(child: $child) -> Self {
                child.from_inner_message()
            }
        }
    };
}
