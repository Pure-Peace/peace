use crate::lazy_init;
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::RangeBounds,
    sync::Arc,
};
use tokio::sync::RwLock;

pub type MessageValidator<T, K> =
    Arc<dyn Fn(&Message<T, K>) -> bool + Sync + Send + 'static>;

pub trait MessageId: Clone + Eq + Ord {
    fn generate() -> Self;
}

#[derive(Clone)]
pub struct Message<T, K> {
    pub content: T,
    pub has_read: Arc<RwLock<HashSet<K>>>,
    pub validator: Option<MessageValidator<T, K>>,
}

impl<T, K> Debug for Message<T, K>
where
    T: Debug,
    K: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("content", &self.content)
            .field("has_read", &self.has_read)
            .field("validator", &self.validator.is_some())
            .finish()
    }
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
    pub async fn is_readed(&self, reader: &K) -> bool {
        self.has_read.read().await.contains(&reader)
    }

    #[inline]
    pub async fn mark_readed(&self, reader: K) -> bool {
        self.has_read.write().await.insert(reader)
    }
}

#[derive(Clone)]
pub struct ReceivedMessages<T, I> {
    pub messages: Vec<T>,
    pub last_msg_id: I,
}

#[derive(Clone, Default)]
pub struct MessageQueue<T, K, I> {
    pub messages: BTreeMap<I, Message<T, K>>,
}

impl<T, K, I> Debug for MessageQueue<T, K, I>
where
    T: Debug,
    K: Debug,
    I: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageQueue")
            .field("messages", &self.messages)
            .finish()
    }
}

impl<T, K, I> MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    #[inline]
    pub fn push_message(
        &mut self,
        content: T,
        validator: Option<MessageValidator<T, K>>,
    ) {
        self.messages.insert(
            I::generate(),
            Message { content, has_read: Default::default(), validator },
        );
    }

    #[inline]
    pub fn push_message_excludes(
        &mut self,
        content: T,
        excludes: impl IntoIterator<Item = K>,
        validator: Option<MessageValidator<T, K>>,
    ) {
        self.messages.insert(
            I::generate(),
            Message {
                content,
                has_read: Arc::new(HashSet::from_iter(excludes).into()),
                validator,
            },
        );
    }

    #[inline]
    pub fn remove_messages(&mut self, message_ids: &[I]) -> usize {
        let mut success = 0;
        for id in message_ids {
            self.messages.remove(id);
            success += 1;
        }

        success
    }

    #[inline]
    pub fn remove_messages_in_range<R>(&mut self, range: R) -> usize
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
    pub fn remove_messages_before_id(&mut self, msg_id: &I) -> usize {
        self.remove_messages_in_range(..=msg_id)
    }

    #[inline]
    pub fn remove_messages_after_id(&mut self, msg_id: &I) -> usize {
        self.remove_messages_in_range(msg_id..)
    }

    #[inline]
    pub fn collect_invalid_mesages(&self) -> Vec<I> {
        self.messages
            .iter()
            .filter(|(_, v)| !v.is_valid())
            .map(|(k, _)| k.clone())
            .collect::<Vec<I>>()
    }

    #[inline]
    pub fn remove_invalid_messages(&mut self) -> usize {
        self.remove_messages(&self.collect_invalid_mesages())
    }

    #[inline]
    pub async fn receive_messages(
        &self,
        reader: &K,
        from_msg_id: &I,
        receive_count: Option<usize>,
    ) -> Option<ReceivedMessages<T, I>> {
        let mut messages = None::<Vec<T>>;
        let mut last_msg_id = None::<I>;

        let mut received_msg_count = 0;

        for (msg_id, msg) in self.messages.range(from_msg_id..) {
            if !msg.is_valid() {
                continue;
            }

            // check read state
            if msg.is_readed(&reader).await {
                continue;
            }

            msg.mark_readed(reader.clone()).await;

            // init or push msg to list
            lazy_init!(messages => messages.push(msg.content.clone()), vec![msg.content.clone()]);
            last_msg_id = Some(msg_id.clone());

            // check receive count
            if let Some(receive_count) = receive_count {
                received_msg_count += 1;

                if received_msg_count >= receive_count {
                    break;
                }
            }
        }

        messages.map(|messages| ReceivedMessages {
            messages,
            last_msg_id: last_msg_id.unwrap(),
        })
    }
}
