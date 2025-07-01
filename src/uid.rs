use rand::random;
use tower_sessions::{session::Error, session_store, Session};

const UID_KEY: &str = "UID";

pub async fn get_or_create_uid(session: &Session) -> Result<u128, Error> {
    let id = get_uid(session).await.transpose();
    if let Some(id) = id {
        return id;
    }
    let id: u128 = random();
    // Potential data race here, but I think it's mostly ok
    session.insert(UID_KEY, id.to_ne_bytes()).await?;
    Ok(id)
}

pub async fn get_uid(session: &Session) -> Result<Option<u128>, Error> {
    let v = session.get(UID_KEY).await?;
    let v: Vec<u8> = match v {
        None => return Ok(None),
        Some(v) => v,
    };
    Ok(Some(u128::from_le_bytes(v.try_into().map_err(|v| {
        Error::Store(session_store::Error::Decode(format!(
            "Failed to deserialize {v:?}"
        )))
    })?)))
}
