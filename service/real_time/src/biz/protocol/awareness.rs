use y_octo::{
    read_var_buffer, AwarenessState, AwarenessStates, CrdtReader, JwstCodecResult, RawDecoder,
};

pub fn read_awareness_update(message: &[u8]) -> JwstCodecResult<AwarenessStates> {
    let (_, update) = read_var_buffer(message).map_err(|err| err.map_input(|u| u.len()))?;

    let mut decoder = RawDecoder::new(update);

    let len = decoder.read_var_u64()? as usize;

    let mut states = Vec::new();
    for _ in 0..len {
        let client_id = decoder.read_var_u64()?;
        let clock = decoder.read_var_u64()?;
        let content = decoder.read_var_string()?;

        states.push((client_id, AwarenessState::new(clock, content)));
    }

    Ok(states.into_iter().collect())
}
