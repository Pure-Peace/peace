use crate::*;
use async_trait::async_trait;
use infra_services::IntoService;
use pb_base::ExecSuccess;
use pb_events::Event;
use std::{collections::HashMap, hash::Hash, ops::Deref, sync::Arc};
use tokio::{
    sync::{
        mpsc::{self, Sender},
        RwLock,
    },
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tools::atomic::{AtomicOperation, Usize};

#[derive(Debug)]
pub struct Subscription<T> {
    pub tx: Sender<T>,
    pub task: JoinHandle<()>,
}

#[derive(Debug)]
pub struct SubscriptionWithOutputStream<T> {
    pub subscription: Arc<Subscription<T>>,
    pub stream: ReceiverStream<Result<T, Status>>,
}

#[derive(Debug)]
pub struct SubscriptionStoreInner<K, T> {
    pub indexes: RwLock<HashMap<K, Arc<Subscription<T>>>>,
    pub len: Usize,
}

pub struct EventsServiceImpl {
    pub store: Arc<SubscriptionStore<String, Event>>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionStore<K, T> {
    pub inner: Arc<SubscriptionStoreInner<K, T>>,
}

impl<K, T> Deref for SubscriptionStore<K, T> {
    type Target = Arc<SubscriptionStoreInner<K, T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, T> Deref for SubscriptionStoreInner<K, T> {
    type Target = RwLock<HashMap<K, Arc<Subscription<T>>>>;

    fn deref(&self) -> &Self::Target {
        &self.indexes
    }
}

impl<K, T> Default for SubscriptionStoreInner<K, T> {
    fn default() -> Self {
        Self { indexes: Default::default(), len: Default::default() }
    }
}

impl<K, T> SubscriptionStoreInner<K, T>
where
    K: Eq + Hash,
{
    pub async fn add(
        &self,
        subscriber_key: K,
        subscription: Arc<Subscription<T>>,
    ) -> Option<Arc<Subscription<T>>> {
        let removed = self.write().await.insert(subscriber_key, subscription);
        if removed.is_none() {
            self.len.add(1);
        }

        removed
    }

    pub async fn remove(
        &self,
        subscriber_key: &K,
    ) -> Option<Arc<Subscription<T>>> {
        let removed = self.write().await.remove(subscriber_key);
        if removed.is_some() {
            self.len.sub(1);
        }

        removed
    }
}

impl<K, T> Default for SubscriptionStore<K, T> {
    fn default() -> Self {
        Self { inner: Default::default() }
    }
}

impl<K, T> SubscriptionStore<K, T>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    T: Send + 'static,
{
    pub async fn create(
        &self,
        subscriber_key: K,
        buffer_server: usize,
        buffer_client: usize,
    ) -> SubscriptionWithOutputStream<T> {
        let (server_tx, mut server_rx) = mpsc::channel::<T>(buffer_server);
        let (client_tx, client_rx) =
            mpsc::channel::<Result<T, Status>>(buffer_client);

        let handle = {
            let inner_cloned = self.inner.clone();
            let subscriber_key_cloned = subscriber_key.clone();

            tokio::spawn(async move {
                let inner = inner_cloned;
                let subscriber_key = subscriber_key_cloned;

                while let Some(t) = server_rx.recv().await {
                    match client_tx.send(Ok(t)).await {
                        Ok(_) => {},
                        Err(_) => {
                            // client stream dropped
                            break;
                        },
                    }
                }
                println!("stream ended");
                inner.remove(&subscriber_key).await;
            })
        };

        let subscription =
            Arc::new(Subscription { tx: server_tx, task: handle });

        self.inner.add(subscriber_key, subscription.clone()).await;

        SubscriptionWithOutputStream {
            subscription,
            stream: ReceiverStream::new(client_rx),
        }
    }
}

impl IntoService<DynEventsService> for EventsServiceImpl {
    #[inline]
    fn into_service(self) -> DynEventsService {
        Arc::new(self) as DynEventsService
    }
}

impl EventsServiceImpl {
    #[inline]
    pub fn new() -> Self {
        Self { store: Arc::default() }
    }
}

#[async_trait]
impl EventsService for EventsServiceImpl {
    async fn create_subscription(
        &self,
        key: String,
        buffer_server: usize,
        buffer_client: usize,
    ) -> Result<SubscriptionWithOutputStream<Event>, EventsError> {
        Ok(self.store.create(key, buffer_server, buffer_client).await)
    }

    async fn remove_subscription(
        &self,
        subscriber_key: &String,
    ) -> Result<Option<Arc<Subscription<Event>>>, EventsError> {
        Ok(self.store.remove(subscriber_key).await)
    }

    async fn publish(
        &self,
        subscriber_key: &String,
        event: Event,
    ) -> Result<ExecSuccess, EventsError> {
        let subscription = {
            self.store
                .read()
                .await
                .get(subscriber_key)
                .cloned()
                .ok_or(EventsError::SubscriptionNotExists)?
        };

        subscription
            .tx
            .send(event)
            .await
            .map_err(|err| EventsError::SendEventError(err.to_string()))?;

        Ok(ExecSuccess::default())
    }
}

/* #[derive(Debug, Clone)]
pub struct EventsServiceRemote(EventsRpcClient<Channel>);

impl RpcClient for EventsServiceRemote {
    type Client = EventsRpcClient<Channel>;

    #[inline]
    fn client(&self) -> Self::Client {
        self.0.clone()
    }
}

impl EventsService for EventsServiceRemote {}

impl IntoService<DynEventsService> for EventsServiceRemote {
    #[inline]
    fn into_service(self) -> DynEventsService {
        Arc::new(self) as DynEventsService
    }
}
 */
