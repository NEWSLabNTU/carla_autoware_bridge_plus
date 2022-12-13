use r2r::{
    qos::{DurabilityPolicy, HistoryPolicy, ReliabilityPolicy},
    QosProfile,
};

// pub fn default() -> QosProfile {
//     QosProfile::default()
// }

pub fn best_effort() -> QosProfile {
    QosProfile {
        history: HistoryPolicy::KeepLast,
        depth: 1,
        reliability: ReliabilityPolicy::BestEffort,
        durability: DurabilityPolicy::Volatile,
        ..QosProfile::default()
    }
}
