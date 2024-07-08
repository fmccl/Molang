pub trait State<In, Out, Error> {
    fn handle(
        &mut self,
        c: Option<In>,
    ) -> Result<(Option<Out>, Option<Box<dyn State<In, Out, Error>>>, SequenceAction), Error>;
}

pub enum SequenceAction {
    Advance,
    Done,
    Hold,
}