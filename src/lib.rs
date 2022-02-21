//! UNTESTED code.
//!
//! TODO: docs
//!
//! TODO: `.await` mode to wait when a node is inserted.

use std::iter::Chain;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(pub usize); // TODO: pub?

pub trait Allocator<Active, Inactive> {
    fn allocate(inactive: &Inactive, pos: Position) -> Active;
}

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.
pub trait VecWithPositions<'a, Active, Inactive>
{
    type Positions: Iterator<Item = &'a Position> + 'a;
    type PositionsMut: Iterator<Item = &'a mut Position> + 'a;

    fn vec(&self) -> &Vec<Inactive>;
    fn vec_mut(&mut self) -> &mut Vec<Inactive>;
    fn positions(&'a self) -> Self::Positions;
    fn positions_mut(&'a mut self) -> Self::PositionsMut;
    fn push(&mut self, value: Inactive) {
        self.vec_mut().push(value)
    }
    fn append(&mut self, other: &mut Vec<Inactive>) {
        self.vec_mut().append(other)
    }
    fn remove(&'a mut self, pos: Position) -> Inactive {
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

    fn get(&self, index: usize) -> Option<&Inactive> {
        self.vec().get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut Inactive> {
        self.vec_mut().get_mut(index)
    }
    fn set(&mut self, index: usize, value: Inactive) {
        self.vec_mut()[index] = value;
    }

    fn is_empty(&self) -> bool {
        self.vec().is_empty()
    }
    fn len(&self) -> usize {
        self.vec().len()
    }

    fn iter(&'a self) -> std::slice::Iter<'a, Inactive> {
        self.vec().iter()
    }
    fn iter_mut(&'a mut self) -> std::slice::IterMut<'a, Inactive> {
        self.vec_mut().iter_mut()
    }
}

pub struct VecWithOnePosition<Inactive> {
    vec: Vec<Inactive>,
    position: Option<Position>,
}

impl<Inactive> VecWithOnePosition<Inactive> {
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
    pub fn get_by_position(&self) -> Option<&Inactive> {
        if let Some(pos) = self.position {
            Some(&self.vec[pos.0])
        } else {
            None
        }
    }
    pub fn get_mut_by_position(&mut self) -> Option<&mut Inactive> {
        if let Some(pos) = self.position {
            Some(&mut self.vec[pos.0])
        } else {
            None
        }
    }
    pub fn set_by_position(&mut self, value: Inactive) {
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

impl<Inactive> Default for VecWithOnePosition<Inactive> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Active, Inactive> VecWithPositions<'a, Active, Inactive> for VecWithOnePosition<Inactive> {
    type Positions = std::option::Iter<'a, Position>;
    type PositionsMut = std::option::IterMut<'a, Position>;
    fn vec(&self) -> &Vec<Inactive> {
        &self.vec
    }
    fn vec_mut(&mut self) -> &mut Vec<Inactive> {
        &mut self.vec
    }
    fn positions(&'a self) -> Self::Positions {
        self.position.iter()
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        self.position.iter_mut()
    }
}

pub struct VecWithPositionsVector<Active, Inactive> {
    vec: Vec<Inactive>,
    positions: Vec<Active>,
}

impl<Inactive> Default for VecWithPositionsVector<Inactive> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Active, Inactive> VecWithPositionsVector<Active, Inactive> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn get_active(&self, index: usize) -> &Active {
        &self.positions[index]
    }
    pub fn get_active_mut(&mut self, index: usize) -> &mut Active {
        &mut self.positions[index]
    }
    pub fn set_active(&mut self, index: usize, value: Active) {
        self.positions[index] = value;
    }
    pub fn get_by_position(&self, pos: Position) -> Option<&Inactive> {
        self.vec.get(pos.0)
    }
    pub fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut Inactive> {
        self.vec.get_mut(pos.0)
    }
    pub fn set_by_position(&mut self, pos: Position, value: Inactive) {
        self.vec[pos.0] = value;
    }

    pub fn get_by_position_index(&self, pos_index: usize) -> Option<&Inactive> {
        self.get_by_position(self.positions[pos_index].position())
    }
    pub fn get_mut_by_position_index(&mut self, pos_index: usize) -> Option<&mut Inactive> {
        self.get_mut_by_position(self.positions[pos_index].position())
    }
    pub fn set_by_position_index(&mut self, pos_index: usize, value: Inactive) {
        self.set_by_position(self.positions[pos_index].position(), value);
    }

    pub fn remove_by_position_index(&mut self, pos_index: usize) -> Inactive {
        self.remove(self.positions[pos_index].position())
    }
    pub fn clear(&mut self) {
        self.vec.clear();
        self.positions.clear();
    }
    pub fn positions_len(&self) -> usize {
        self.positions.len()
    }
    pub fn positions_is_empty(&self) -> bool {
        self.positions.is_empty()
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
pub struct ResourcesPool<Inactive> {
    resources: Vec<Inactive>,
    allocated: Vec<Position>,
    next: Option<Position>, // wraps around circularly
}

impl<'a, Active, Inactive> VecWithPositions<'a, Active, Inactive> for ResourcesPool<Active, Inactive> {
    type Positions = Box<Iterator<Item = &'a Position> + 'a>;
    type PositionsMut = Box<Iterator<Item = &'a mut Position> + 'a>;
    fn vec(&self) -> &Vec<Active> {
        &self.resources
    }
    fn vec_mut(&mut self) -> &mut Vec<Active> {
        &mut self.resources
    }
    fn positions(&'a self) -> Self::Positions {
        Box::new(self.allocated.iter().chain(self.next.iter()))
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        Box::new(self.allocated.iter_mut().chain(self.next.iter_mut()))
    }
}

impl<Active, Inactive> ResourcesPool<Active, Inactive> {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            allocated: Vec::new(),
            next: None,
        }
    }

    pub fn push(&mut self, value: Inactive) {
        self.resources.push(value);
        if self.next.is_none() {
            self.next = Some(Position(0));
        }
    }
    pub fn append(&mut self, other: &mut Vec<Inactive>) {
        self.resources.append(other);
        if self.next.is_none() {
            self.next = Some(Position(0));
        }
    }
    pub fn len(&self) -> usize {
        self.resources.len()
    }
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Allocates a resource if there are free resources.
    pub fn allocate_new_position<Allocator>(&mut self) -> Option<usize> {
        if self.allocated.len() >= self.resources.len() {
            None
        } else {
            if let Some(new_pos) = self.allocate_rapacious() {
                let result = self.allocated.len();
                self.allocated.push(new_pos);
                Some(Allocator::allocate(, new_pos))
                Some(result)
            } else {
                None
            }
        }
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

    pub fn get_position(&self, index: usize) -> &Position {
        &self.allocated[index]
    }
    pub fn get_position_mut(&mut self, index: usize) -> &mut Position {
        &mut self.allocated[index]
    }
    pub fn set_position(&mut self, index: usize, pos: Position) {
        self.allocated[index] = pos;
    }
    pub fn get_by_position(&self, pos: Position) -> Option<&Inactive> {
        self.resources.get( pos.0)
    }
    pub fn get_mut_by_position(&mut self, pos: Position) -> Option<&mut Inactive> {
        self.resources.get_mut(pos.0)
    }
    pub fn set_by_position(&mut self, pos: Position, value: Inactive) {
        self.resources[pos.0] = value;
    }

    pub fn get_by_position_index(&self, pos_index: usize) -> Option<&Inactive> {
        self.get_by_position( self.allocated[pos_index])
    }
    pub fn get_mut_by_position_index(&mut self, pos_index: usize) -> Option<&mut Inactive> {
        self.get_mut_by_position(self.allocated[pos_index])
    }
    pub fn set_by_position_index(&mut self, pos_index: usize, value: Inactive) {
        self.set_by_position(self.allocated[pos_index], value);
    }

    pub fn remove_by_position(&mut self, pos: Position) -> Inactive {
        self.remove(pos)
    }
    pub fn remove_by_position_index(&mut self, pos_index: usize) -> Inactive {
        self.remove(self.allocated[pos_index])
    }
    pub fn clear(&mut self) {
        self.resources.clear();
        self.allocated.clear();
    }

    pub fn allocated_len(&self) -> usize {
        self.allocated.len()
    }
    pub fn allocated_is_empty(&self) -> bool {
        self.allocated.is_empty()
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
