//! UNTESTED code.
//!
//! TODO: docs

use std::iter::{once, Once};

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(usize); // TODO: pub?

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.
pub trait VecWithPositions<'a, T>
{
    type Positions: Iterator<Item = &'a usize> + 'a;
    type PositionsMut: Iterator<Item = &'a mut usize> + 'a;

    fn vec(&self) -> &Vec<T>;
    fn vec_mut(&mut self) -> &mut Vec<T>;
    fn positions(&'a self) -> Self::Positions;
    fn positions_mut(&'a mut self) -> Self::PositionsMut;
    fn push(&mut self, value: T) {
        self.vec_mut().push(value)
    }
    fn append(&mut self, other: &mut Vec<T>) {
        self.vec_mut().append(other)
    }
    fn remove(&'a mut self, index: usize) -> T { // FIXME
        let result = self.vec_mut().remove(index);
        for pos in self.positions_mut() {
            if *pos > index {
                *pos -= 1;
            }
        }
        result
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
    fn set(&mut self, index: usize, value: T) {
        self.vec_mut()[index] = value;
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
            position: usize::MAX, // a nonsense value
        }
    }
    pub fn get_position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, index: usize) {
        self.position = index;
    }
}

impl<T> Default for VecWithOnePosition<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> VecWithPositions<'a, T> for VecWithOnePosition<T> {
    type Positions = Once<&'a usize>;
    type PositionsMut = Once<&'a mut usize>;
    fn vec(&self) -> &Vec<T> {
        &self.vec
    }
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }
    fn positions(&'a self) -> Self::Positions {
        once(&self.position)
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        once(&mut self.position)
    }
}

pub struct VecWithPositionsVector<T> {
    vec: Vec<T>,
    positions: Vec<usize>,
}

impl<T> Default for VecWithPositionsVector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> VecWithPositionsVector<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn get_position(&self, pos: Position) -> usize {
        self.positions[pos.0]
    }
    pub fn set_position(&mut self, pos: Position, index: usize) {
        self.positions[pos.0] = index;
    }

    fn get_by_position(&self, pos: Position) -> Option<&T> {
        self.get(self.positions[pos.0])
    }
    fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut T> {
        self.get_mut(self.positions[pos.0])
    }
    fn set_by_position(&mut self, pos: Position, value: T) {
        self.set(self.positions[pos.0], value);
    }
}

impl<'a, T> VecWithPositions<'a, T> for VecWithPositionsVector<T> {
    type Positions = std::slice::Iter<'a, usize>;
    type PositionsMut = std::slice::IterMut<'a, usize>;
    fn vec(&self) -> &Vec<T> {
        &self.vec
    }
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }
    fn positions(&'a self) -> Self::Positions {
        self.positions.iter()
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        self.positions.iter_mut()
    }
}

/// Example: Several threads use a pool of network nodes to download from.
/// From the pool we "view" a range of currently used nodes, one by thread.
/// If a note is invalidated, it is removed from the list and the lacking thread
/// is moved to the end of the list receiving a new node.
/// Nodes later than it in the range decrease their positions.
/// Despite of the name, positions can be the same, if shortage of the pool.
pub struct VecWithPositionsAllDifferent<T> {
    vec_with_positions: VecWithPositionsVector<T>,
    range_start: Position,
    range_end: Position, // wraps around circularly
}

impl<T> VecWithPositionsAllDifferent<T> {
    pub fn push(&mut self, value: T) {
        self.vec_with_positions.push(value);
    }
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.vec_with_positions.append(other);
    }
    pub fn remove(&mut self, index: usize) -> T {
        self.range_end.0 = if self.range_end.0 != 0 {
            self.range_end.0 - 1
        } else {
            self.len()
        };
        self.vec_with_positions.remove(index)
    }
    pub fn clear(&mut self) {
        self.range_start = Position(0);
        self.range_end = Position(0);
        self.vec_with_positions.clear();
    }
    pub fn len(&self) -> usize {
        self.vec_with_positions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.vec_with_positions.is_empty()
    }
    pub fn new_position(&mut self) -> Position {
        self.range_end.0 += 1;
        if self.range_end.0 == self.len() {
            self.range_end.0 = 0;
        }
        self.range_end
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec_with_positions.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec_with_positions.get_mut(index)
    }
    pub fn set(&mut self, index: usize, value: T) {
        self.vec_with_positions.set(index, value)
    }
    pub fn get_by_position(&self, pos: Position) -> Option<&T> {
        self.vec_with_positions.get_by_position(pos)
    }
    pub fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut T> {
        self.vec_with_positions.get_mut_by_position(pos)
    }
    pub fn set_by_position(&mut self, pos: Position, value: T) {
        self.vec_with_positions.set_by_position(pos, value)
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
