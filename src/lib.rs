use std::iter::{once, Once};

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.

pub trait VecWithPositions<'a, T, Positions: Iterator<Item = &'a mut usize>> {
    fn vec(&self) -> &Vec<T>;
    fn vec_mut(&mut self) -> &mut Vec<T>;
    fn positions(&'a self) -> Box<dyn Iterator<Item = &'a usize> + 'a>;
    fn positions_mut(&'a mut self) -> Box<dyn Iterator<Item = &'a mut usize> + 'a>;
    fn push(&mut self, value: T) {
        self.vec_mut().push(value)
    }
    fn append(&mut self, other: &mut Vec<T>) {
        self.vec_mut().append(other)
    }
    fn remove(&'a mut self, index: usize) -> Option<T> {
        if self.vec().is_empty() {
            return None;
        }
        let result = self.vec_mut().remove(index);
        for pos in self.positions_mut() {
            if *pos > index {
                *pos -= 1;
            }
        }
        Some(result)
    }
    fn clear(&mut self) {
        self.vec_mut().clear();
    }
    fn get(&self, index: usize) -> Option<&T> {
        self.vec().get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec_mut().get_mut(index)
    }

    fn is_empty(&self) -> bool {
        self.vec().is_empty()
    }
    fn len(&self) -> usize {
        self.vec().len()
    }

    fn iter(&'a self) -> std::slice::Iter<'a, T> {
        self.vec().iter()
    }
    fn iter_mut(&'a mut self) -> std::slice::IterMut<'a, T> {
        self.vec_mut().iter_mut()
    }
}

pub struct VecWithOnePosition<T> {
    vec: Vec<T>,
    position: usize,
}

impl<T> VecWithOnePosition<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            position: 0
        }
    }
    pub fn get_position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
    }
}

impl<'a, T> VecWithPositions<'a, T, Once<&'a mut usize>> for VecWithOnePosition<T> {
    fn vec(&self) -> &Vec<T> {
        &self.vec
    }
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }
    fn positions(&'a self) -> Box<dyn Iterator<Item = &'a usize> + 'a> {
        Box::new(once(&self.position))
    }
    fn positions_mut(&'a mut self) -> Box<dyn Iterator<Item = &'a mut usize> + 'a>  {
        Box::new(once(&mut self.position))
    }
}

#[cfg(test)]
mod tests {
    use crate::{VecWithOnePosition, VecWithPositions};

    #[test]
    fn before() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(3);
        v.remove(5);
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), 3);
    }

    #[test]
    fn middle() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(5);
        v.remove(5);
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), 5);
    }

    #[test]
    fn after() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(7);
        v.remove(5);
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), 6);
    }
}
