use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::{Deref, RangeBounds},
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

impl<T, K, I> From<MessageData<T, K, I>> for Message<T, K>
where
    K: Eq + Hash,
{
    fn from(data: MessageData<T, K, I>) -> Self {
        Self {
            content: data.content,
            has_read: Arc::new(RwLock::new(HashSet::from_iter(data.has_read))),
            validator: None,
        }
    }
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
        self.has_read.read().await.contains(reader)
    }

    #[inline]
    pub async fn mark_readed(&self, reader: K) -> bool {
        self.has_read.write().await.insert(reader)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData<T, K, I> {
    pub msg_id: I,
    pub content: T,
    pub has_read: Vec<K>,
    pub has_validator: bool,
}

impl<T, K, I> MessageData<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    pub async fn from_message(msg_id: &I, message: &Message<T, K>) -> Self {
        Self {
            msg_id: msg_id.clone(),
            content: message.content.clone(),
            has_read: message.has_read.read().await.iter().cloned().collect(),
            has_validator: message.validator.is_some(),
        }
    }
}

#[derive(Clone)]
pub struct ReceivedMessages<T, I> {
    pub messages: Vec<T>,
    pub last_msg_id: I,
}

#[derive(Debug, Default)]
pub struct MessageQueue<T, K, I> {
    pub raw: RwLock<RawMessageQueue<T, K, I>>,
}

impl<T, K, I> From<Vec<MessageData<T, K, I>>> for MessageQueue<T, K, I>
where
    I: Ord + Copy,
    K: Eq + Hash,
{
    fn from(data: Vec<MessageData<T, K, I>>) -> Self {
        Self { raw: RwLock::new(data.into()) }
    }
}

impl<T, K, I> Deref for MessageQueue<T, K, I> {
    type Target = RwLock<RawMessageQueue<T, K, I>>;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<T, K, I> MessageQueue<T, K, I>
where
    T: Clone,
    K: Clone + Eq + Hash,
    I: MessageId,
{
    #[inline]
    pub async fn push_message(
        &self,
        content: T,
        validator: Option<MessageValidator<T, K>>,
    ) {
        self.write().await.push_message(content, validator)
    }

    #[inline]
    pub async fn push_message_excludes(
        &self,
        content: T,
        excludes: impl IntoIterator<Item = K>,
        validator: Option<MessageValidator<T, K>>,
    ) {
        self.write().await.push_message_excludes(content, excludes, validator)
    }

    #[inline]
    pub async fn remove_messages(&self, message_ids: &[I]) -> usize {
        self.write().await.remove_messages(message_ids)
    }

    #[inline]
    pub async fn remove_messages_in_range<R>(&self, range: R) -> usize
    where
        R: RangeBounds<I>,
    {
        self.write().await.remove_messages_in_range(range)
    }

    #[inline]
    pub async fn remove_messages_before_id(&self, msg_id: &I) -> usize {
        self.write().await.remove_messages_before_id(msg_id)
    }

    #[inline]
    pub async fn remove_messages_after_id(&self, msg_id: &I) -> usize {
        self.write().await.remove_messages_after_id(msg_id)
    }

    #[inline]
    pub async fn collect_invalid_mesages(&self) -> Vec<I> {
        self.read().await.collect_invalid_mesages()
    }

    #[inline]
    pub async fn remove_invalid_messages(&self) -> usize {
        self.write().await.remove_invalid_messages()
    }

    #[inline]
    pub async fn receive_messages(
        &self,
        reader: &K,
        from_msg_id: &I,
        receive_count: Option<usize>,
    ) -> Option<ReceivedMessages<T, I>> {
        self.read()
            .await
            .receive_messages(reader, from_msg_id, receive_count)
            .await
    }

    pub async fn snapshot_messages(&self) -> Vec<MessageData<T, K, I>> {
        self.read().await.snapshot_messages().await
    }
}

#[derive(Clone, Default)]
pub struct RawMessageQueue<T, K, I> {
    pub messages: BTreeMap<I, Message<T, K>>,
}

impl<T, K, I> From<Vec<MessageData<T, K, I>>> for RawMessageQueue<T, K, I>
where
    I: Ord + Copy,
    K: Eq + Hash,
{
    fn from(data: Vec<MessageData<T, K, I>>) -> Self {
        Self {
            messages: BTreeMap::from_iter(
                data.into_iter().map(|d| (d.msg_id, d.into())),
            ),
        }
    }
}

impl<T, K, I> Debug for RawMessageQueue<T, K, I>
where
    T: Debug,
    K: Debug,
    I: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawMessageQueue")
            .field("messages", &self.messages)
            .finish()
    }
}

impl<T, K, I> RawMessageQueue<T, K, I>
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
            if msg.is_readed(reader).await {
                continue;
            }

            msg.mark_readed(reader.clone()).await;

            // init or push msg to list
            match messages {
                Some(ref mut messages) => messages.push(msg.content.clone()),
                None => messages = Some(vec![msg.content.clone()]),
            };

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

    pub async fn snapshot_messages(&self) -> Vec<MessageData<T, K, I>> {
        let mut messages = Vec::with_capacity(self.messages.len());

        for (msg_id, msg) in self.messages.iter() {
            messages.push(MessageData::from_message(msg_id, msg).await);
        }

        messages
    }
}
