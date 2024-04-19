pub(crate) mod packet;

pub(crate) trait Looper {
    fn loop_forever(&mut self);
}
