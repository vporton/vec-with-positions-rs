//! UNTESTED code.
//!
//! TODO: docs

use std::iter::Chain;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(usize); // TODO: pub?

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.
pub trait VecWithPositions<'a, T>
{
    type Positions: Iterator<Item = &'a Position> + 'a;
    type PositionsMut: Iterator<Item = &'a mut Position> + 'a;

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
    fn remove(&'a mut self, pos: Position) -> T {
        let result = self.vec_mut().remove(pos.0);
        self.positions_mut().for_each(|p| {
            if p.0 > pos.0 {
                p.0 -= 1;
            }
        });
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
    pub fn get_by_position(&self) -> Option<&T> {
        if let Some(pos) = self.position {
            Some(&self.vec[pos.0])
        } else {
            None
        }
    }
    pub fn get_mut_by_position(&mut self) -> Option<&mut T> {
        if let Some(pos) = self.position {
            Some(&mut self.vec[pos.0])
        } else {
            None
        }
    }
    pub fn set_by_position(&mut self, value: T) {
        if let Some(pos) = self.position {
            self.vec[pos.0] = value;
        } else {
            panic!("Attempt to set nonexisting position.");
        }
    }
    pub fn clear(&mut self) {
        self.vec.clear();
        self.position = None;
    }
}

impl<T> Default for VecWithOnePosition<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> VecWithPositions<'a, T> for VecWithOnePosition<T> {
    type Positions = std::option::Iter<'a, Position>;
    type PositionsMut = std::option::IterMut<'a, Position>;
    fn vec(&self) -> &Vec<T> {
        &self.vec
    }
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }
    fn positions(&'a self) -> Self::Positions {
        self.position.iter()
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        self.position.iter_mut()
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

    pub fn get_position(&self, index: usize) -> &Position {
        &self.positions[index]
    }
    pub fn get_position_mut(&mut self, index: usize) -> &mut Position {
        &mut self.positions[index]
    }
    pub fn set_position(&mut self, index: usize, pos: Position) {
        self.positions[index] = pos;
    }
    pub fn get_by_position(&self, pos: Position) -> Option<&T> {
        self.vec.get(pos.0)
    }
    pub fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut T> {
        self.vec.get_mut(pos.0)
    }
    pub fn set_mut_by_position(&mut self, pos: Position, value: T) {
        self.vec[pos.0] = value;
    }
    pub fn remove_by_position(&mut self, pos: Position) -> T {
        self.remove(pos)
    }
    pub fn clear(&mut self) {
        self.vec.clear();
        self.positions.clear();
    }
}

impl<'a, T> VecWithPositions<'a, T> for VecWithPositionsVector<T> {
    type Positions = std::slice::Iter<'a, Position>;
    type PositionsMut = std::slice::IterMut<'a, Position>;
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
/// If a note is invalidated, it is removed from the list.
/// Nodes later than it in the range decrease their positions.
///
/// TODO: Test it.
pub struct ResourcesPool<T> {
    resources: Vec<T>,
    allocated: Vec<Position>,
    next: Option<Position>, // wraps around circularly
}

impl<'a, T> VecWithPositions<'a, T> for ResourcesPool<T> {
    type Positions = Chain<std::slice::Iter<'a, Position>, std::option::Iter<'a, Position>>;
    type PositionsMut = Chain<std::slice::IterMut<'a, Position>, std::option::IterMut<'a, Position>>;
    fn vec(&self) -> &Vec<T> {
        &self.resources
    }
    fn vec_mut(&mut self) -> &mut Vec<T> {
        &mut self.resources
    }
    fn positions(&'a self) -> Self::Positions {
        self.allocated.iter().chain(self.next.iter())
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        self.allocated.iter_mut().chain(self.next.iter_mut())
    }
}

impl<T> ResourcesPool<T> {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            allocated: Vec::new(),
            next: None,
        }
    }

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
    pub fn allocate_new_position(&mut self) {
        if self.allocated.len() >= self.resources.len() {
            ()
        } else {
            if let Some(new_pos) = self.allocate_rapacious() {
                self.allocated.push(new_pos);
            }
        };
    }
    /// Reallocates a resource.
    pub fn reallocate_position(&mut self, index: usize) {
        if let Some(new_pos) = self.allocate_rapacious() {
            self.allocated[index] = new_pos;
        }
    }
    /// Allocates a resource even if all resources are busy.
    fn allocate_rapacious(&mut self) -> Option<Position> {
        let new_pos = self.next;
        let len = self.len();
        if let Some(new_pos) = new_pos {
            self.next = Some(Position(if new_pos.0 + 1 == len {
                0
            } else {
                new_pos.0 + 1
            }));
        }
        new_pos
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
    pub fn get_position(&self, index: usize) -> &Position {
        &self.allocated[index]
    }
    pub fn get_position_mut(&mut self, index: usize) -> &mut Position {
        &mut self.allocated[index]
    }
    pub fn set_position(&mut self, index: usize, pos: Position) {
        self.allocated[index] = pos;
    }
    pub fn get_by_position(&self, pos: Position) -> Option<&T> {
        self.resources.get( pos.0)
    }
    pub fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut T> {
        self.resources.get_mut(pos.0)
    }
    pub fn set_by_position(&mut self, pos: Position, value: T) {
        self.resources[pos.0] = value;
    }
    pub fn remove_by_position(&mut self, pos: Position) -> T {
        self.remove(pos)
    }
    pub fn clear(&mut self) {
        self.resources.clear();
        self.allocated.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Position, VecWithOnePosition, VecWithPositions};

    #[test]
    fn one_position_before() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(3)));
        v.remove(Position(5));
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(3)));
    }

    #[test]
    fn one_position_middle() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(5)));
        v.remove(Position(5));
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(5)));
    }

    #[test]
    fn one_position_after() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(7)));
        v.remove(Position(5));
        assert_eq!(v.iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(6)));
    }
}
