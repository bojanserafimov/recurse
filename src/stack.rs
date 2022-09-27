use crate::hacks::*;
use crate::adapter;

/// Stack of iterators that keeps intermediate results too
struct Stack {
    levels: Vec<RcBundleReader>,
    adapter: Box<dyn adapter::Adapter>,

    // Largest index for which we can hard_peek without pulling from parent.
    // Assumes there's no batching. Works fine in case there's batching.
    next_from: usize,

    // If true, elements pulled in advance because of batching will be returned
    // right after the batch result. This is more eager than usual, but will produce
    // out of order results
    return_eager: bool,
}

impl Stack {
    pub fn new(a: Box<dyn adapter::Adapter>) -> Self {
        let levels = vec![
            RcBundleReader::new(a.nei(a.root())),
        ];
        Stack {
            levels,
            adapter: a,
            next_from: 0,
            return_eager: false,
        }
    }

    pub fn extend(&mut self) {
        let top = RcBundleReader::clone(&self.levels[self.levels.len() - 1]);
        let new = RcBundleReader::new(self.adapter.nei(boxit(top)));
        self.levels.push(new);
    }
}

impl Iterator for Stack {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.return_eager {
            for l in &mut self.levels {
                if let Some(x) = l.pop_passed_unprepared() {
                    return Some(x);
                }
            }
        }

        // Add a new level if necessary
        if self.next_from == self.levels.len() {
            self.extend();
        }

        // We have 1 output prepared at level self.next_from - 1
        // because of the self.next_from invariant, so it's safe
        // to prepare(1).
        if let Some(x) = self.levels[self.next_from].prepare(1) {
            // Increment so that the search behaves like dfs
            self.next_from += 1;
            return Some(x);
        }

        // If prepare(1) at this level returned None, it's not safe to
        // try again since we don't have inputs prepared anymore. Try to
        // prepare using prepare(0). Move up the stack until we find
        // something.
        while self.next_from > 0 {
            if let Some(x) = self.levels[self.next_from - 1].prepare(0) {
                return Some(x);
            }
            self.next_from -= 1;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_with_binary_tree() {
        let a = Box::new(adapter::CompleteBinaryTree {
            batching: 2,
        });
        let mut s = Stack::new(a);

        for result in s {
            println!("out {}", result);
        }
    }

    #[test]
    fn stack_with_comb() {
        let a = Box::new(adapter::Comb {
            batching: 1,
        });
        let mut s = Stack::new(a);

        for result in s {
            println!("out {}", result);
        }
    }
}
