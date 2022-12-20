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

pub fn latched() -> QosProfile {
    QosProfile {
        history: HistoryPolicy::KeepAll,
        depth: 1,
        reliability: ReliabilityPolicy::Reliable,
        durability: DurabilityPolicy::TransientLocal,
        ..QosProfile::default()
    }
}
