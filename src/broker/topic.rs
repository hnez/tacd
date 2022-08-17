use std::marker::PhantomData;

use async_std::channel::{unbounded, Receiver, Sender, TrySendError};
use async_std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;

use serde::{de::DeserializeOwned, Serialize};

use unique_token::Unique;

use super::TopicName;

pub struct Topic<E> {
    pub(super) path: TopicName,
    pub(super) web_readable: bool,
    pub(super) web_writable: bool,
    pub(super) senders: Mutex<Vec<(Unique, Sender<Arc<E>>)>>,
    pub(super) retained: Mutex<Option<Arc<E>>>,
    pub(super) senders_serialized: Mutex<Vec<(Unique, Sender<(TopicName, Arc<[u8]>)>)>>,
    pub(super) retained_serialized: Mutex<Option<Arc<[u8]>>>,
}

pub struct Native;
pub struct Encoded;

pub struct SubscriptionHandle<E, T> {
    topic: Weak<Topic<E>>,
    token: Unique,
    phantom: PhantomData<T>,
}

impl<E> SubscriptionHandle<E, Native> {
    #[allow(dead_code)]
    pub async fn unsubscribe(&self) {
        if let Some(topic) = self.topic.upgrade() {
            let mut senders = topic.senders.lock().await;

            if let Some(idx) = senders.iter().position(|(token, _)| *token == self.token) {
                senders.swap_remove(idx);
            }
        }
    }
}

impl<E> SubscriptionHandle<E, Encoded> {
    pub async fn unsubscribe(&self) {
        if let Some(topic) = self.topic.upgrade() {
            let mut senders = topic.senders_serialized.lock().await;

            if let Some(idx) = senders.iter().position(|(token, _)| *token == self.token) {
                senders.swap_remove(idx);
            }
        }
    }
}

#[async_trait]
pub trait AnySubscriptionHandle: Sync + Send {
    async fn unsubscribe(&self);
}

#[async_trait]
impl<E: Send + Sync> AnySubscriptionHandle for SubscriptionHandle<E, Encoded> {
    async fn unsubscribe(&self) {
        Self::unsubscribe(self).await
    }
}

impl<E: Serialize + DeserializeOwned> Topic<E> {
    async fn set_arc_with_retain_lock(&self, msg: Arc<E>, retained: &mut Option<Arc<E>>) {
        let mut senders = self.senders.lock().await;
        let mut senders_serialized = self.senders_serialized.lock().await;
        let mut retained_serialized = self.retained_serialized.lock().await;

        *retained_serialized = None;

        senders.retain(|(_, s)| match s.try_send(msg.clone()) {
            Ok(_) => true,
            Err(TrySendError::Full(_)) => {
                s.close();
                false
            }
            Err(TrySendError::Closed(_)) => false,
        });

        if !senders_serialized.is_empty() {
            let encoded = serde_json::to_vec(&msg).unwrap();
            let encoded: Arc<[u8]> = Arc::from(encoded.into_boxed_slice());

            senders_serialized.retain(|(_, s)| {
                match s.try_send((self.path.clone(), encoded.clone())) {
                    Ok(_) => true,
                    Err(TrySendError::Full(_)) => {
                        s.close();
                        false
                    }
                    Err(TrySendError::Closed(_)) => false,
                }
            });

            *retained_serialized = Some(encoded);
        }

        *retained = Some(msg);
    }

    pub async fn set_arc(&self, msg: Arc<E>) {
        let mut retained = self.retained.lock().await;

        self.set_arc_with_retain_lock(msg, &mut *retained).await
    }

    pub async fn set(&self, msg: E) {
        self.set_arc(Arc::new(msg)).await
    }

    pub async fn get(&self) -> Option<Arc<E>> {
        self.retained.lock().await.as_ref().cloned()
    }

    pub async fn modify<F>(&self, cb: F)
    where
        F: FnOnce(Arc<E>) -> Arc<E>,
    {
        let mut retained = self.retained.lock().await;

        if let Some(prev) = retained.as_ref().cloned() {
            self.set_arc_with_retain_lock(cb(prev), &mut *retained)
                .await;
        }
    }

    pub async fn subscribe(
        self: Arc<Self>,
        sender: Sender<Arc<E>>,
    ) -> SubscriptionHandle<E, Native> {
        let token = Unique::new();
        self.senders.lock().await.push((token, sender));

        SubscriptionHandle {
            topic: Arc::downgrade(&self),
            token: token,
            phantom: PhantomData,
        }
    }

    pub async fn subscribe_unbounded(
        self: Arc<Self>,
    ) -> (Receiver<Arc<E>>, SubscriptionHandle<E, Native>) {
        let (tx, rx) = unbounded();
        (rx, self.subscribe(tx).await)
    }

    async fn set_from_bytes<'a>(&self, msg: &[u8]) -> Result<(), ()> {
        match serde_json::from_slice(msg) {
            Ok(m) => Ok(self.set(m).await),
            Err(_) => Err(()),
        }
    }

    async fn subscribe_as_bytes(
        self: Arc<Self>,
        sender: Sender<(TopicName, Arc<[u8]>)>,
    ) -> SubscriptionHandle<E, Encoded> {
        let token = Unique::new();
        self.senders_serialized.lock().await.push((token, sender));

        SubscriptionHandle {
            topic: Arc::downgrade(&self),
            token: token,
            phantom: PhantomData,
        }
    }

    async fn get_as_bytes(&self) -> Option<Arc<[u8]>> {
        let mut retained_serialized = self.retained_serialized.lock().await;

        if retained_serialized.is_none() {
            if let Some(native) = self.get().await {
                let encoded = serde_json::to_vec(&native).unwrap();
                *retained_serialized = Some(Arc::from(encoded.into_boxed_slice()));
            }
        }

        retained_serialized.as_ref().cloned()
    }
}

#[async_trait]
pub trait AnyTopic: Sync + Send {
    fn path(&self) -> &TopicName;
    fn web_readable(&self) -> bool;
    fn web_writable(&self) -> bool;
    async fn set_from_bytes(&self, msg: &[u8]) -> Result<(), ()>;
    async fn subscribe_as_bytes(
        self: Arc<Self>,
        sender: Sender<(TopicName, Arc<[u8]>)>,
    ) -> Box<dyn AnySubscriptionHandle>;
    async fn get_as_bytes(&self) -> Option<Arc<[u8]>>;
}

#[async_trait]
impl<E: Serialize + DeserializeOwned + Send + Sync + 'static> AnyTopic for Topic<E> {
    fn path(&self) -> &TopicName {
        &self.path
    }

    fn web_readable(&self) -> bool {
        self.web_readable
    }

    fn web_writable(&self) -> bool {
        self.web_writable
    }

    async fn set_from_bytes(&self, msg: &[u8]) -> Result<(), ()> {
        self.set_from_bytes(msg).await
    }

    async fn subscribe_as_bytes(
        self: Arc<Self>,
        sender: Sender<(TopicName, Arc<[u8]>)>,
    ) -> Box<dyn AnySubscriptionHandle> {
        Box::new(self.subscribe_as_bytes(sender).await)
    }

    async fn get_as_bytes(&self) -> Option<Arc<[u8]>> {
        self.get_as_bytes().await
    }
}
