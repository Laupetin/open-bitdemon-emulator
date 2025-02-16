use bitdemon::lobby::group::GroupService;
use bitdemon::networking::bd_session::{BdSession, SessionId};
use bitdemon::networking::session_manager::SessionManager;
use log::{error, info};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};

type GroupId = u32;

pub struct DwGroupService {
    aggregated_group_counts: RwLock<HashMap<GroupId, u64>>,
    session_groups: Mutex<HashMap<SessionId, Vec<GroupId>>>,
}

impl GroupService for DwGroupService {
    fn get_group_counts(
        &self,
        _session: &BdSession,
        groups: &[u32],
    ) -> Result<Vec<u64>, Box<dyn Error>> {
        info!("Retrieving counts for {} groups", groups.len());

        let aggregated_group_counts = self.aggregated_group_counts.read().unwrap();

        Ok(groups
            .iter()
            .map(|group_id| {
                aggregated_group_counts
                    .get(group_id)
                    .copied()
                    .unwrap_or(0u64)
            })
            .collect())
    }

    fn set_groups(&self, session: &BdSession, groups: &[u32]) -> Result<(), Box<dyn Error>> {
        info!("Setting {} groups for session", groups.len());

        let previous_groups;
        let groups_clone = groups.iter().cloned().collect();

        {
            let mut session_groups = self.session_groups.lock().unwrap();

            previous_groups = session_groups
                .remove(&session.id)
                .map(|vec| HashSet::from_iter(vec.into_iter()))
                .unwrap_or_else(|| HashSet::new());

            session_groups.insert(session.id, groups_clone);
        }

        let new_groups: HashSet<GroupId> = HashSet::from_iter(
            groups
                .iter()
                .cloned()
                .filter(|group_id| !previous_groups.contains(group_id)),
        );

        let left_groups: Vec<GroupId> = previous_groups
            .into_iter()
            .filter(|group_id| !new_groups.contains(group_id))
            .collect();

        let mut aggregated_group_counts = self.aggregated_group_counts.write().unwrap();
        for group_id in new_groups {
            if let Some(previous_value) = aggregated_group_counts.get_mut(&group_id) {
                *previous_value += 1;
            } else {
                aggregated_group_counts.insert(group_id, 1);
            }
        }
        for group_id in left_groups {
            if let Some(previous_value) = aggregated_group_counts.get_mut(&group_id) {
                if *previous_value > 0 {
                    *previous_value -= 1;
                }
            } else {
                error!("Aggregated group counts appear to be wrong!");
            }
        }

        Ok(())
    }
}

impl DwGroupService {
    pub fn new(session_manager: Arc<SessionManager>) -> Arc<DwGroupService> {
        let service = Arc::new(DwGroupService {
            aggregated_group_counts: RwLock::new(HashMap::new()),
            session_groups: Mutex::new(HashMap::new()),
        });

        Self::register_session_manager_callbacks(service.clone(), session_manager);

        service
    }

    fn register_session_manager_callbacks(
        service: Arc<Self>,
        session_manager: Arc<SessionManager>,
    ) {
        session_manager.on_session_unregistered(move |session| {
            service.remove_all_groups_for_session(session.id);
        });
    }

    fn remove_all_groups_for_session(&self, session_id: SessionId) {
        let maybe_groups;
        {
            let mut session_groups = self.session_groups.lock().unwrap();
            maybe_groups = session_groups.remove(&session_id);
        }

        if let Some(groups) = maybe_groups {
            info!("Removing {} groups due to disconnect", groups.len());
            let mut aggregated_group_counts = self.aggregated_group_counts.write().unwrap();

            for group_id in groups {
                if let Some(group_count) = aggregated_group_counts.get_mut(&group_id) {
                    if *group_count > 0 {
                        *group_count -= 1;
                    } else {
                        error!("Aggregated group counts appear to be wrong!");
                    }
                } else {
                    error!("Aggregated group counts appear to be wrong!");
                }
            }
        }
    }
}
