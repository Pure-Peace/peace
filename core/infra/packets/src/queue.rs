use crate::Packet;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, Default)]
pub struct PacketsQueue {
    pub queue: Arc<Mutex<VecDeque<Packet>>>,
}

impl From<Vec<Packet>> for PacketsQueue {
    fn from(packets: Vec<Packet>) -> Self {
        let mut queue = VecDeque::with_capacity(packets.len());
        for p in packets {
            queue.push_back(p);
        }
        Self::new(queue)
    }
}

impl From<Vec<u8>> for PacketsQueue {
    fn from(packets: Vec<u8>) -> Self {
        Self::new(VecDeque::from([packets.into()]))
    }
}

impl PacketsQueue {
    #[inline]
    pub fn new(packets: VecDeque<Packet>) -> Self {
        Self { queue: Arc::new(Mutex::new(packets)) }
    }

    #[inline]
    pub async fn queued_packets(&self) -> usize {
        self.queue.lock().await.len()
    }

    #[inline]
    pub async fn push_packet(&self, packet: Packet) -> usize {
        let mut queue = self.queue.lock().await;
        queue.push_back(packet);
        queue.len()
    }

    #[inline]
    pub async fn enqueue_packets<I>(&self, packets: I) -> usize
    where
        I: IntoIterator<Item = Packet>,
    {
        let mut queue = self.queue.lock().await;
        queue.extend(packets);
        queue.len()
    }

    #[inline]
    pub async fn dequeue_packet(
        &self,
        queue_lock: Option<&mut MutexGuard<'_, VecDeque<Packet>>>,
    ) -> Option<Packet> {
        match queue_lock {
            Some(queue) => queue.pop_front(),
            None => self.queue.lock().await.pop_front(),
        }
    }

    #[inline]
    pub async fn dequeue_all_packets(
        &self,
        queue_lock: Option<&mut MutexGuard<'_, VecDeque<Packet>>>,
    ) -> Vec<u8> {
        let mut buf = Vec::new();

        #[inline]
        fn dequeue(
            buf: &mut Vec<u8>,
            queue_lock: &mut MutexGuard<'_, VecDeque<Packet>>,
        ) {
            while let Some(packet) = queue_lock.pop_front() {
                buf.extend(packet);
            }
        }

        match queue_lock {
            Some(queue_lock) => dequeue(&mut buf, queue_lock),
            None => dequeue(&mut buf, &mut self.queue.lock().await),
        };

        buf
    }

    pub async fn snapshot_packets(&self) -> Vec<Packet> {
        let queue = self.queue.lock().await;
        Vec::from_iter(queue.iter().cloned())
    }
}

impl serde::Serialize for PacketsQueue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let packets = self.queue.blocking_lock().clone();
        packets.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PacketsQueue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(VecDeque::deserialize(deserializer)?))
    }
}
