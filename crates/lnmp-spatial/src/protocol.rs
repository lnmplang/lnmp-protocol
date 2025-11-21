use crate::delta::Delta;
use crate::error::SpatialError;
use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FrameMode {
    Absolute = 0x00,
    Delta = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpatialFrameHeader {
    pub mode: FrameMode,
    pub sequence_id: u32,
    pub timestamp: u64, // Nanoseconds
    pub checksum: u32,  // CRC32 of payload
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialFrame {
    pub header: SpatialFrameHeader,
    pub payload: SpatialValue,
}

#[derive(Debug, Clone)]
pub struct SpatialStreamerConfig {
    pub abs_interval: u32,
    pub enable_prediction: bool,
    pub max_prediction_frames: u8,
}

impl Default for SpatialStreamerConfig {
    fn default() -> Self {
        Self {
            abs_interval: 100,
            enable_prediction: true,
            max_prediction_frames: 3,
        }
    }
}

pub struct SpatialStreamer {
    config: SpatialStreamerConfig,
    sequence_counter: u32,
    last_sent_state: Option<SpatialState>,

    // Receiver state
    last_received_seq: Option<u32>,
    current_state: Option<SpatialState>,
    predicted_next: Option<Position3D>,
    prediction_frame_count: u8,
}

impl SpatialStreamer {
    pub fn new(abs_interval: u32) -> Self {
        Self::with_config(SpatialStreamerConfig {
            abs_interval,
            ..Default::default()
        })
    }

    pub fn with_config(config: SpatialStreamerConfig) -> Self {
        Self {
            config,
            sequence_counter: 0,
            last_sent_state: None,
            last_received_seq: None,
            current_state: None,
            predicted_next: None,
            prediction_frame_count: 0,
        }
    }

    /// Generates the next frame for a given state.
    /// Automatically decides whether to send ABS or DELTA.
    pub fn next_frame(
        &mut self,
        new_state: &SpatialState,
        timestamp: u64,
    ) -> Result<SpatialFrame, SpatialError> {
        let seq = self.sequence_counter;
        self.sequence_counter += 1;

        let force_abs = seq.is_multiple_of(self.config.abs_interval);

        let (mode, payload) = if force_abs || self.last_sent_state.is_none() {
            (FrameMode::Absolute, SpatialValue::S10(new_state.clone()))
        } else {
            let delta =
                SpatialState::compute_delta(self.last_sent_state.as_ref().unwrap(), new_state);
            (FrameMode::Delta, SpatialValue::S13(delta))
        };

        self.last_sent_state = Some(new_state.clone());

        // Predictive Delta: Compute predicted_next if enabled
        if self.config.enable_prediction {
            if let (Some(pos), Some(vel)) = (&new_state.position, &new_state.velocity) {
                // Predict next position based on current velocity
                // Assuming dt = 1ms (typical for high-frequency control)
                let dt = 0.001; // 1ms in seconds
                self.predicted_next = Some(Position3D {
                    x: pos.x + vel.vx * dt,
                    y: pos.y + vel.vy * dt,
                    z: pos.z + vel.vz * dt,
                });
            }
        }

        // Compute checksum of payload
        let payload_bytes = bincode::serialize(&payload)
            .map_err(|e| SpatialError::ValidationError(format!("Serialization error: {}", e)))?;
        let checksum = crate::checksum::compute_checksum(&payload_bytes);

        Ok(SpatialFrame {
            header: SpatialFrameHeader {
                mode,
                sequence_id: seq,
                timestamp,
                checksum,
            },
            payload,
        })
    }

    /// Processes an incoming frame and updates the internal state.
    /// Handles drift correction and sequence checking.
    pub fn process_frame(&mut self, frame: &SpatialFrame) -> Result<&SpatialState, SpatialError> {
        // 0. Checksum Verification
        let payload_bytes = bincode::serialize(&frame.payload)
            .map_err(|e| SpatialError::ValidationError(format!("Serialization error: {}", e)))?;

        if !crate::checksum::verify_checksum(&payload_bytes, frame.header.checksum) {
            return Err(SpatialError::ValidationError(
                "Checksum mismatch! Frame corrupted.".into(),
            ));
        }

        // 1. Sequence Check
        if let Some(last_seq) = self.last_received_seq {
            if frame.header.sequence_id <= last_seq {
                // Out of order or duplicate
                // For this simple implementation, we ignore or warn.
                // In a strict system, we might error.
            } else if frame.header.sequence_id > last_seq + 1 {
                // Gap detected! Packet loss.
                // If this is a DELTA frame, we CANNOT apply it safely because we missed the base.

                if frame.header.mode == FrameMode::Delta {
                    // Predictive Fallback: Use prediction if enabled
                    if self.config.enable_prediction && self.predicted_next.is_some() {
                        // Use predicted position to continue smoothly
                        self.prediction_frame_count += 1;

                        if self.prediction_frame_count > self.config.max_prediction_frames {
                            // Too many predictions, must reset with ABS
                            return Err(SpatialError::ValidationError(format!(
                                "Prediction limit exceeded ({} frames). Waiting for ABS frame.",
                                self.prediction_frame_count
                            )));
                        }

                        // Use prediction to update current state
                        if let Some(predicted_pos) = self.predicted_next {
                            if let Some(mut state) = self.current_state.clone() {
                                state.position = Some(predicted_pos);
                                self.current_state = Some(state);
                            }
                        }
                    } else {
                        // No prediction, must wait for ABS
                        return Err(SpatialError::ValidationError(format!(
                            "Packet loss detected (gap {} -> {}). Waiting for ABS frame.",
                            last_seq, frame.header.sequence_id
                        )));
                    }
                }
            }
        }

        // 2. Apply Update
        match frame.header.mode {
            FrameMode::Absolute => {
                if let SpatialValue::S10(state) = &frame.payload {
                    self.current_state = Some(state.clone());
                    // Reset prediction counter on ABS frame
                    self.prediction_frame_count = 0;
                } else {
                    return Err(SpatialError::ValidationError(
                        "ABS frame must contain SpatialState".into(),
                    ));
                }
            }
            FrameMode::Delta => {
                if let SpatialValue::S13(delta) = &frame.payload {
                    if let Some(current) = &self.current_state {
                        // Apply delta to current state
                        // Note: SpatialState::apply_delta expects &SpatialDelta (ref)
                        // but S13 contains SpatialDelta.
                        // We need to match the enum variant inside S13.
                        // Wait, S13 IS SpatialDelta enum variant in SpatialValue.
                        // But SpatialDelta is the type.
                        // Let's verify types.rs.
                        // SpatialValue::S13(SpatialDelta)

                        let new_state = SpatialState::apply_delta(current, delta);
                        self.current_state = Some(new_state);
                    } else {
                        return Err(SpatialError::ValidationError(
                            "Received DELTA frame without prior ABS state".into(),
                        ));
                    }
                } else {
                    return Err(SpatialError::ValidationError(
                        "DELTA frame must contain SpatialDelta".into(),
                    ));
                }
            }
        }

        self.last_received_seq = Some(frame.header.sequence_id);

        Ok(self.current_state.as_ref().unwrap())
    }
}
