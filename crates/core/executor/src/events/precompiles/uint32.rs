use serde::{Deserialize, Serialize};

use crate::events::{
    memory::{MemoryReadRecord, MemoryWriteRecord},
    LookupId, MemoryLocalEvent,
};

/// Uint32 Sqr Event.
///
/// This event is emitted when a uint32 sqr operation is performed.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Uint32SqrEvent {
    /// The lookup identifier.
    pub lookup_id: LookupId,
    /// The shard number.
    pub shard: u32,
    /// The clock cycle.
    pub clk: u32,
    /// The pointer to the x value.
    pub x_ptr: u32,
    /// The x value as a list of words.
    pub x: Vec<u32>,
    /// The pointer to the modulus value.
    pub modulus_ptr: u32,
    /// The modulus as a list of words.
    pub modulus: Vec<u32>,
    /// The memory records for the x value.
    pub x_memory_records: Vec<MemoryWriteRecord>,
    /// The memory records for the modulus value.
    pub modulus_memory_records: Vec<MemoryReadRecord>,
    /// The local memory access records.
    pub local_mem_access: Vec<MemoryLocalEvent>,
}