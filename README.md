# A minimal actor framework for handling messages

### But...why?

In writing Penrose I've ended up with a truely horrifying trait: `XConn`. I need
to be able to swap out talking to a _real_ x-server for something that I can use
in tests (and CI) but the way that the X11 APIs works means that I end up with
a trait that has dozens of methods. I tried breaking it down into smaller traits
which are based on related functionality and then compose those together, but
in practice most of the methods in the window manager only really need a handful
of the functionality on offer.

So, as a bit of a "what if?" experiment, lets try something completely different!

I've no idea what the performance of this will be like yet and it almost certainly
shouldn't be used for anything important. If making use of it ends up being an
improvement over the current "mega-trait" approach then I'll probably start tinkering
a bit and see what I can do to improve things.
