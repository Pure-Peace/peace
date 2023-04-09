use std::{
    collections::{BTreeMap, HashSet},
    hash::Hash,
    sync::Arc,
};

pub type Validator = Arc<dyn Fn() -> bool + Sync + Send + 'static>;

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
    pub validator: Option<Validator>,
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

impl<T, K, I> MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    #[inline]
    pub fn push(&mut self, content: T, validator: Option<Validator>) {
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
        validator: Option<Validator>,
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

            if let Some(valid) = &msg.validator {
                if !valid() {
                    match should_delete {
                        Some(ref mut should_delete) => {
                            should_delete.push(msg_id.clone())
                        },
                        None => should_delete = Some(vec![msg_id.clone()]),
                    }
                    continue;
                }
            }

            match messages {
                Some(ref mut messages) => messages.push(msg.content.clone()),
                None => messages = Some(vec![msg.content.clone()]),
            }
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
