#![allow(non_snake_case)]

//! Air-integration type stubs. Twenty children: one per request/response
//! DTO, the placeholder `AirClientType::Struct`, and the
//! `DEFAULT_AIR_SERVER_ADDRESS::Const` server-address constant. Every
//! `AirClientType::Struct` method returns "feature not implemented" until
//! the real `AirLibrary` client lands behind `--features AirIntegration`.
//!
//! TODO: zero callers as of 2026-05-02; remove this entire module when
//! the live Air client is wired in.

pub mod AirClientType;

pub mod AirMetricsProtoDTO;

pub mod ApplyUpdateRequest;

pub mod ApplyUpdateResponse;

pub mod AuthenticationRequest;

pub mod AuthenticationResponse;

pub mod DEFAULT_AIR_SERVER_ADDRESS;

pub mod DownloadFileResponse;

pub mod DownloadRequest;

pub mod FileResultProtoDTO;

pub mod IndexFilesResponse;

pub mod IndexRequest;

pub mod MetricsRequest;

pub mod MetricsResponse;

pub mod SearchFilesResponse;

pub mod SearchRequest;

pub mod StatusRequest;

pub mod StatusResponse;

pub mod UpdateCheckRequest;

pub mod UpdateCheckResponse;
