use core::future::Future;

pub trait Button {
    fn wait_for_press(&mut self) -> impl Future<Output = ()>;

    fn wait_for_release(&mut self) -> impl Future<Output = ()>;
}
