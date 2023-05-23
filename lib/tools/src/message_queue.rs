use crate::lazy_init;
use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    ops::{Deref, DerefMut, RangeBounds},
    sync::Arc,
};

pub type MessageValidator<T, K> =
    Arc<dyn Fn(&Message<T, K>) -> bool + Sync + Send + 'static>;

pub trait MessageId: Clone + Eq + Ord {
    fn generate() -> Self;
}

#[derive(Clone)]
pub struct Message<T, K> {
    pub content: T,
    pub has_read: HashSet<K>,
    pub validator: Option<MessageValidator<T, K>>,
}

impl<T, K> Message<T, K> {
    #[inline]
    pub fn is_valid(&self) -> bool {
        if let Some(validator) = &self.validator {
            validator(self)
        } else {
            true
        }
    }
}

impl<T, K> Message<T, K>
where
    K: Eq + Hash,
{
    #[inline]
    pub fn is_readed(&self, k: &K) -> bool {
        self.has_read.contains(k)
    }
}

impl<T, K> Deref for Message<T, K> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T, K> DerefMut for Message<T, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

#[derive(Clone)]
pub struct ReceivedMessages<T, I> {
    pub messages: Vec<T>,
    pub last_msg_id: I,
}

impl<T, I> Deref for ReceivedMessages<T, I> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}

impl<T, I> DerefMut for ReceivedMessages<T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}

#[derive(Clone, Default)]
pub struct MessageQueue<T, K, I> {
    pub messages: BTreeMap<I, Message<T, K>>,
}

impl<T, K, I> Deref for MessageQueue<T, K, I> {
    type Target = BTreeMap<I, Message<T, K>>;

    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}

impl<T, K, I> DerefMut for MessageQueue<T, K, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}

impl<T, K, I> MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    #[inline]
    pub fn push(
        &mut self,
        content: T,
        validator: Option<MessageValidator<T, K>>,
    ) {
        self.messages.insert(
            I::generate(),
            Message { content, has_read: HashSet::default(), validator },
        );
    }

    #[inline]
    pub fn push_excludes(
        &mut self,
        content: T,
        excludes: impl IntoIterator<Item = K>,
        validator: Option<MessageValidator<T, K>>,
    ) {
        self.messages.insert(
            I::generate(),
            Message {
                content,
                has_read: HashSet::from_iter(excludes),
                validator,
            },
        );
    }

    #[inline]
    pub fn batch_remove(&mut self, keys: &[I]) -> usize {
        for id in keys {
            self.messages.remove(id);
        }
        keys.len()
    }

    #[inline]
    pub fn remove_range<R>(&mut self, range: R) -> usize
    where
        R: RangeBounds<I>,
    {
        let should_delete = self
            .messages
            .range(range)
            .map(|(k, _)| k.clone())
            .collect::<Vec<I>>();

        let mut removed_count = 0;
        for id in should_delete.iter() {
            self.messages.remove(id);
            removed_count += 1;
        }

        removed_count
    }

    #[inline]
    pub fn remove_before_id(&mut self, msg_id: &I) -> usize {
        self.remove_range(..=msg_id)
    }

    #[inline]
    pub fn remove_after_id(&mut self, msg_id: &I) -> usize {
        self.remove_range(msg_id..)
    }

    #[inline]
    pub fn receive(
        &mut self,
        read_key: &K,
        start_msg_id: &I,
        receive_count: Option<usize>,
    ) -> Option<ReceivedMessages<T, I>> {
        let mut should_delete = None::<Vec<I>>;
        let mut messages = None::<Vec<T>>;
        let mut last_msg_id = None::<I>;

        let mut received_msg_count = 0;

        for (msg_id, msg) in self.messages.range_mut(start_msg_id..) {
            if msg.has_read.contains(read_key) {
                continue
            }

            if !msg.is_valid() {
                lazy_init!(should_delete => should_delete.push(msg_id.clone()), vec![msg_id.clone()]);
                continue
            }

            lazy_init!(messages => messages.push(msg.content.clone()), vec![msg.content.clone()]);
            msg.has_read.insert(read_key.clone());
            last_msg_id = Some(msg_id.clone());

            if let Some(receive_count) = receive_count {
                received_msg_count += 1;

                if received_msg_count >= receive_count {
                    break
                }
            }
        }

        should_delete.map(|list| {
            list.into_iter().map(|msg_id| self.messages.remove(&msg_id))
        });

        messages.map(|messages| ReceivedMessages {
            messages,
            last_msg_id: last_msg_id.unwrap(),
        })
    }
}
