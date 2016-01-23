use std::cell::Cell;

pub struct Cache<T>(Cell<Option<T>>);

impl<T:Copy> Cache<T> {
    pub fn new() -> Self {
        Cache(Cell::new(None))
    }

    pub fn eval<F>(&self, f: F) -> T
        where F: FnOnce() -> T {
        match self.0.get() {
            Some(cached) => cached,
            None => {
                let calc = f();
                self.0.set(Some(calc));
                calc
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cache() {
        let mut t = 0;
        let cache = Cache::new();
        let res0 = cache.eval(|| {t += 1; 5});
        assert_eq!(res0, 5);
        assert_eq!(t, 1);
        let res1 = cache.eval(|| {t += 1; 10});
        assert_eq!(res1, 5);
        assert_eq!(t, 1);
    }
}
