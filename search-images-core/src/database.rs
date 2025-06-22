use crate::error::{Error, Result};
use qdrant_client::{
    Payload, Qdrant,
    qdrant::{PointStruct, UpsertPointsBuilder},
};
use serde::Serialize;
use uuid::Uuid;

pub async fn add_one<T: Serialize>(
    client: &Qdrant,
    collection: &str,
    data: &[f32],
    payload: T,
) -> Result<()> {
    let point = create_point(data, payload)?;

    upsert_points(client, collection, vec![point]).await?;
    Ok(())
}

pub async fn add<T: Serialize>(
    client: &Qdrant,
    collection: &str,
    data: &[Vec<f32>],
    payload: &[T],
) -> Result<()> {
    if data.len() != payload.len() {
        return Err(Error::UpsertPointsError(
            "data and payload must have the same length".to_string(),
        ));
    }
    let points = data
        .iter()
        .zip(payload.iter())
        .map(|(data, payload)| create_point(data, payload))
        .collect::<Result<Vec<_>>>()?;

    upsert_points(client, collection, points).await?;
    Ok(())
}

fn create_point<T: Serialize>(data: &[f32], payload: T) -> Result<PointStruct> {
    let json_val = serde_json::to_value(payload)?;
    let payload =
        Payload::try_from(json_val).map_err(|e| Error::JsonToPayloadError(e.to_string()))?;
    let point = PointStruct::new(Uuid::new_v4().to_string(), data.to_vec(), payload);
    Ok(point)
}

async fn upsert_points(client: &Qdrant, collection: &str, points: Vec<PointStruct>) -> Result<()> {
    let res = client
        .upsert_points(UpsertPointsBuilder::new(collection, points).wait(true))
        .await
        .map_err(|e| Error::UpsertPointsError(e.to_string()))?;
    if res.result.is_none() {
        return Err(Error::UpsertPointsError("upsert points failed".to_string()));
    }

    Ok(())
}

pub async fn delete(_client: &Qdrant, _collection: &str, _id: &str) -> Result<()> {
    todo!()
}
