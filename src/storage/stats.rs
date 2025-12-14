//! Statistics and monitoring operations

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::storage::Index;

/// Get cluster statistics
pub async fn get_cluster_stats(
    indices: &Arc<RwLock<HashMap<String, Index>>>,
    es_version: &str,
) -> serde_json::Value {
    let indices_guard = indices.read().await;
    let total_indices = indices_guard.len();
    let total_docs: usize = indices_guard.values().map(|idx| idx.documents.len()).sum();

    serde_json::json!({
        "cluster_name": "gbs",
        "cluster_uuid": "gbs-cluster",
        "timestamp": Utc::now().timestamp_millis(),
        "status": "green",
        "indices": {
            "count": total_indices,
            "shards": {
                "total": total_indices,
                "primaries": total_indices,
                "replication": 0,
                "index": {
                    "shards": {
                        "min": 1,
                        "max": 1,
                        "avg": 1.0
                    },
                    "primaries": {
                        "min": 1,
                        "max": 1,
                        "avg": 1.0
                    },
                    "replication": {
                        "min": 0,
                        "max": 0,
                        "avg": 0.0
                    }
                }
            },
            "docs": {
                "count": total_docs,
                "deleted": 0
            },
            "store": {
                "size_in_bytes": 0,
                "throttle_time_in_millis": 0
            },
            "fielddata": {
                "memory_size_in_bytes": 0,
                "evictions": 0
            },
            "query_cache": {
                "memory_size_in_bytes": 0,
                "total_count": 0,
                "hit_count": 0,
                "miss_count": 0,
                "cache_size": 0,
                "cache_count": 0,
                "evictions": 0
            },
            "completion": {
                "size_in_bytes": 0
            },
            "segments": {
                "count": 0,
                "memory_in_bytes": 0,
                "terms_memory_in_bytes": 0,
                "stored_fields_memory_in_bytes": 0,
                "term_vectors_memory_in_bytes": 0,
                "norms_memory_in_bytes": 0,
                "points_memory_in_bytes": 0,
                "doc_values_memory_in_bytes": 0,
                "index_writer_memory_in_bytes": 0,
                "version_map_memory_in_bytes": 0,
                "fixed_bit_set_memory_in_bytes": 0,
                "max_unsafe_auto_id_timestamp": -1,
                "file_sizes": {}
            },
            "mappings": {
                "field_types": [],
                "runtime_field_types": []
            },
            "analysis": {
                "char_filter_types": [],
                "tokenizer_types": [],
                "filter_types": [],
                "analyzer_types": [],
                "built_in_char_filters": [],
                "built_in_tokenizers": [],
                "built_in_filters": [],
                "built_in_analyzers": []
            }
        },
        "nodes": {
            "count": {
                "total": 1,
                "data": 1,
                "coordinating_only": 0,
                "master": 1,
                "ingest": 1
            },
            "versions": [es_version],
            "os": {
                "available_processors": num_cpus::get(),
                "allocated_processors": num_cpus::get(),
                "names": []
            },
            "process": {
                "cpu": {
                    "percent": 0
                },
                "open_file_descriptors": {
                    "min": 0,
                    "max": 0,
                    "avg": 0
                }
            },
            "jvm": {
                "max_uptime_in_millis": 0,
                "versions": []
            },
            "fs": {
                "total_in_bytes": 0,
                "free_in_bytes": 0,
                "available_in_bytes": 0
            },
            "plugins": [],
            "network_types": {}
        }
    })
}
