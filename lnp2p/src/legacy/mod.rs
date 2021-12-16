// LNP P2P library, plmeneting both legacy (BOLT) and Bifrost P2P messaging
// system for Lightning network protocol (LNP)
//
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

mod bolt1;
mod bolt2;
mod bolt4;
mod bolt7;
mod bolt9;
mod types;

use std::io;

pub use bolt1::*;
pub use bolt2::*;
pub use bolt4::*;
pub use bolt7::*;
pub use bolt9::{Feature, FeatureContext, InitFeatures, UnknownFeatureError};
use internet2::{CreateUnmarshaller, Payload, Unmarshall, Unmarshaller};
use lightning_encoding::{self, LightningDecode, LightningEncode};
pub use types::*;

/// Default legacy Lightning port number
pub const LNP2P_LEGACY_PORT: u16 = 9735;

lazy_static! {
    pub static ref LNP2P_LEGACY_UNMARSHALLER: Unmarshaller<Messages> =
        Messages::create_unmarshaller();
}

#[derive(Clone, Debug, Display, Api)]
#[cfg_attr(feature = "strict_encoding", derive(NetworkEncode, NetworkDecode))]
#[api(encoding = "lightning")]
#[non_exhaustive]
#[display(inner)]
pub enum Messages {
    // Part I: Generic messages outside of channel operations (BOLT-1)
    // ===============================================================
    /// Once authentication is complete, the first message reveals the features
    /// supported or required by this node, even if this is a reconnection.
    #[api(type = 16)]
    Init(Init),

    /// For simplicity of diagnosis, it's often useful to tell a peer that
    /// something is incorrect.
    #[api(type = 17)]
    Error(Error),

    /// In order to allow for the existence of long-lived TCP connections, at
    /// times it may be required that both ends keep alive the TCP connection
    /// at the application level. Such messages also allow obfuscation of
    /// traffic patterns.
    #[api(type = 18)]
    Ping(Ping),

    /// The pong message is to be sent whenever a ping message is received. It
    /// serves as a reply and also serves to keep the connection alive, while
    /// explicitly notifying the other end that the receiver is still active.
    /// Within the received ping message, the sender will specify the number of
    /// bytes to be included within the data payload of the pong message.
    #[api(type = 19)]
    #[display("pong(...)")]
    Pong(Vec<u8>),

    // Part II: Channel management protocol (BOLT-2)
    // =============================================
    //
    // 1. Channel establishment / closing
    // ----------------------------------
    /// This message contains information about a node and indicates its desire
    /// to set up a new channel. This is the first step toward creating the
    /// funding transaction and both versions of the commitment transaction.
    #[api(type = 32)]
    OpenChannel(OpenChannel),

    /// This message contains information about a node and indicates its
    /// acceptance of the new channel. This is the second step toward creating
    /// the funding transaction and both versions of the commitment
    /// transaction.
    #[api(type = 33)]
    AcceptChannel(AcceptChannel),

    /// This message describes the outpoint which the funder has created for
    /// the initial commitment transactions. After receiving the peer's
    /// signature, via `funding_signed`, it will broadcast the funding
    /// transaction.
    #[api(type = 34)]
    FundingCreated(FundingCreated),

    /// This message gives the funder the signature it needs for the first
    /// commitment transaction, so it can broadcast the transaction knowing
    /// that funds can be redeemed, if need be.
    ///
    /// This message introduces the `channel_id` to identify the channel.
    #[api(type = 35)]
    FundingSigned(FundingSigned),

    /// This message indicates that the funding transaction has reached the
    /// `minimum_depth` asked for in `accept_channel`. Once both nodes have
    /// sent this, the channel enters normal operating mode.
    #[api(type = 36)]
    FundingLocked(FundingLocked),

    #[api(type = 38)]
    Shutdown(Shutdown),

    #[api(type = 39)]
    ClosingSigned(ClosingSigned),

    // 2. Channel operations
    // ---------------------
    #[api(type = 128)]
    UpdateAddHtlc(UpdateAddHtlc),

    #[api(type = 130)]
    UpdateFulfillHtlc(UpdateFulfillHtlc),

    #[api(type = 131)]
    UpdateFailHtlc(UpdateFailHtlc),

    #[api(type = 135)]
    UpdateFailMalformedHtlc(UpdateFailMalformedHtlc),

    #[api(type = 132)]
    CommitmentSigned(CommitmentSigned),

    #[api(type = 133)]
    RevokeAndAck(RevokeAndAck),

    #[api(type = 134)]
    UpdateFee(UpdateFee),

    #[api(type = 136)]
    ChannelReestablish(ChannelReestablish),

    // Part III. Gossip protocol (BOLT-7)
    // ==================================
    #[api(type = 259)]
    AnnouncementSignatures(AnnouncementSignatures),

    #[api(type = 256)]
    ChannelAnnouncements(ChannelAnnouncements),

    #[api(type = 257)]
    NodeAnnouncements(NodeAnnouncements),

    #[api(type = 258)]
    ChannelUpdate(ChannelUpdate),

    /// Extended Gossip queries
    /// Negotiating the gossip_queries option via init enables a number of
    /// extended queries for gossip synchronization.
    #[api(type = 261)]
    QueryShortChannelIds(QueryShortChannelIds),

    #[api(type = 262)]
    ReplyShortChannelIdsEnd(ReplyShortChannelIdsEnd),

    #[api(type = 263)]
    QueryChannelRange(QueryChannelRange),

    #[api(type = 264)]
    ReplyChannelRange(ReplyChannelRange),

    #[api(type = 265)]
    GossipTimestampFilter(GossipTimestampFilter),
}

impl LightningEncode for Messages {
    fn lightning_encode<E: io::Write>(
        &self,
        e: E,
    ) -> Result<usize, lightning_encoding::Error> {
        Payload::from(self.clone()).lightning_encode(e)
    }
}

impl LightningDecode for Messages {
    fn lightning_decode<D: io::Read>(
        d: D,
    ) -> Result<Self, lightning_encoding::Error> {
        let message = &*LNP2P_LEGACY_UNMARSHALLER
            .unmarshall(&Vec::<u8>::lightning_decode(d)?)
            .map_err(|err| {
                lightning_encoding::Error::DataIntegrityError(format!(
                    "can't unmarshall legacy LNP2P message. Details: {}",
                    err
                ))
            })?;
        Ok(message.clone())
    }
}
