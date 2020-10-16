use tokio::stream::Stream;

pub trait Source {
    type Output;

    // Hmm what goes here???
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
// TODO



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



// (6) join a sink into a pipe.
// TODO



// (7) Connected pipeline.
// TODO
