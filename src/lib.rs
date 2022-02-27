//! UNTESTED code.
//!
//! TODO: docs

use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(pub usize); // TODO: pub?

pub trait ActiveResource: Clone {
    fn position(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;
}

/// A `Vec` inside together with positions that move together with the elements if the `Vec`
/// has deletions or insertions.
///
/// Implemented partially.
pub trait VecWithPositions<'a, Active: ActiveResource, Inactive>
{
    type Positions: Iterator<Item = &'a Position>;
    type PositionsMut: Iterator<Item = &'a mut Position>;

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

    fn get_inactive(&self, pos: Position) -> Option<&Inactive> {
        self.vec().get(pos.0)
    }
    fn get_inactive_mut(&mut self, pos: Position) -> Option<&mut Inactive> {
        self.vec_mut().get_mut(pos.0)
    }
    fn set_inactive(&mut self, pos: Position, value: Inactive) {
        self.vec_mut()[pos.0] = value;
    }

    fn inactive_is_empty(&self) -> bool {
        self.vec().is_empty()
    }
    fn inactive_len(&self) -> usize {
        self.vec().len()
    }

    fn inactive_iter(&'a self) -> std::slice::Iter<'a, Inactive> {
        self.vec().iter()
    }
    fn inactive_iter_mut(&'a mut self) -> std::slice::IterMut<'a, Inactive> {
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
    pub fn get_inactive(&self) -> Option<&Inactive> {
        self.position.map(|pos| &self.vec[pos.0])
    }
    pub fn get_mut_inactive(&mut self) -> Option<&mut Inactive> {
        if let Some(pos) = self.position {
            Some(&mut self.vec[pos.0])
        } else {
            None
        }
    }
    pub fn set_inactive(&mut self, value: Inactive) {
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

impl<'a, Active: ActiveResource, Inactive> VecWithPositions<'a, Active, Inactive> for VecWithOnePosition<Inactive> {
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

pub struct VecWithPositionsVector<Active: ActiveResource, Inactive> {
    inactive: Vec<Inactive>,
    active: Vec<Active>,
}

impl<Active: ActiveResource, Inactive> Default for VecWithPositionsVector<Active, Inactive> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Active: ActiveResource, Inactive> VecWithPositionsVector<Active, Inactive> {
    pub fn new() -> Self {
        Self {
            inactive: Vec::new(),
            active: Vec::new(),
        }
    }

    pub fn get_inactive(&self, pos: Position) -> Option<&Inactive> {
        self.inactive.get(pos.0)
    }
    pub fn get_mut_inactive(&mut self, pos: Position) -> Option<&mut Inactive> {
        self.inactive.get_mut(pos.0)
    }
    pub fn set_inactive(&mut self, pos: Position, value: Inactive) {
        self.inactive[pos.0] = value;
    }

    pub fn get_active(&self, pos_index: usize) -> Option<Active> {
        self.active.get(pos_index).map(|v| v.clone())
    }
    pub fn set_active(&mut self, pos_index: usize, value: Active) {
        self.active[pos_index] = value;
    }

    pub fn clear(&mut self) {
        self.inactive.clear();
        self.active.clear();
    }
    pub fn active_len(&self) -> usize {
        self.active.len()
    }
    pub fn active_is_empty(&self) -> bool {
        self.active.is_empty()
    }
}

impl<'a, Active: ActiveResource, Inactive> VecWithPositions<'a, Active, Inactive> for VecWithPositionsVector<Active, Inactive> {
    type Positions = Box<dyn Iterator<Item = &'a Position> + 'a>;
    type PositionsMut = Box<dyn Iterator<Item = &'a mut Position> + 'a>;
    fn vec(&self) -> &Vec<Inactive> {
        &self.inactive
    }
    fn vec_mut(&mut self) -> &mut Vec<Inactive> {
        &mut self.inactive
    }
    fn positions(&'a self) -> Self::Positions {
        Box::new(self.active.iter().map(|value| value.position()))
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        Box::new(self.active.iter_mut().map(|value| value.position_mut()))
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
pub struct ResourcePool<Active: ActiveResource, Inactive: Clone> {
    inactive: Vec<Inactive>,
    active: Vec<Active>,
    next: Option<Position>, // wraps around circularly
    allocator: Box<dyn Fn(Inactive, Position, usize) -> Pin<Box<dyn Future<Output = Active> + Send + Sync>> + Send + Sync>,
}

impl<'a, Active: ActiveResource, Inactive: Clone> VecWithPositions<'a, Active, Inactive> for ResourcePool<Active, Inactive> {
    type Positions = Box<dyn Iterator<Item = &'a Position> + 'a>;
    type PositionsMut = Box<dyn Iterator<Item = &'a mut Position> + 'a>;
    fn vec(&self) -> &Vec<Inactive> {
        &self.inactive
    }
    fn vec_mut(&mut self) -> &mut Vec<Inactive> {
        &mut self.inactive
    }
    fn positions(&'a self) -> Self::Positions {
        Box::new(self.active.iter().map(|value| value.position()).chain(self.next.iter()))
    }
    fn positions_mut(&'a mut self) -> Self::PositionsMut {
        Box::new(self.active.iter_mut().map(|value| value.position_mut()).chain(self.next.iter_mut()))
    }
}

impl<'a, Active: ActiveResource, Inactive: Clone> ResourcePool<Active, Inactive> {
    pub fn new(allocator: Box<dyn Fn(Inactive, Position, usize) -> Pin<Box<dyn Future<Output = Active> + Send + Sync>> + Send + Sync>) -> Self {
        Self {
            inactive: Vec::new(),
            active: Vec::new(),
            next: None,
            allocator,
        }
    }

    pub fn push(&mut self, value: Inactive) {
        self.inactive.push(value);
        if self.next.is_none() {
            self.next = Some(Position(0));
        }
    }
    pub fn append(&mut self, other: &mut Vec<Inactive>) {
        self.inactive.append(other);
        if self.next.is_none() {
            self.next = Some(Position(0));
        }
    }
    pub fn inactive_len(&self) -> usize {
        self.inactive.len()
    }
    pub fn inactive_is_empty(&self) -> bool {
        self.inactive.is_empty()
    }
    pub fn active_len(&self) -> usize {
        self.active.len()
    }
    pub fn active_is_empty(&self) -> bool {
        self.active.is_empty()
    }

    /// Allocates a resource if there are free resources.
    pub async fn allocate_new_position(&mut self) -> Option<usize> {
        if self.active.len() >= self.inactive.len() {
            None
        } else {
            self.allocate_rapacious().await
        }
    }
    /// Allocates a resource even if all resources are busy.
    pub async fn allocate_rapacious(&mut self) -> Option<usize> {
        let len = self.active.len();
        if let Some(new) = self.allocate_base(len).await {
            self.active.push(new);
            Some(len)
        } else {
            None
        }
    }
    /// Reallocates a resource.
    pub async fn reallocate_position(&mut self, pos_index: usize) {
        if let Some(new) = self.allocate_base(pos_index).await {
            self.active[pos_index] = new;
        }
    }
    /// Allocates a resource even if all resources are busy.
    async fn allocate_base(&mut self, pos_index: usize) -> Option<Active> {
        if let Some(new_pos) = self.next {
            if let Some(inactive) = self.get_inactive(new_pos) {
                let active = (self.allocator)(inactive.clone(), new_pos, pos_index).await;
                let len = self.inactive_len();
                self.next = Some(Position(if new_pos.0 + 1 == len {
                    0
                } else {
                    new_pos.0 + 1
                }));
                Some(active)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_active(&self, pos_index: usize) -> Option<Active> {
        self.active.get(pos_index).map(|v| v.clone())
    }
    pub fn set_active(&mut self, pos_index: usize, value: Active) {
        self.active[pos_index] = value;
    }

    pub fn get_inactive(&self, pos: Position) -> Option<&Inactive> {
        self.inactive.get(pos.0)
    }
    pub fn get_mut_inactive(&mut self, pos: Position) -> Option<&mut Inactive> {
        self.inactive.get_mut(pos.0)
    }
    pub fn set_inactive(&mut self, pos: Position, value: Inactive) {
        self.inactive[pos.0] = value;
    }
    pub fn get_inactive_by_pos_index(&self, pos_index: usize) -> Option<&Inactive> {
        if let Some(active) = self.get_active(pos_index) {
            self.get_inactive(*active.position())
        } else {
            None
        }
    }
    pub fn get_inactive_mut_by_pos_index(&mut self, pos_index: usize) -> Option<&mut Inactive> {
        if let Some(active) = self.get_active(pos_index) {
            self.get_inactive_mut(*active.position())
        } else {
            None
        }
    }
    pub fn set_inactive_by_pos_index(&mut self, pos_index: usize, value: Inactive) {
        if let Some(active) = self.get_active(pos_index) {
            self.set_inactive(*active.position(), value);
        }
    }

    pub fn remove_by_position_index(&mut self, pos_index: usize) -> Inactive {
        self.remove(*self.active[pos_index].position())
    }
    pub fn clear(&mut self) {
        self.inactive.clear();
        self.active.clear();
    }

    pub fn allocated_len(&self) -> usize {
        self.active.len()
    }
    pub fn allocated_is_empty(&self) -> bool {
        self.active.is_empty()
    }
}

/// Tests do not pass.
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
        assert_eq!(v.inactive_iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(3)));
    }

    #[test]
    fn one_position_middle() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(5)));
        v.remove(Position(5));
        assert_eq!(v.inactive_iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(5)));
    }

    #[test]
    fn one_position_after() {
        let mut v = VecWithOnePosition::new();
        let mut input = (0..10).collect::<Vec<i32>>();
        v.append(&mut input);
        v.set_position(Some(Position(7)));
        v.remove(Position(5));
        assert_eq!(v.inactive_iter().map(|n| *n).collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
        assert_eq!(v.get_position(), Some(Position(6)));
    }
}
