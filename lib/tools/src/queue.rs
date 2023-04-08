use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    sync::Arc,
};

use crate::Ulid;

pub type Validator = Arc<dyn Fn() -> bool + Sync + Send + 'static>;

#[derive(Clone)]
pub struct Message<T, K>
where
    T: Clone,
{
    pub content: T,
    pub has_read: HashSet<K>,
    pub validator: Option<Validator>,
}

#[derive(Clone)]
pub struct ReceivedMessages<T>
where
    T: Clone,
{
    pub messages: Vec<T>,
    pub last_msg_id: Ulid,
}

#[derive(Clone, Default)]
pub struct Queue<T, K>
where
    T: Clone,
    K: Clone + Eq + Hash,
{
    pub messsages: BTreeMap<Ulid, Message<T, K>>,
}

impl<T, K> Queue<T, K>
where
    T: Clone,
    K: Clone + Eq + Hash,
{
    #[inline]
    pub fn push(&mut self, content: T, validator: Option<Validator>) {
        self.messsages.insert(
            Ulid::generate(),
            Message { content, has_read: HashSet::default(), validator },
        );
    }

    #[inline]
    pub fn push_excludes(
        &mut self,
        content: T,
        excludes: impl IntoIterator<Item = K>,
        validator: Option<Validator>,
    ) {
        self.messsages.insert(
            Ulid::generate(),
            Message {
                content,
                has_read: HashSet::from_iter(excludes),
                validator,
            },
        );
    }

    #[inline]
    pub fn receive(
        &mut self,
        read_key: &K,
        start_msg_id: &Ulid,
    ) -> Option<ReceivedMessages<T>> {
        let mut should_delete = None::<Vec<Ulid>>;
        let mut messages = None::<Vec<T>>;
        let mut last_msg_id = None::<Ulid>;

        for (msg_id, msg) in self.messsages.range_mut(start_msg_id..) {
            if msg.has_read.contains(read_key) {
                continue;
            }

            if let Some(valid) = &msg.validator {
                if !valid() {
                    match should_delete {
                        Some(ref mut should_delete) => {
                            should_delete.push(*msg_id)
                        },
                        None => should_delete = Some(vec![*msg_id]),
                    }
                    continue;
                }
            }

            match messages {
                Some(ref mut messages) => messages.push(msg.content.clone()),
                None => messages = Some(vec![msg.content.clone()]),
            }
            msg.has_read.insert(read_key.clone());
            last_msg_id = Some(*msg_id);
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
