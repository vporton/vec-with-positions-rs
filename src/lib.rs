/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.

pub struct VecWithPositions<T> {
    vec: Vec<T>, // Apparently, I violated single-responsibility principle: any container-like object would suit.
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
            if *pos > index {
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

    pub fn new_position(&mut self, value: usize) -> usize {
        let pos = self.positions.len();
        self.positions.push(value);
        pos
    }
    pub fn move_position(&mut self, pos_index: usize, value: usize) {
        self.positions[pos_index] = value;
    }
    pub fn position_index(&self, pos_index: usize) -> usize {
        self.positions[pos_index]
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.vec.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::VecWithPositions;

    #[test]
    fn test() {
        let mut v = VecWithPositions::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        let mut _pos3 = v.new_position(3);
        let mut _pos5 = v.new_position(5);
        let mut _pos7 = v.new_position(7);
        v.remove(5);
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0,1,2,3,4,6,7,8,9]);
        assert_eq!(v.position_index(0), 3);
        assert_eq!(v.position_index(1), 5);
        assert_eq!(v.position_index(2), 6);
    }
}