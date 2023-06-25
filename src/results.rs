#[derive(Debug)]
pub enum Error{
    ChannelIsNSFW,
    PrivateChannelUserIsNotOwner,
    Generic,
    CouldntConvertToJSON,
    NothingUsefulToBeSaved,
}

#[derive(Debug)]
pub enum DatabaseResult{
    SavedMessagesFromChannel(SavedMessagesFromChannel),
    SavedMemories(SavedMemories)
}

#[derive(Debug)]
pub struct SavedMessagesFromChannel{
    pub channel_name:String,
    pub quantity:usize,
}

#[derive(Debug)]
pub struct SavedMemories{
    pub memory:String,
}