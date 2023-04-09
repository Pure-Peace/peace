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
pub struct Message<T, K>
where
    T: Clone,
{
    pub content: T,
    pub has_read: HashSet<K>,
    pub validator: Option<MessageValidator<T, K>>,
}

impl<T, K> Message<T, K>
where
    T: Clone,
{
    #[inline]
    pub fn is_valid(&self) -> bool {
        if let Some(validator) = &self.validator {
            validator(self)
        } else {
            true
        }
    }
}

#[derive(Clone)]
pub struct ReceivedMessages<T, I>
where
    T: Clone,
    I: MessageId,
{
    pub messages: Vec<T>,
    pub last_msg_id: I,
}

#[derive(Clone, Default)]
pub struct MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    pub messsages: BTreeMap<I, Message<T, K>>,
}

impl<T, K, I> Deref for MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    type Target = BTreeMap<I, Message<T, K>>;

    fn deref(&self) -> &Self::Target {
        &self.messsages
    }
}

impl<T, K, I> DerefMut for MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messsages
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
        self.messsages.insert(
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
        self.messsages.insert(
            I::generate(),
            Message {
                content,
                has_read: HashSet::from_iter(excludes),
                validator,
            },
        );
    }

    #[inline]
    pub fn batch_remove(&mut self, keys: &[I]) {
        for id in keys {
            self.messsages.remove(id);
        }
    }

    #[inline]
    pub fn remove_range<R>(&mut self, range: R)
    where
        R: RangeBounds<I>,
    {
        let should_delete = self
            .messsages
            .range(range)
            .map(|(k, _)| k.clone())
            .collect::<Vec<I>>();

        for id in should_delete.iter() {
            self.messsages.remove(id);
        }
    }

    #[inline]
    pub fn remove_before_id(&mut self, msg_id: &I) {
        self.remove_range(..=msg_id)
    }

    #[inline]
    pub fn remove_after_id(&mut self, msg_id: &I) {
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

        for (msg_id, msg) in self.messsages.range_mut(start_msg_id..) {
            if msg.has_read.contains(read_key) {
                continue;
            }

            if !msg.is_valid() {
                lazy_init!(should_delete => should_delete.push(msg_id.clone()), vec![msg_id.clone()]);
                continue;
            }

            lazy_init!(messages => messages.push(msg.content.clone()), vec![msg.content.clone()]);
            msg.has_read.insert(read_key.clone());
            last_msg_id = Some(msg_id.clone());

            if let Some(receive_count) = receive_count {
                received_msg_count += 1;

                if received_msg_count >= receive_count {
                    break;
                }
            }
        }

        should_delete.map(|list| {
            list.into_iter().map(|msg_id| self.messsages.remove(&msg_id))
        });

        messages.map(|messages| ReceivedMessages {
            messages,
            last_msg_id: last_msg_id.unwrap(),
        })
    }
}
