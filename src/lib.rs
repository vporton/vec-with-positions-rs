/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.

pub struct VecWithPositions<T> {
    vec: Vec<T>,
    positions: Vec<usize>,
}

impl<T> VecWithPositions<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            positions: Vec::new(),
        }
    }
    pub fn push(&mut self, value: T) {
        self.vec.push(value)
    }
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec.append(other)
    }
    pub fn remove(&mut self, index: usize) -> T {
        let result = self.vec.remove(index);
        for pos in self.positions.iter_mut() {
            if *pos >= index && *pos != 0 {
                *pos -= 1;
            }
        }
        result
    }
    pub fn clear(&mut self) {
        self.vec.clear();
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec.get_mut(index)
    }
}
