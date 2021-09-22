use std::{collections::HashMap, ops::DerefMut, slice::Iter, sync::{Arc, RwLock}};
use bevy::{ecs::schedule::SystemDescriptor, prelude::*};
use dashmap::DashMap;

use crate::pops::BasePop;
pub struct Messaging;

const MESSAGING_MAIN: &'static str = "MESSAGING_MAIN";

pub fn add_message_handler(app: &mut AppBuilder, system: impl Into<SystemDescriptor>) {
    app.add_system_to_stage(MESSAGING_MAIN, system);
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum MessageType {
    PopTransfer,
}

pub trait Message {
    fn kind(&self) -> MessageType;
}

pub struct MessageQueue {
    queue: DashMap<MessageType, Vec<Box<dyn Message + Send + Sync>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            queue: DashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.queue = DashMap::new();
    }

    pub fn add_message(&self, message: Box<dyn Message + Send + Sync>) {
        if let Some(mut queue) = self.queue.get_mut(&message.kind()) {
            queue.push(message);
        } else {
            self.queue.insert(message.kind(), vec![message]);
        }
    }
}

fn clear_messages(
    mut message_queue: ResMut<MessageQueue>,
) {
    message_queue.clear();
}

pub struct PopTransfer {
    size: isize,
    src: Entity,
    dest: Entity,
}

impl Message for PopTransfer {
    fn kind(&self) -> MessageType {
        MessageType::PopTransfer
    }
}

fn pop_transfer_message(
    message_queue: Res<MessageQueue>,
    mut pop_query: Query<(Entity, &mut BasePop)>,
) {
    for pop_transfer in message_queue.queue.get(&MessageType::PopTransfer).unwrap().iter() {
        let pop = pop_query.get_component_mut::<BasePop>(pop_transfer.src).unwrap();
    }
}

pub struct MessagingPlugin;

impl Plugin for MessagingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::Last, clear_messages.system());
    }
}
