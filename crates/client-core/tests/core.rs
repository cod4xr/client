// SPDX-License-Identifier: AGPL-3.0-or-later
//! One integration executable; component files retain independent fixtures and cases.
//! Repository policy checks that every top-level test file is registered here.

mod client_commands;
mod client_data;
mod client_identities;
mod client_states;
mod config_state;
mod configstrings;
mod connect_limits;
mod connect_plan;
mod connect_replies;
mod connect_state;
mod connect_state_limits;
mod connect_state_support;
mod connect_support;
mod control_messages;
mod discovery;
mod entity_baselines;
mod entity_deltas;
mod gamestate;
mod handshake_limits;
mod handshake_state;
mod legacy_entity_baselines;
mod legacy_entity_deltas;
mod legacy_entity_support;
mod message_body;
mod mixed_services;
mod player_arrays;
mod player_hud;
mod player_objectives;
mod pre_game;
mod server_commands;
mod server_payloads;
mod service_controls;
mod service_support;
mod snapshot_clients;
mod snapshot_entities;
mod system_info;
