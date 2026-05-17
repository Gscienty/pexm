use y_octo::{Awareness, Doc};

pub trait Context {
    fn get_document_name(&self) -> &str;

    fn get_document(&self) -> &Doc;
    fn get_document_mut(&mut self) -> &mut Doc;

    fn get_awareness(&self) -> &Awareness;
    fn get_awareness_mut(&mut self) -> &mut Awareness;

    async fn unicast(&self, msg: Vec<u8>);
    async fn broadcast(&mut self, msg: &[u8]);

    async fn close(&mut self);
}
