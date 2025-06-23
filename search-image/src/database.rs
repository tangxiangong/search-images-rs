use crate::{
    app::ImageInfo,
    error::{Error, Result},
};
use qdrant_client::{
    Payload, Qdrant,
    qdrant::{
        Condition, DeletePointsBuilder, Filter, GetPointsBuilder, PointStruct, PointsIdsList,
        QueryPointsBuilder, RetrievedPoint, ScoredPoint, UpsertPointsBuilder, r#match::MatchValue,
    },
};
use serde::Serialize;

pub async fn add<T: Serialize>(
    client: &Qdrant,
    collection: &str,
    data: &[Vec<f32>],
    image_info: &[ImageInfo<T>],
) -> Result<()> {
    if data.len() != image_info.len() {
        return Err(Error::UpsertPointsError(
            "`data` and `image_info` must have the same length".to_string(),
        ));
    }
    let points = data
        .iter()
        .zip(image_info.iter())
        .map(|(data, image_info)| {
            let json_val = serde_json::to_value(image_info)?;
            let payload = Payload::try_from(json_val)
                .map_err(|e| Error::JsonToPayloadError(e.to_string()))?;
            let point = PointStruct::new(image_info.id().to_string(), data.to_vec(), payload);
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

    Ok(())
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

pub async fn delete_by_extras<T: Into<MatchValue>>(
    client: &Qdrant,
    collection: &str,
    extra: T,
) -> Result<()> {
    let response = client
        .delete_points(
            DeletePointsBuilder::new(collection)
                .points(Filter::must([Condition::matches("extra", extra.into())]))
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
    ids: &[&str],
    with_payload: bool,
    with_vectors: bool,
) -> Result<Vec<RetrievedPoint>> {
    let point_ids = ids
        .iter()
        .map(|id| id.to_string().into())
        .collect::<Vec<_>>();

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
