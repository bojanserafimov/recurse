use crate::hacks::*;

pub trait Adapter {
    fn root(&self) -> Iter;
    fn nei(&self, input: Iter) -> Bundle;
}

pub struct CompleteBinaryTree {
    pub batching: usize,
}

impl Adapter for CompleteBinaryTree {
    fn root(&self) -> Iter {
        boxit(0..=0)
    }

    fn nei(&self, input: Iter) -> Bundle {
        let input = Chunks::new(input, self.batching).flatten();
        boxit(input.map(|x| (x, boxit((10*x+1..=10*x+3).inspect(|x| println!("gen {}", x))))))
    }
}

pub struct Comb {
    pub batching: usize,
}

impl Adapter for Comb {
    fn root(&self) -> Iter {
        boxit(0..=0)
    }

    fn nei(&self, input: Iter) -> Bundle {
        let input = Chunks::new(input, self.batching).flatten();
        boxit(input.map(|x| (x, boxit((10*x+1..=10*x+5*((x+1)%2)).inspect(|x| println!("gen {}", x))))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary() {
        let a = CompleteBinaryTree {
            batching: 1,
        };
        let r = a.root();
        let r = flat(a.nei(r));
        let r = flat(a.nei(r));

        for result in r {
            println!("{}", result);
        }
    }

    #[test]
    fn test_batched() {
        let a = CompleteBinaryTree {
            batching: 2,
        };
        let r = a.root();
        let r = flat(a.nei(r));
        let r = flat(a.nei(r));
        let r = flat(a.nei(r));

        for result in r {
            println!("{}", result);
        }
    }
}
