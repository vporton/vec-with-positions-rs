//! UNTESTED code.
//!
//! TODO: docs

use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position(usize); // TODO: pub?

pub trait ActiveResource: Clone {
    fn position(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;
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

    pub fn remove(&mut self, pos_index: usize) -> Inactive { // TODO: Duplicate code.
        let pos = *self.active[pos_index].position();
        let result = self.inactive.remove(pos.0);
        self.active.iter_mut().for_each(|p| {
            let mut p2 = p.position_mut();
            if p2.0 > pos.0 {
                p2.0 -= 1;
            }
        });
        self.active.remove(pos_index);
        result
    }
    pub fn push(&mut self, value: Inactive) {
        self.inactive.push(value)
    }
    pub fn append(&mut self, other: &mut Vec<Inactive>) {
        self.inactive.append(other)
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

    pub fn inactive_is_empty(&self) -> bool {
        self.inactive.is_empty()
    }
    pub fn inactive_len(&self) -> usize {
        self.inactive.len()
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
    pub fn remove(&mut self, pos_index: usize) -> Inactive { // TODO: Duplicate code.
        let pos = *self.active[pos_index].position();
        let result = self.inactive.remove(pos.0);
        self.active.iter_mut().for_each(|p| {
            let mut p2 = p.position_mut();
            if p2.0 > pos.0 {
                p2.0 -= 1;
            }
        });
        self.active.remove(pos_index);
        result

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
            if pos_index < self.active.len() {
                self.active[pos_index] = new;
            }
        }
    }
    /// Allocates a resource even if all resources are busy.
    async fn allocate_base(&mut self, pos_index: usize) -> Option<Active> {
        let new_pos = if let Some(new_pos) = self.next {
            new_pos
        } else {
            Position(0)
        };

        let len = self.inactive_len();
        self.next = Some(Position(if new_pos.0 + 1 == len {
            0
        } else {
            new_pos.0 + 1
        }));

        if let Some(inactive) = self.get_inactive(new_pos) {
            let active = (self.allocator)(inactive.clone(), new_pos, pos_index).await;
            Some(active)
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
            self.get_mut_inactive(*active.position())
        } else {
            None
        }
    }
    pub fn set_inactive_by_pos_index(&mut self, pos_index: usize, value: Inactive) {
        if let Some(active) = self.get_active(pos_index) {
            self.set_inactive(*active.position(), value);
        }
    }

    pub fn clear(&mut self) {
        self.inactive.clear();
        self.active.clear();
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
