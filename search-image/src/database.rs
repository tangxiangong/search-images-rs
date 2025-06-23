use crate::error::{Error, Result};
use qdrant_client::{
    Payload, Qdrant,
    qdrant::{
        Condition, DeletePointsBuilder, Filter, GetPointsBuilder, PointStruct, PointsIdsList,
        QueryPointsBuilder, RetrievedPoint, ScoredPoint, UpsertPointsBuilder, r#match::MatchValue,
    },
};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn add<T: Serialize>(
    client: &Qdrant,
    collection: &str,
    data: &[Vec<f32>],
    payload: &[T],
) -> Result<Vec<String>> {
    if data.len() != payload.len() {
        return Err(Error::UpsertPointsError(
            "data and payload must have the same length".to_string(),
        ));
    }
    let ids = (0..data.len())
        .map(|_| Uuid::new_v4().to_string())
        .collect::<Vec<_>>();
    let points = data
        .iter()
        .zip(payload.iter().zip(ids.iter()))
        .map(|(data, (payload, id))| {
            let json_val = serde_json::to_value(payload)?;
            let payload = Payload::try_from(json_val)
                .map_err(|e| Error::JsonToPayloadError(e.to_string()))?;
            let point = PointStruct::new(id.to_string(), data.to_vec(), payload);
            Ok(point)
        })
        .collect::<Result<Vec<_>>>()?;

    let res = client
        .upsert_points(UpsertPointsBuilder::new(collection, points).wait(true))
        .await
        .map_err(|e| Error::UpsertPointsError(e.to_string()))?;
    if res.result.is_none() {
        return Err(Error::UpsertPointsError("upsert points failed".to_string()));
    }

    Ok(ids)
}

pub async fn delete_by_ids(client: &Qdrant, collection: &str, ids: &[String]) -> Result<()> {
    let point_ids = ids.iter().map(|id| id.clone().into()).collect::<Vec<_>>();
    let res = client
        .delete_points(
            DeletePointsBuilder::new(collection)
                .points(PointsIdsList { ids: point_ids })
                .wait(true),
        )
        .await
        .map_err(|e| Error::DeletePointsError(e.to_string()))?;
    if res.result.is_none() {
        return Err(Error::DeletePointsError("delete points failed".to_string()));
    }

    Ok(())
}

pub async fn delete_by_payloads<T: Into<MatchValue>>(
    client: &Qdrant,
    collection: &str,
    payloads: HashMap<String, T>,
) -> Result<()> {
    let response = client
        .delete_points(
            DeletePointsBuilder::new(collection)
                .points(Filter::must(
                    payloads
                        .into_iter()
                        .map(|(key, value)| Condition::matches(&key, value.into()))
                        .collect::<Vec<_>>(),
                ))
                .wait(true),
        )
        .await
        .map_err(|e| Error::DeletePointsError(e.to_string()))?;
    if response.result.is_none() {
        return Err(Error::DeletePointsError("delete points failed".to_string()));
    }
    Ok(())
}

pub async fn search_by_ids(
    client: &Qdrant,
    collection: &str,
    ids: &[String],
    with_payload: bool,
    with_vectors: bool,
) -> Result<Vec<RetrievedPoint>> {
    let point_ids = ids.iter().map(|id| id.clone().into()).collect::<Vec<_>>();

    let response = client
        .get_points(
            GetPointsBuilder::new(collection, point_ids)
                .with_payload(with_payload)
                .with_vectors(with_vectors),
        )
        .await
        .map_err(|e| Error::SearchPointsError(e.to_string()))?;
    Ok(response.result)
}

pub async fn similarity_search(
    client: &Qdrant,
    collection: &str,
    feature: &[f32],
    k: usize,
    with_payload: bool,
    with_vectors: bool,
) -> Result<Vec<ScoredPoint>> {
    let response = client
        .query(
            QueryPointsBuilder::new(collection)
                .query(feature.to_vec())
                .limit(k as u64)
                .with_payload(with_payload)
                .with_vectors(with_vectors),
        )
        .await
        .map_err(|e| Error::SearchPointsError(e.to_string()))?;
    Ok(response.result)
}
