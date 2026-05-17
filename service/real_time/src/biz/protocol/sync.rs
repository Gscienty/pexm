use std::io;

use y_octo::{
    read_var_buffer, write_var_buffer, write_var_u64, CrdtRead, CrdtWrite, Doc, JwstCodecError,
    JwstCodecResult, RawDecoder, RawEncoder, StateVector,
};

use super::message_type::{DocMessage, MessageType};

pub fn read_sync_step1(msg: &[u8]) -> JwstCodecResult<StateVector> {
    let (_, state_vector_bytes) = read_var_buffer(msg).map_err(|err| err.map_input(|u| u.len()))?;

    StateVector::read(&mut RawDecoder::new(state_vector_bytes))
}

pub fn read_sync_step2(msg: &[u8]) -> JwstCodecResult<Vec<u8>> {
    let (_, update_bytes) = read_var_buffer(msg).map_err(|err| err.map_input(|u| u.len()))?;

    Ok(update_bytes.to_owned())
}

#[inline]
pub fn read_sync_update(msg: &[u8]) -> JwstCodecResult<Vec<u8>> {
    read_sync_step2(msg)
}

pub fn write_sync_step1(doc: &Doc) -> JwstCodecResult<Vec<u8>> {
    let mut encoder = RawEncoder::default();
    doc.get_state_vector().write(&mut encoder)?;
    let state_vector_bytes = encoder.into_inner();

    write_sync_step1_inline(&state_vector_bytes)
        .map_err(|err| JwstCodecError::InvalidWriteBuffer(err.to_string()))
}

pub fn write_sync_step2(doc: &Doc, state_vector: &StateVector) -> JwstCodecResult<Vec<u8>> {
    let update_bytes = doc.encode_state_as_update_v1(state_vector)?;

    write_sync_step2_inline(&update_bytes)
        .map_err(|err| JwstCodecError::InvalidWriteBuffer(err.to_string()))
}

pub fn write_sync_status(update_saved: bool) -> JwstCodecResult<Vec<u8>> {
    write_sync_status_inline(update_saved)
        .map_err(|err| JwstCodecError::InvalidWriteBuffer(err.to_string()))
}

pub fn write_sync_update(update: &[u8]) -> JwstCodecResult<Vec<u8>> {
    write_sync_step2_inline(update)
        .map_err(|err| JwstCodecError::InvalidWriteBuffer(err.to_string()))
}

#[inline]
fn write_sync_step1_inline(state_vector: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut sync_step1 = Vec::with_capacity(11 + state_vector.len());
    write_var_u64(&mut sync_step1, MessageType::Sync.into())?;

    write_var_u64(&mut sync_step1, DocMessage::Step1.into())?;
    write_var_buffer(&mut sync_step1, state_vector)?;

    Ok(sync_step1)
}

#[inline]
fn write_sync_step2_inline(update: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut sync_step2 = Vec::with_capacity(11 + update.len());
    write_var_u64(&mut sync_step2, MessageType::Sync.into())?;

    write_var_u64(&mut sync_step2, DocMessage::Step2.into())?;
    write_var_buffer(&mut sync_step2, update)?;

    Ok(sync_step2)
}

#[allow(unused)]
#[inline]
fn write_sync_update_inline(update: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut sync_update = Vec::with_capacity(11 + update.len());
    write_var_u64(&mut sync_update, MessageType::Sync.into())?;

    write_var_u64(&mut sync_update, DocMessage::Update.into())?;
    write_var_buffer(&mut sync_update, update)?;

    Ok(sync_update)
}

#[inline]
fn write_sync_status_inline(update_saved: bool) -> Result<Vec<u8>, io::Error> {
    let mut sync_status = Vec::with_capacity(2);

    write_var_u64(&mut sync_status, MessageType::SyncStatus.into())?;
    write_var_u64(&mut sync_status, if update_saved { 1 } else { 0 })?;

    Ok(sync_status)
}
