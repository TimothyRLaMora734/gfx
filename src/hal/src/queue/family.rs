//! Queue family and groups.

use Backend;
use queue::{CommandQueue, QueueType};
use queue::capability::{Capability, Graphics, Compute};

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

/// General information about a queue family, available upon adapter discovery.
///
/// *Note*: A backend can expose multiple queue families with the same properties.
pub trait QueueFamily: Debug {
    /// Returns the type of queues.
    fn queue_type(&self) -> QueueType;
    /// Returns maximum number of queues created from this family.
    fn max_queues(&self) -> usize;
    /// Returns true if the queue supports graphics operations.
    fn supports_graphics(&self) -> bool {
        Graphics::supported_by(self.queue_type())
    }
    /// Returns true if the queue supports graphics operations.
    fn supports_compute(&self) -> bool {
        Compute::supported_by(self.queue_type())
    }
}

/// Identifier for a queue family of a physical device.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct QueueFamilyId(pub usize);

// Only needed for backend implementations.
#[doc(hidden)]
pub struct RawQueueGroup<B: Backend> {
    pub family: B::QueueFamily,
    pub queues: Vec<B::CommandQueue>,
}

impl<B: Backend> RawQueueGroup<B> {
    #[doc(hidden)]
    pub fn new(family: B::QueueFamily) -> Self {
        RawQueueGroup {
            family,
            queues: Vec::new(),
        }
    }
    #[doc(hidden)]
    pub fn add_queue(&mut self, queue: B::CommandQueue) {
        assert!(self.queues.len() < self.family.max_queues());
        self.queues.push(queue);
    }
}


/// Strong-typed group of queues of the same queue family.
pub struct QueueGroup<B: Backend, C> {
    pub(crate) family: QueueFamilyId,
    /// Command queues created in this family.
    pub queues: Vec<CommandQueue<B, C>>,
}

impl<B: Backend, C: Capability> QueueGroup<B, C> {
    /// Create a new strongly typed queue group from a raw one.
    ///
    /// # Panics
    ///
    /// Panics if the family doesn't expose required queue capabilities.
    fn new(id: QueueFamilyId, raw: RawQueueGroup<B>) -> Self {
        assert!(C::supported_by(raw.family.queue_type()));
        QueueGroup {
            family: id,
            queues: raw.queues
                .into_iter()
                .map(|q| CommandQueue(q, PhantomData))
                .collect(),
        }
    }
}

/// Contains a list of all instantiated queue queues, grouped by their
/// associated queue family.
pub struct Queues<B: Backend>(HashMap<QueueFamilyId, RawQueueGroup<B>>);

impl<B: Backend> Queues<B> {
    #[doc(hidden)]
    pub fn new(queues: HashMap<QueueFamilyId, RawQueueGroup<B>>) -> Self {
        Queues(queues)
    }

    /// Removes the queue family with the passed id from the queue list and
    /// returns the queue group.
    ///
    /// # Panics
    ///
    /// Panics if the family doesn't expose required queue capabilities.
    pub fn take<C: Capability>(&mut self, id: QueueFamilyId) -> Option<QueueGroup<B, C>> {
        self.0.remove(&id).map(|group| QueueGroup::new(id, group))
    }

    /// Removes the queue family with the passed id from the queue list and
    /// returns the command queues.
    pub fn take_raw(&mut self, id: QueueFamilyId) -> Option<Vec<B::CommandQueue>> {
        self.0.remove(&id).map(|group| group.queues)
    }
}
