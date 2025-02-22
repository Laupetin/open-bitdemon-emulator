use crate::lobby::profile::db::{ProfileType, PROFILE_DB};
use bitdemon::auth::authentication::SessionAuthentication;
use bitdemon::lobby::profile::{ProfileInfo, ProfileService, ProfileServiceError};
use bitdemon::networking::bd_session::BdSession;
use chrono::Utc;
use log::info;
use num_traits::ToPrimitive;
use rusqlite::DropBehavior;

pub struct DwProfileService {}

impl ProfileService for DwProfileService {
    fn get_public_profiles(
        &self,
        session: &BdSession,
        user_ids: Vec<u64>,
    ) -> Result<Vec<ProfileInfo>, ProfileServiceError> {
        info!("Requesting public profiles for {} users", user_ids.len());

        let authentication = session.authentication().expect("user to be authenticated");
        let title_num = authentication.title.to_u32().expect("title to be u32");
        let res: Vec<ProfileInfo> = PROFILE_DB.with_borrow_mut(|db| {
            let mut transaction = db.transaction().expect("transaction to be started");
            transaction.set_drop_behavior(DropBehavior::Commit);

            user_ids
                .iter()
                .copied()
                .flat_map(|user_id| {
                    transaction.query_row(
                        "SELECT data FROM user_profile u
                     WHERE u.title = ?1 AND u.owner_id = ?2 AND u.profile_type = ?3",
                        (title_num, user_id, u8::from(ProfileType::Public)),
                        |row| {
                            Ok(ProfileInfo {
                                user_id,
                                data: row.get(0)?,
                            })
                        },
                    )
                })
                .collect()
        });

        if !res.is_empty() || user_ids.is_empty() {
            Ok(res)
        } else {
            Err(ProfileServiceError::NoProfileInfoFound)
        }
    }

    fn get_private_profile(&self, session: &BdSession) -> Result<ProfileInfo, ProfileServiceError> {
        info!("Requesting own private profile");

        let authentication = session.authentication().expect("user to be authenticated");
        let title_num = authentication.title.to_u32().expect("title to be u32");
        let user_id = authentication.user_id;
        PROFILE_DB
            .with_borrow(|db| {
                db.query_row(
                    "SELECT data FROM user_profile u
                     WHERE u.title = ?1 AND u.owner_id = ?2 AND u.profile_type = ?3",
                    (title_num, user_id, u8::from(ProfileType::Private)),
                    |row| {
                        Ok(ProfileInfo {
                            user_id,
                            data: row.get(0)?,
                        })
                    },
                )
            })
            .map_err(|_| ProfileServiceError::NoProfileInfoFound)
    }

    fn set_public_profile(
        &self,
        session: &BdSession,
        public_profile_data: Vec<u8>,
    ) -> Result<(), ProfileServiceError> {
        info!("Setting own public profile");

        let authentication = session.authentication().expect("user to be authenticated");

        Self::update_user_profile(authentication, ProfileType::Public, public_profile_data);

        Ok(())
    }

    fn set_private_profile(
        &self,
        session: &BdSession,
        private_profile_data: Vec<u8>,
    ) -> Result<(), ProfileServiceError> {
        info!("Setting own private profile");

        let authentication = session.authentication().expect("user to be authenticated");

        Self::update_user_profile(authentication, ProfileType::Private, private_profile_data);

        Ok(())
    }

    fn delete_profile(&self, session: &BdSession) -> Result<(), ProfileServiceError> {
        info!("Deleting own profile");

        let authentication = session.authentication().expect("user to be authenticated");
        let title_num = authentication.title.to_u32().expect("title to be u32");
        let user_id = authentication.user_id;

        PROFILE_DB.with_borrow(|db| {
            db.execute(
                "DELETE FROM user_profile u
                     WHERE u.title = ?1 AND u.owner_id = ?2",
                (title_num, user_id),
            )
            .expect("operation to not fail")
        });

        Ok(())
    }
}

impl DwProfileService {
    pub fn new() -> DwProfileService {
        DwProfileService {}
    }

    fn update_user_profile(
        authentication: &SessionAuthentication,
        profile_type: ProfileType,
        public_profile_data: Vec<u8>,
    ) {
        let title_num = authentication.title.to_u32().expect("title to be u32");
        let user_id = authentication.user_id;
        let profile_type_num: u8 = profile_type.into();
        let now = Utc::now().timestamp();

        PROFILE_DB
            .with_borrow_mut(|db| {
                let transaction = db.transaction().expect("transaction to be started");

                let maybe_existing_id: rusqlite::Result<u64> = transaction.query_row(
                    "SELECT u.id FROM user_profile u WHERE u.title = ? AND owner_id = ? AND profile_type = ?",
                    (title_num, user_id, profile_type_num),
                    |row| row.get(0),
                );

                if let Ok(existing_id) = maybe_existing_id {
                    transaction.execute(
                        "UPDATE user_profile SET modified_at = ?2, data = ?3 WHERE id = ?1",
                        (existing_id, now, public_profile_data),
                    ).expect("update to be successful");
                } else {
                    transaction.execute(
                        "INSERT INTO user_profile
                        (title, owner_id, profile_type, created_at, modified_at, data)
                        VALUES (?, ?, ?, ?, ?, ?)",
                        (title_num, user_id, profile_type_num, now, now, public_profile_data),
                    ).expect("insert to be successful");
                }

                transaction.commit().expect("commit to be successful");
            });
    }
}
