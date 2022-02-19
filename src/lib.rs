//! UNTESTED code.
//!
//! TODO: docs

use std::iter::{Chain, once, Once};

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(usize); // TODO: pub?

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.
pub trait VecWithPositions<'a, T>
{
    type Positions: Iterator<Item = &'a Option<&'a Position>> + 'a;
    type PositionsMut: Iterator<Item = &'a mut Option<&'a mut Position>> + 'a;

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
        for ref mut pos in self.positions_mut().filter_map(|p| *p) {
            if (*pos).0 > index {
                (*pos).0 -= 1;
            }
        }
        result
    }
    fn clear(&mut self) { // FIXME: Clear positions, too.
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
    position: Option<Position>,
}

impl<T> VecWithOnePosition<T> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            position: None,
        }
    }
    pub fn get_position(&self) -> Option<Position> {
        self.position
    }
    pub fn set_position(&mut self, pos: Option<Position>) {
        self.position = pos;
    }
}

impl<T> Default for VecWithOnePosition<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> VecWithPositions<'a, T> for VecWithOnePosition<T> {
    type Positions = Once<&'a Option<&'a Position>>;
    type PositionsMut = Once<&'a mut Option<&'a mut Position>>;
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
    positions: Vec<Position>,
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

    pub fn get_position(&self, index: usize) -> Option<&Position> {
        self.positions.get(index)
    }
    pub fn set_position(&mut self, index: usize, pos: Position) {
        self.positions[index] = pos;
    }
    fn remove_by_position_index(&mut self, index: usize) -> Option<T> {
        if let Some(Position(Some(pos))) = self.get_position() {
            Some(self.remove(*pos))
        } else {
            None
        }
    }
}

impl<'a, T> VecWithPositions<'a, T> for VecWithPositionsVector<T> {
    type Positions = std::slice::Iter<'a, Option<&'a Position>>;
    type PositionsMut = std::slice::IterMut<'a, Option<&'a mut Position>>;
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

/// We have a vector of "resources", "allocated" positions therein, and "next" resource to be allocated.
/// There is the operation to replace a position in the vector of positions
/// by the available position.
///
/// Example: Several threads use a pool of network nodes to download from.
/// From the pool we "view" a range of currently used nodes, one by thread.
/// If a note is invalidated, it is removed from the list and the lacking thread
/// is moved to the end of the list receiving a new node.
/// Nodes later than it in the range decrease their positions.
/// Despite of the name, positions can be the same, if shortage of the pool.
pub struct VecWithPositionsAllDifferent<T> {
    resources: Vec<T>,
    allocated: Vec<Position>,
    next: Option<Position>, // wraps around circularly // FIXME: If it is deleted, further allocation fails.
}

impl<'a, T> VecWithPositions<'a, T> for VecWithPositionsAllDifferent<T> {
    type Positions = Chain<std::slice::Iter<'a, Option<&'a Position>>, Once<&'a Option<&'a Position>>>;
    type PositionsMut = Chain<std::slice::IterMut<'a, Option<&'a mut Position>>, Once<&'a mut Option<&'a mut Position>>>;
    fn vec(&self) -> &Vec<T> {
        &self.resources
    }
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.resources
    }
    fn positions(&'a self) -> Self::Positions {
        self.allocated.iter().chain(once(&self.next))
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        self.allocated.iter_mut().chain(once(&mut self.next))
    }
}


impl<T> VecWithPositionsAllDifferent<T> {
    pub fn push(&mut self, value: T) {
        self.resources.push(value);
    }
    pub fn append(&mut self, other: &mut Vec<T>) {
        self.resources.append(other);
    }
    pub fn len(&self) -> usize {
        self.resources.len()
    }
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Allocates a resource if there are free resources.
    pub fn allocate(&mut self) -> Option<Position> {
        if self.allocated.contains(&self.next) {
            None
        } else {
            Some(self.allocate_voracious())
        }
    }
    /// Allocates a resource even if all resources are busy.
    pub fn allocate_voracious(&mut self) -> Position {
        let result = self.next;
        let len = self.len();
        if let Some(ref mut current) = self.next.0 {
            *current += 1;
            if *current == len {
                *current = 0;
            }
        }
        result
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.resources.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.resources.get_mut(index)
    }
    pub fn set(&mut self, index: usize, value: T) {
        self.resources[index] = value;
    }
    // pub fn get_by_position(&self, pos: Position) -> Option<&T> {
    //     self.resources.get_by_position(pos)
    // }
    // pub fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut T> {
    //     self.resources.get_mut_by_position(pos)
    // }
    // pub fn set_by_position(&mut self, pos: Position, value: T) {
    //     self.resources.set_by_position(pos, value)
    // }
    // pub fn remove_by_position(&mut self, pos: Position) -> T {
    //     self.resources.remove_by_position(pos)
    // }
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
