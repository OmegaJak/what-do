use crate::{
    pages::{ranking_page::RankingPage, results_page::ResultsPage, veto_page::VetoPage, AppPage},
    room_state::{RoomState, VotingStage},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

const ROOM_CODE_CHARSET: &str = "abcdefghijklmnopqrstuvwxyz";

#[derive(Default)]
pub struct ServerState {
    pub rooms: HashMap<String, Arc<RwLock<RoomState>>>,
}

impl ServerState {
    pub fn create_room(
        &mut self,
        original_input_text: String,
    ) -> Result<(String, &mut Arc<RwLock<RoomState>>), ()> {
        let room_code = self.get_valid_room_code()?;
        Ok((
            room_code.clone(),
            self.rooms.entry(room_code.clone()).or_insert_with(|| {
                Arc::new(RwLock::new(RoomState::new(room_code, original_input_text)))
            }),
        ))
    }

    pub fn get_room_voting_page(
        &self,
        room_code: &str,
    ) -> Result<Box<dyn AppPage + Send + Sync>, String> {
        if let Some(room) = self.rooms.get(room_code) {
            match room.read().unwrap().voting_stage() {
                VotingStage::Vetoing => {
                    Ok(Box::new(VetoPage::new(room_code.to_string(), room.clone())))
                }
                VotingStage::Ranking => Ok(Box::new(RankingPage::new(
                    room_code.to_string(),
                    room.clone(),
                ))),
            }
        } else {
            Err(format!("Room \"{}\" not found", room_code))
        }
    }

    pub fn get_room_results_page(
        &self,
        room_code: &str,
    ) -> Result<Box<dyn AppPage + Send + Sync>, String> {
        if let Some(room) = self.rooms.get(room_code) {
            Ok(Box::new(ResultsPage {
                room_code: room_code.to_string(),
                room_state: room.clone(),
            }))
        } else {
            Err(format!("Room \"{}\" not found", room_code))
        }
    }

    fn get_valid_room_code(&self) -> Result<String, ()> {
        const MAX_ROOM_CODE_ATTEMPTS: usize = 100;
        let mut attempts = 0;
        let mut room_code = Self::random_room_code();
        while self.rooms.contains_key(&room_code) && attempts < MAX_ROOM_CODE_ATTEMPTS {
            room_code = Self::random_room_code();
            attempts += 1;
        }

        if attempts >= MAX_ROOM_CODE_ATTEMPTS {
            Err(())
        } else {
            Ok(room_code)
        }
    }

    fn random_room_code() -> String {
        random_string::generate(4, ROOM_CODE_CHARSET)
    }
}
