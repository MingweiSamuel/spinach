// use std::pin::Pin;
// use std::task::{ Context, Poll };

// use tokio::stream::Stream;

// pub struct PipelineContext {
//     source_counter: u32,
// }
// impl SourceContext {
//     pub fn new() -> Self {
//         Self {
//             _priv: (),
//         }
//     }

//     pub fn create_source<'a, S: Stream>(&'a self, stream: S) -> Source<'a> {
//         Source {
//             ctx: self,
//         }
//     }
// }

// pub struct Source<'a> {
//     ctx: &'a PipelineContext,
// }

use std::task::{ Context, Poll };

pub trait Source {
    type Output;

    /// Looks like an async stream.
    fn poll_next(&mut self, cx: &mut Context<'_>)
        -> Poll<Option<Self::Output>>;
}

pub trait Pipe {
    type Input;
    type Output;

    fn process(&self, input: Self::Input) -> Option<Self::Output>;
}

pub trait Sink {
    type Input;

    fn receive(&self, input: Self::Input);
}

pub trait Pipeflow {
}

// (1) A morphism is a pipe.
// TODO: enforce that F is a morphism rather than any function.
pub struct MorphismPipe<A, B, F>
where
    F: Fn(A) -> Option<B>
{
    morphism: F,
    _phantom: std::marker::PhantomData<(A, B)>,
}

impl <A, B, F> MorphismPipe<A, B, F>
where
    F: Fn(A) -> Option<B>
{
    pub fn new(morphism: F) -> Self {
        Self {
            morphism: morphism,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl <A, B, F> Pipe for MorphismPipe<A, B, F>
where
    F: Fn(A) -> Option<B>
{
    type Input = A;
    type Output = B;

    fn process(&self, input: A) -> Option<B> {
        (self.morphism)(input)
    }
}



// (2) A source with a pipe attached is still a source.
pub struct PipeSource<S, A>
where
    S: Source,
    A: Pipe<Input = <S as Source>::Output>,
{
    source: S,
    pipe: A,
}

impl <S, A> PipeSource<S, A>
where
    S: Source,
    A: Pipe<Input = <S as Source>::Output>,
{
    pub fn new(source: S, pipe: A) -> Self {
        Self {
            source: source,
            pipe: pipe,
        }
    }
}

impl <S, A> Source for PipeSource<S, A>
where
    S: Source,
    A: Pipe<Input = <S as Source>::Output>,
{
    type Output = <A as Pipe>::Output;

    fn poll_next(&mut self, cx: &mut Context<'_>)
        -> Poll<Option<Self::Output>>
    {
        self.source.poll_next(cx)
            .map(|opt| opt.and_then(|x| self.pipe.process(x)))
    }
}



// (3) A sink with a pipe attached is still a sink.
pub struct PipeSink<A, S>
where
    A: Pipe,
    S: Sink<Input = <A as Pipe>::Output>,
{
    pipe: A,
    sink: S,
}

impl <A, S> PipeSink<A, S>
where
    A: Pipe,
    S: Sink<Input = <A as Pipe>::Output>,
{
    pub fn new(pipe: A, sink: S) -> Self {
        Self {
            pipe: pipe,
            sink: sink,
        }
    }
}

impl <A, S> Sink for PipeSink<A, S>
where
    A: Pipe,
    S: Sink<Input = <A as Pipe>::Output>,
{
    type Input = <A as Pipe>::Input;

    fn receive(&self, input: Self::Input) {
        if let Some(input) = self.pipe.process(input) {
            self.sink.receive(input);
        }
    }
}



// (4) Two pipes form a pipe.
pub struct LongPipe<A, B>
where
    A: Pipe,
    B: Pipe<Input = <A as Pipe>::Output>,
{
    pipe_a: A,
    pipe_b: B,
}

impl <A, B> LongPipe<A, B>
where
    A: Pipe,
    B: Pipe<Input = <A as Pipe>::Output>,
{
    pub fn new(pipe_a: A, pipe_b: B) -> Self {
        Self {
            pipe_a: pipe_a,
            pipe_b: pipe_b,
        }
    }
}

impl <A, B> Pipe for LongPipe<A, B>
where
    A: Pipe,
    B: Pipe<Input = <A as Pipe>::Output>,
{
    type Input = <A as Pipe>::Input;
    type Output = <B as Pipe>::Output;

    fn process(&self, input: Self::Input) -> Option<Self::Output> {
        self.pipe_a.process(input)
            .and_then(|v| self.pipe_b.process(v))
    }
}



// (5) Diverge pipe into sink.
pub struct SplitPipe<S, F>
where
    S: Sink,
    F: Fn(&<S as Sink>::Input) -> bool,
    <S as Sink>::Input: Clone,
{
    sink: S,
    filter: F,
}

impl <S, F> SplitPipe<S, F>
where
    S: Sink,
    F: Fn(&<S as Sink>::Input) -> bool,
    <S as Sink>::Input: Clone,
{
    pub fn new(sink: S, filter: F) -> Self {
        Self {
            sink: sink,
            filter: filter,
        }
    }
}

impl <S, F> Pipe for SplitPipe<S, F>
where
    S: Sink,
    F: Fn(&<S as Sink>::Input) -> bool,
    <S as Sink>::Input: Clone,
{
    type Input = <S as Sink>::Input;
    type Output = Self::Input;

    fn process(&self, input: Self::Input) -> Option<Self::Output> {
        if (self.filter)(&input) {
            self.sink.receive(input.clone());
        }
        Some(input)
    }
}



// (6) join a source into a pipe.
// TODO
impl <S: Sink> Sink for std::rc::Rc<S> {
    type Input = <S as Sink>::Input;

    fn receive(&self, input: Self::Input) {
        self.as_ref().receive(input);
    }
}

pub fn merge_sink<S: Sink>(sink: S) -> (impl Sink, impl Sink) {
    let rc = std::rc::Rc::new(sink);
    (rc.clone(), rc)
}



// (7) Connected pipeline.
// TODO