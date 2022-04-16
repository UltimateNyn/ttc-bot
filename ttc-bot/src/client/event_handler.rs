use crate::groups;
use poise::serenity_prelude::*;

// -------------------------------------
// Event Handler and it's implementation
// -------------------------------------

// Custom handler for events
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::listening("Kirottu's screaming"))
            .await;
        log::info!("Bot ready logged in as {}", ready.user.tag());
    }

    async fn message(&self, ctx: Context, msg: Message) {
        //events::conveyance::message(&ctx, &msg).await;

        if msg.content.contains("bots will take over the world") {
            match msg.channel_id.say(ctx, "*hides*").await {
                Ok(_) => (),
                Err(why) => log::error!("Error sending message: {}", why),
            }
        }
    }

    // Update thread status on the database when it is updated
    async fn thread_update(&self, ctx: Context, thread: GuildChannel) {
        //groups::support::thread_update(&ctx, &thread).await;
    }

    // For conveyance
    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        _: Option<GuildId>,
    ) {
        //events::conveyance::message_delete(&ctx, &channel_id, &deleted_message_id).await;
    }

    // For conveyance
    async fn message_update(
        &self,
        ctx: Context,
        old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        //events::conveyance::message_update(&ctx, old_if_available, new, &event).await;
    }

    // Greeting messages and user join events
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        //events::conveyance::guild_member_addition(&ctx, &new_member).await;
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _: GuildId,
        user: User,
        member: Option<Member>,
    ) {
        //events::conveyance::guild_member_removal(&ctx, &user, member).await;
    }

    async fn interaction_create(&self, ctx: Context, intr: Interaction) {
        //events::interactions::interaction_create(&ctx, intr).await;
    }

    async fn guild_ban_addition(&self, ctx: Context, _: GuildId, banned_user: User) {
        //events::conveyance::guild_ban_addition(&ctx, banned_user).await;
    }

    async fn guild_ban_removal(&self, ctx: Context, _: GuildId, unbanned_user: User) {
        //events::conveyance::guild_ban_removal(&ctx, unbanned_user).await;
    }

    async fn guild_member_update(&self, ctx: Context, old: Option<Member>, new: Member) {
        //events::conveyance::guild_member_update(&ctx, old, new).await;
    }
}
