use crate::domain::result_slice::ResultSlice;
use crate::lobby::response::task_reply::TaskReply;
use crate::lobby::LobbyHandler;
use crate::messaging::bd_message::BdMessage;
use crate::messaging::bd_reader::BdReader;
use crate::messaging::bd_response::{BdResponse, ResponseCreator};
use crate::messaging::bd_serialization::BdDeserialize;
use crate::messaging::BdErrorCode;
use crate::networking::bd_session::BdSession;
use log::{info, warn};
use num_traits::FromPrimitive;
use snafu::{OptionExt, Snafu};
use std::error::Error;

pub struct VoteRankHandler {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum VoteRankTaskId {
    // GetLikeDislikeRatioFromRating
    SubmitRating = 1,
    SubmitCategorizedRating = 2,
    GetVoteHistory = 3,
}

#[derive(Debug, Snafu)]
enum VoteRankError {
    #[snafu(display("There is no such vote entry for value={value}"))]
    InvalidVote { value: u8 },
}

impl LobbyHandler for VoteRankHandler {
    fn handle_message(
        &self,
        session: &mut BdSession,
        mut message: BdMessage,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let task_id_value = message.reader.read_u8()?;
        let maybe_task_id = VoteRankTaskId::from_u8(task_id_value);
        if maybe_task_id.is_none() {
            warn!("Client called unknown task {task_id_value}");
            return TaskReply::with_only_error_code(BdErrorCode::NoError, task_id_value)
                .to_response();
        }
        let task_id = maybe_task_id.unwrap();

        match task_id {
            VoteRankTaskId::SubmitRating => self.submit_rating(session, &mut message.reader),
            VoteRankTaskId::SubmitCategorizedRating => {
                self.submit_categorized_rating(session, &mut message.reader)
            }
            VoteRankTaskId::GetVoteHistory => self.get_vote_history(session, &mut message.reader),
        }
    }
}

impl VoteRankHandler {
    pub fn new() -> VoteRankHandler {
        VoteRankHandler {}
    }

    fn submit_rating(
        &self,
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut votes = Vec::new();

        while let Ok(rating_info) = RatingInfo::deserialize(reader) {
            votes.push(rating_info);
        }

        info!("User submitted rating: {votes:?}");

        TaskReply::with_only_error_code(BdErrorCode::NoError, VoteRankTaskId::SubmitRating)
            .to_response()
    }

    fn submit_categorized_rating(
        &self,
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let mut votes = Vec::new();

        while let Ok(categorized_rating_info) = CategorizedRatingInfo::deserialize(reader) {
            votes.push(categorized_rating_info);
        }

        info!("User submitted categorized rating: {votes:?}");

        TaskReply::with_only_error_code(BdErrorCode::NoError, VoteRankTaskId::SubmitRating)
            .to_response()
    }

    fn get_vote_history(
        &self,
        _session: &mut BdSession,
        reader: &mut BdReader,
    ) -> Result<BdResponse, Box<dyn Error>> {
        let unknown = reader.read_u16()?;
        let item_offset = reader.read_u32()?;
        let item_count = reader.read_u32()?;

        info!("Retrieving vote history unknown={unknown} item_offset={item_offset} item_count={item_count}");

        // Returns result slice with CategorizedRatingInfo
        TaskReply::with_result_slice(
            VoteRankTaskId::GetVoteHistory,
            ResultSlice::new(Vec::new(), 0),
        )
        .to_response()
    }
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Vote {
    DISLIKE = 0x0,
    LIKE = 0xA,
}

#[derive(Debug)]
struct RatingInfo {
    entity_id: u64,
    rating: Vote,
}

#[derive(Debug)]
struct CategorizedRatingInfo {
    rating_info: RatingInfo,
    category: u16,
}

impl BdDeserialize for RatingInfo {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let entity_id = reader.read_u64()?;
        let rating_value = reader.read_u8()?;
        let rating = Vote::from_u8(rating_value).with_context(|| InvalidVoteSnafu {
            value: rating_value,
        })?;

        Ok(RatingInfo { entity_id, rating })
    }
}

impl BdDeserialize for CategorizedRatingInfo {
    fn deserialize(reader: &mut BdReader) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let rating_info = RatingInfo::deserialize(reader)?;
        let category = reader.read_u16()?;

        Ok(CategorizedRatingInfo {
            rating_info,
            category,
        })
    }
}
