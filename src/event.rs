// src/event_processor.rs
use crate::{AptosClient, types::Event};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::broadcast;

/// event handler
pub struct EventHandler;

#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: String,
    pub event_data: Value,
    pub sequence_number: u64,
    pub transaction_hash: String,
    pub block_height: u64,
}

impl EventHandler {
    /// Real-time monitoring of event streams
    pub async fn start_event_stream(
        client: Arc<AptosClient>,
        address: String,
        event_handle: String,
        event_sender: broadcast::Sender<EventData>,
    ) -> Result<(), String> {
        let mut last_sequence: Option<u64> = None;
        loop {
            match client
                .get_account_event_vec(&address, &event_handle, Some(100), last_sequence)
                .await
            {
                Ok(events) => {
                    for event in events {
                        let sequence_number = match event.sequence_number.parse::<u64>() {
                            Ok(seq) => seq,
                            Err(_) => continue,
                        };
                        // 只处理新事件
                        if last_sequence
                            .map(|last| sequence_number > last)
                            .unwrap_or(true)
                        {
                            let event_data = EventData {
                                event_type: event.r#type.clone(),
                                event_data: event.data.clone(),
                                sequence_number,
                                transaction_hash: "hash".to_string(),
                                block_height: client.get_chain_height().await.unwrap() as u64,
                            };
                            let _ = event_sender.send(event_data);
                            last_sequence = Some(sequence_number);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching events: {}", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    /// Event stream containing transaction information
    pub async fn start_event_stream_with_tx_info(
        client: Arc<AptosClient>,
        address: String,
        event_handle: String,
        event_sender: broadcast::Sender<EventData>,
    ) -> Result<(), String> {
        let mut last_sequence: Option<u64> = None;
        loop {
            match client
                .get_account_event_vec(&address, &event_handle, Some(100), last_sequence)
                .await
            {
                Ok(events) => {
                    for event in events {
                        let sequence_number = match event.sequence_number.parse::<u64>() {
                            Ok(seq) => seq,
                            Err(_) => continue,
                        };
                        if last_sequence
                            .map(|last| sequence_number > last)
                            .unwrap_or(true)
                        {
                            // Get transaction information
                            let transaction_hash = "hash".to_string();
                            let block_height = client.get_chain_height().await.unwrap() as u64;
                            let event_data = EventData {
                                event_type: event.r#type.clone(),
                                event_data: event.data.clone(),
                                sequence_number,
                                transaction_hash,
                                block_height,
                            };
                            let _ = event_sender.send(event_data);
                            last_sequence = Some(sequence_number);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching events: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    /// event filter
    pub fn filter_events(
        events: Vec<EventData>,
        filters: HashMap<String, Value>,
    ) -> Vec<EventData> {
        events
            .into_iter()
            .filter(|event| {
                filters
                    .iter()
                    .all(|(key, value)| event.event_data.get(key).map_or(false, |v| v == value))
            })
            .collect()
    }

    /// event aggregator
    pub fn event_aggregator(
        events: Vec<EventData>,
        group_by: &str,
    ) -> HashMap<String, Vec<EventData>> {
        let mut grouped = HashMap::new();

        for event in events {
            if let Some(group_key) = event.event_data.get(group_by).and_then(|v| v.as_str()) {
                grouped
                    .entry(group_key.to_string())
                    .or_insert_with(Vec::new)
                    .push(event);
            }
        }
        grouped
    }
}

/// Event Subscription Manager
pub struct EventSubscriptionManager {
    subscriptions: HashMap<String, broadcast::Sender<EventData>>,
}

impl EventSubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribe to specific events
    pub fn subscribe(&mut self, event_key: String) -> broadcast::Receiver<EventData> {
        let (sender, receiver) = broadcast::channel(100);
        self.subscriptions.insert(event_key, sender);
        receiver
    }

    /// Publish events to subscribers
    pub fn publish_event(&self, event_key: &str, event: EventData) -> Result<(), String> {
        if let Some(sender) = self.subscriptions.get(event_key) {
            let _ = sender.send(event);
            Ok(())
        } else {
            Err(format!("No subscribers for event key: {}", event_key))
        }
    }

    /// Create EventData from the original Event and publish it
    pub fn publish_from_raw_event(
        &self,
        event_key: &str,
        event: Event,
        transaction_hash: String,
        block_height: u64,
    ) -> Result<(), String> {
        let sequence_number = match event.sequence_number.parse::<u64>() {
            Ok(seq) => seq,
            Err(_) => return Err("Invalid sequence number".to_string()),
        };
        let event_data = EventData {
            event_type: event.r#type,
            event_data: event.data,
            sequence_number,
            transaction_hash,
            block_height,
        };
        self.publish_event(event_key, event_data)
    }
}

/// event handling tools
pub struct EventUtils;

impl EventUtils {
    /// Create EventData from the Event structure
    pub fn create_event_data_from_event(
        event: Event,
        transaction_hash: String,
        block_height: u64,
    ) -> Result<EventData, String> {
        let sequence_number = event
            .sequence_number
            .parse::<u64>()
            .map_err(|_| "Invalid sequence number".to_string())?;
        Ok(EventData {
            event_type: event.r#type,
            event_data: event.data,
            sequence_number,
            transaction_hash,
            block_height,
        })
    }

    /// Extract specific fields from events
    pub fn extract_event_field(event: &EventData, field: &str) -> Option<Value> {
        event.event_data.get(field).cloned()
    }

    /// Check if the event type matches
    pub fn is_event_type(event: &EventData, expected_type: &str) -> bool {
        event.event_type == expected_type
    }

    /// Batch processing of events
    pub fn process_events_batch<F>(events: Vec<EventData>, processor: F)
    where
        F: Fn(EventData) -> Result<(), String>,
    {
        for event in events {
            if let Err(e) = processor(event) {
                eprintln!("Error processing event: {}", e);
            }
        }
    }
}
