use std::collections::HashMap;

use futures::Future;
use itertools::Itertools;
use segment::data_types::groups::PseudoId;
use segment::types::{PointIdType, WithPayloadInterface, WithVector};
use tokio::sync::RwLockReadGuard;

use crate::collection::Collection;
use crate::operations::consistency_params::ReadConsistency;
use crate::operations::types::{CollectionError, CollectionResult, PointRequest, Record};
use crate::shards::shard::ShardId;

#[derive(Debug, Clone)]
pub struct LookupRequest {
    pub collection_name: String,
    pub values: Vec<PseudoId>,
    pub with_payload: WithPayloadInterface,
    pub with_vectors: WithVector,
}

pub async fn lookup_ids<'a, F, Fut>(
    request: LookupRequest,
    collection_by_name: F,
    read_consistency: Option<ReadConsistency>,
    shard_selection: Option<ShardId>,
) -> CollectionResult<HashMap<PseudoId, Record>>
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = Option<RwLockReadGuard<'a, Collection>>>,
{
    let collection = collection_by_name(request.collection_name.clone())
        .await
        .ok_or(CollectionError::NotFound {
            what: format!("Collection {}", request.collection_name),
        })?;

    let ids = request
        .values
        .into_iter()
        .filter_map(|v| PointIdType::try_from(v).ok())
        .collect_vec();

    let ids_len = ids.len();

    if ids_len == 0 {
        return Ok(HashMap::new());
    }

    let point_request = PointRequest {
        ids,
        with_payload: Some(request.with_payload),
        with_vector: request.with_vectors,
    };

    let result = collection
        .retrieve(point_request, read_consistency, shard_selection)
        .await?
        .into_iter()
        .map(|point| (PseudoId::from(point.id), point))
        .collect();

    Ok(result)
}
