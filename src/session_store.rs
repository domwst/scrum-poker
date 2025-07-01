use async_nats::jetstream::kv::{CreateErrorKind, Store};
use async_trait::async_trait;
use rand::random;
use tower_sessions::{
    session::{Id, Record},
    session_store::{Error, Result},
    SessionStore,
};

fn to_nats_key(v: &Id) -> String {
    format!("{:X}", v.0)
}

fn serialize(record: &Record) -> Result<Vec<u8>> {
    serde_json::to_vec(record).map_err(|e| Error::Encode(e.to_string()))
}

#[derive(Debug, Clone)]
pub struct NatsSessionStore {
    client: Store,
}

impl NatsSessionStore {
    pub fn new(client: Store) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SessionStore for NatsSessionStore {
    async fn create(&self, session_record: &mut Record) -> Result<()> {
        loop {
            let result = self
                .client
                .create(
                    to_nats_key(&session_record.id),
                    serialize(session_record)?.into(),
                )
                .await;
            match result {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == CreateErrorKind::AlreadyExists => {
                    tracing::warn!("Collision on record key {}", session_record.id.0);
                }
                Err(e) => {
                    return Err(Error::Backend(e.to_string()));
                }
            }
            session_record.id.0 = random();
        }
    }

    async fn save(&self, session_record: &Record) -> Result<()> {
        self.client
            .put(
                to_nats_key(&session_record.id),
                serialize(session_record)?.into(),
            )
            .await
            .map_err(|e| Error::Backend(e.to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        let record = self
            .client
            .get(to_nats_key(session_id))
            .await
            .map_err(|e| Error::Backend(e.to_string()))?;
        let record = match record {
            None => return Ok(None),
            Some(r) => r,
        };
        let record = serde_json::from_slice(&record).map_err(|e| Error::Decode(e.to_string()))?;
        Ok(Some(record))
    }

    async fn delete(&self, session_id: &Id) -> Result<()> {
        self.client
            .delete(to_nats_key(session_id))
            .await
            .map_err(|e| Error::Backend(e.to_string()))
    }
}
