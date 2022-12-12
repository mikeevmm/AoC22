use nom::{Parser, IResult};

/// Nom implements `and_then` which takes two parsers, and returns the output of the latter over
/// the output of the former. However, I often want to do the opposite, where I discard the output
/// of the former and run the latter over the rest. This module implements that behaviour, via a
/// function `.chain(other)`, defined for every `Parser`.

pub struct Chain<F, G, O1> {
    f: F,
    g: G,
    phantom: core::marker::PhantomData<O1>,
}

impl<'a, I, O1, O2, E, F: Parser<I, O1, E>, G: Parser<I, O2, E>> Parser<I, O2, E>
    for Chain<F, G, O1>
{
    fn parse(&mut self, i: I) -> IResult<I, O2, E> {
        let (i, _o1) = self.f.parse(i)?;
        let (i, o2) = self.g.parse(i)?;
        Ok((i, o2))
    }
}

pub trait ParserChain<I, O, E> {
    fn chain<G, O2>(self, g: G) -> Chain<Self, G, O>
    where
        G: Parser<O, O2, E>,
        Self: core::marker::Sized;
}

impl<F, I, O, E> ParserChain<I, O, E> for F
where
    F: Parser<I, O, E>,
{
    fn chain<G, O2>(self, g: G) -> Chain<Self, G, O>
    where
        G: Parser<O, O2, E>,
        Self: core::marker::Sized,
    {
        Chain {
            f: self,
            g,
            phantom: core::marker::PhantomData,
        }
    }
}