use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;

pub fn vc_is_empty(guild: &Guild, channel_id: ChannelId) -> bool {
    !guild
        .voice_states
        .values()
        .any(|x| matches!(x.channel_id, Some(c) if c == channel_id))
}
