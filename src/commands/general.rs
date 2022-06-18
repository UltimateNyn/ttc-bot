use crate::{
    get_config,
    types::{Context, Data, Error},
    utils::helper_functions::format_datetime,
};
use chrono::{Duration, Utc};
use futures::{lock::Mutex, StreamExt};
use poise::{
    serenity_prelude::{Color, CreateEmbed, Member, RoleId, Timestamp, User, UserId},
    Command,
};
use std::{collections::HashMap, iter::Iterator, sync::Arc};
use tokio::time::Instant;
// ----------------------
// General group commands
// ----------------------

/// Ping command
///
/// Command that the bot will respond to with "pong"
/// ``ping``
#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("pong").await?;

    Ok(())
}
/// User info for a user
///
/// Can be used to get the user info of the specified user. A context menu command is also available
/// ``userinfo [user]``
#[poise::command(
    prefix_command,
    slash_command,
    context_menu_command = "User info",
    category = "General"
)]
pub async fn userinfo(ctx: Context<'_>, #[description = "User"] user: User) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let (nickname, joined_at, roles) = match ctx.guild() {
        Some(guild) => {
            match guild.member(ctx.discord(), user.id).await {
                Ok(member) => {
                    let nick = member.nick.clone().unwrap_or("None".to_string());
                    let joined_at = match member.joined_at {
                        Some(joined_at) => format_datetime(&joined_at),
                        None => "N/A".to_string(),
                    };
                    let mut roles = match member.roles(ctx.discord()) {
                        Some(roles) => roles
                            .iter()
                            .map(|role| format!("<@&{}>, ", role.id))
                            .collect::<String>(),
                        None => "None".to_string(),
                    };
                    // Remove trailing comma and space
                    roles.pop();
                    roles.pop();

                    // Make sure it isn't empty
                    if roles == "" {
                        roles = "None".to_string()
                    }
                    (nick, joined_at, roles)
                }
                Err(_) => ("N/A".to_string(), "N/A".to_string(), "N/A".to_string()),
            }
        }
        None => ("N/A".to_string(), "N/A".to_string(), "N/A".to_string()),
    };

    ctx.send(|m| {
        m.embed(|e| {
            e.author(|a| a.name(user.tag()).icon_url(user.face()))
                .field("User ID", user.id.0, true)
                .field("Nickname", nickname, true)
                .field("Created At", format_datetime(&user.id.created_at()), false)
                .field("Joined At", joined_at, false)
                .field("Roles", roles, false)
                .field("Icon URL", user.face(), false)
                .color(Color::BLITZ_BLUE)
        })
        .ephemeral(true)
    })
    .await?;

    Ok(())
}

/// Server info
///
/// Can be used to get server info
/// ``serverinfo``
#[poise::command(prefix_command, slash_command, guild_only, category = "General")]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let guild = ctx.guild().unwrap();
    let guild_id_part = guild.id.to_partial_guild_with_counts(ctx.discord()).await?;
    let online_members = match guild_id_part.approximate_presence_count {
        Some(s) => s.to_string(),
        None => String::from("N/A"),
    };
    let icon = guild.icon_url().unwrap_or(String::from("N/A"));
    let emojis = guild.emojis(ctx.discord()).await?;
    let mut fields: Vec<(String, String, bool)> = Vec::new();

    fields.push(("Guild name".to_string(), guild.name.clone(), false));
    fields.push((
        "Server owner".to_string(),
        format!("<@{}>", guild.owner_id.0),
        false,
    ));
    fields.push((
        "Online Members".to_string(),
        format!("{}/{}", online_members, guild.member_count),
        false,
    ));

    let mut tmp = String::new();
    let mut count = 1;
    emojis
        .iter()
        .filter(|emoji| !(emoji.animated))
        .for_each(|emoji| {
            let t = format!("{}<:{}:{}> ", tmp, emoji.name, emoji.id.0);
            if t.len() > 1024 {
                fields.push((
                    format!("Custom Emojis {}", count).to_string(),
                    tmp.clone(),
                    false,
                ));
                tmp = format!("<:{}:{}> ", emoji.name, emoji.id.0);
                count += 1;
            } else {
                tmp = t;
            }
        });
    if count > 1 {
        fields.push((
            format!("Custom Emojis {}", count).to_string(),
            tmp.clone(),
            false,
        ));
    } else {
        fields.push(("Custom Emojis".to_string(), tmp.clone(), false));
    }

    tmp = String::new();
    count = 1;
    emojis
        .iter()
        .filter(|emoji| emoji.animated)
        .for_each(|emoji| {
            let t = format!("{}<a:{}:{}> ", tmp, emoji.name, emoji.id.0);
            if t.len() > 1024 {
                fields.push((
                    format!("Animated Emojis {}", count).to_string(),
                    tmp.clone(),
                    false,
                ));
                tmp = format!("<a:{}:{}> ", emoji.name, emoji.id.0);
                count += 1;
            } else {
                tmp = t;
            }
        });
    if count > 1 {
        fields.push((
            format!("Animated Emojis {}", count).to_string(),
            tmp.clone(),
            false,
        ));
    } else {
        fields.push(("Animated Emojis".to_string(), tmp.clone(), false));
    }

    tmp = String::new();
    count = 1;
    guild
        .roles
        .iter()
        .filter(|role| role.1.name != "@everyone")
        .for_each(|role| {
            let t = format!("{}<@&{}> ", tmp, role.0 .0);
            if t.len() > 1024 {
                fields.push((format!("Roles {}", count).to_string(), tmp.clone(), false));
                tmp = format!("{}<@&{}> ", tmp, role.0 .0);
                count += 1;
            } else {
                tmp = t;
            }
        });
    if count > 1 {
        fields.push((format!("Roles {}", count).to_string(), tmp.clone(), false));
    } else {
        fields.push(("Roles".to_string(), tmp.clone(), false));
    }

    fields.push(("Icon URL".to_string(), icon.clone(), false));

    ctx.send(|m| {
        m.embed(|e| {
            e.author(|a| a.name(&guild.name))
                .fields(fields)
                .color(Color::BLITZ_BLUE)
                .thumbnail(&icon)
        })
        .ephemeral(true)
    })
    .await?;

    Ok(())
}

/// Harold.
///
/// Count the harolds of the server and the specified user, if provided. The leaderboard flag will toggle these 3 leaderboards:
/// 1. Harold message count
/// 2. Harold message percentage (of all messages by user)
/// 3. Messages sent in total
/// **NOTE**: This command will take a long time to run, so grab some popcorn while you let it run.
/// ``harold [member (optional)] [leaderboard (True or False)]``
#[poise::command(slash_command, prefix_command, guild_only, category = "General")]
pub async fn harold(
    ctx: Context<'_>,
    #[description = "User to calculate harold percentage of"] user: Option<User>,
    #[description = "Whether to show the leaderboard or not"]
    #[flag]
    leaderboard: bool,
) -> Result<(), Error> {
    {
        let harold_message = ctx.data().harold_message.read().await;
        match &*harold_message {
            Some(message) => {
                ctx.send(|m| {
                    m.embed(|e| {
                        e.title("Harolds are already being calculated")
                            .description(format!(
                                "You can view current progress at this message: {}",
                                message.link()
                            ))
                            .color(Color::RED)
                    })
                    .ephemeral(true)
                })
                .await?;
                return Ok(());
            }
            None => (),
        }
    }

    ctx.send(|m| {
        m.embed(|e| {
            e.title("Started harold counting process.")
                .description("This could take a while. Grab some popcorn and wait.")
                .color(Color::BLITZ_BLUE)
        })
    })
    .await?;

    let progress_message = Arc::new(Mutex::new(
        ctx.channel_id()
            .send_message(ctx.discord(), |m| {
                m.embed(|e| {
                    e.title("Harold counting in progress")
                        .description("Channels counted: 0/0")
                })
            })
            .await?,
    ));

    // Set the lock to avoid running multiple concurrent instances of this command
    {
        let mut harold_message = ctx.data().harold_message.write().await;
        *harold_message = Some(progress_message.lock().await.clone());
    }

    let channel_progress = Arc::new(Mutex::new(0));

    let mut handles = Vec::new();

    let channels = ctx.guild().unwrap().channels;
    let channel_amount = channels.len();

    let start_time = Instant::now();

    for (channel_id, _) in channels {
        let ctx = ctx.discord().clone();
        let progress_message = progress_message.clone();
        let channel_amount = channel_amount.clone();
        let channel_progress = channel_progress.clone();
        let handle = tokio::spawn(async move {
            let mut global_messages: (u64, u64) = (0, 0);
            let mut user_hash_map: HashMap<UserId, (u64, u64)> = HashMap::new();
            let mut messages = channel_id.messages_iter(ctx.clone()).boxed();
            while let Some(message) = messages.next().await {
                match message {
                    Ok(message) => {
                        let user_messages = if user_hash_map.contains_key(&message.author.id) {
                            match user_hash_map.get_mut(&message.author.id) {
                                Some(user_messages) => user_messages,
                                None => unreachable!(),
                            }
                        } else {
                            user_hash_map.insert(message.author.id, (0, 0));
                            match user_hash_map.get_mut(&message.author.id) {
                                Some(user_messages) => user_messages,
                                None => unreachable!(),
                            }
                        };
                        global_messages.0 += 1;
                        user_messages.0 += 1;
                        if message.content.contains(":helpmeplz:") {
                            global_messages.1 += 1;
                            user_messages.1 += 1;
                        }
                    }
                    Err(why) => log::error!("Something went wrong when getting message: {}", why),
                }
            }
            let mut channel_progress = channel_progress.lock().await;
            *channel_progress += 1;
            match progress_message
                .lock()
                .await
                .edit(ctx, |m| {
                    m.embed(|e| {
                        e.title("Harold counting in progress")
                            .description(format!(
                                "Channels counted: {}/{}",
                                *channel_progress, channel_amount
                            ))
                            .color(Color::BLITZ_BLUE)
                    })
                })
                .await
            {
                Ok(_) => (),
                Err(why) => log::error!("Failed to edit message: {}", why),
            }

            (channel_id, user_hash_map, global_messages)
        });
        handles.push(handle);
    }
    let mut global_messages: (u64, u64) = (0, 0);
    let mut global_user_hash_map: HashMap<UserId, (u64, u64)> = HashMap::new();

    for handle in handles {
        let value = handle.await?;
        global_messages.0 += value.2 .0;
        global_messages.1 += value.2 .1;
        for (user_id, user_messages) in value.1 {
            if global_user_hash_map.contains_key(&user_id) {
                match global_user_hash_map.get_mut(&user_id) {
                    Some(global_user_messages) => {
                        global_user_messages.0 += user_messages.0;
                        global_user_messages.1 += user_messages.1;
                    }
                    None => unreachable!(),
                }
            } else {
                global_user_hash_map.insert(user_id, (0, 0));
                match global_user_hash_map.get_mut(&user_id) {
                    Some(global_user_messages) => {
                        global_user_messages.0 += user_messages.0;
                        global_user_messages.1 += user_messages.1;
                    }
                    None => unreachable!(),
                }
            }
        }
    }
    // Dump the whole hashmap into a Vec
    let user_message_vec = global_user_hash_map
        .iter()
        .map(|(user_id, user_messages)| {
            (
                user_id.clone(),
                user_messages.clone(),
                (user_messages.1 as f32 / user_messages.0 as f32) * 100.0,
            )
        })
        .collect::<Vec<(UserId, (u64, u64), f32)>>();

    // Create the different leaderboards
    let mut harold_message_leaderboard = user_message_vec.clone();
    let mut message_leaderboard = user_message_vec.clone();
    let mut harold_percentage_leaderboard = user_message_vec.clone();

    // Sort them
    harold_message_leaderboard.sort_by(|a, b| {
        b.1 .1
            .partial_cmp(&a.1 .1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    message_leaderboard.sort_by(|a, b| {
        b.1 .0
            .partial_cmp(&a.1 .0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    harold_percentage_leaderboard
        .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut embeds = Vec::new();

    embeds.push({
        let mut embed = CreateEmbed::default();
        embed.title("Harold counting finished")
            .description(format!(
                "Total messages: {}\nTotal harold messages: {}\nGlobal harold percentage: {:.2}%\nTime taken: {} minutes and {} seconds",
                global_messages.0,
                global_messages.1,
                (global_messages.1 as f64 / global_messages.0 as f64) * 100.0,
                start_time.elapsed().as_secs() / 60,
                start_time.elapsed().as_secs() % 60
            ))
            .color(Color::BLURPLE);
            embed
        }
    );

    match user {
        Some(user) => {
            let mut messages: (u64, u64) = (0, 0);
            let mut harold_percentage: f32 = 0.0;
            let mut leaderboard_positions: (u32, u32, u32) = (0, 0, 0);
            for i in 0..harold_message_leaderboard.len() {
                if harold_message_leaderboard[i].0 == user.id {
                    leaderboard_positions.0 = i as u32 + 1;
                    messages.1 = harold_message_leaderboard[i].1 .1;
                }
            }
            for i in 0..message_leaderboard.len() {
                if message_leaderboard[i].0 == user.id {
                    leaderboard_positions.1 = i as u32 + 1;
                    messages.0 = message_leaderboard[i].1 .0;
                }
            }
            for i in 0..harold_percentage_leaderboard.len() {
                if harold_percentage_leaderboard[i].0 == user.id {
                    leaderboard_positions.2 = i as u32 + 1;
                    harold_percentage = harold_percentage_leaderboard[i].2;
                }
            }
            embeds.push({
                let mut embed = CreateEmbed::default();
                embed
                    .title("User harold statistics")
                    .description("Harold statistics for the specified user")
                    .field("User", format!("<@{}>", user.id.0), false)
                    .field(
                        "Harold messages",
                        format!(
                            "Amount: {}\nLeaderboard position: {}",
                            messages.1, leaderboard_positions.0
                        ),
                        false,
                    )
                    .field(
                        "Messages",
                        format!(
                            "Amount: {}\nLeaderboard position: {}",
                            messages.0, leaderboard_positions.1
                        ),
                        false,
                    )
                    .field(
                        "Harold percentage",
                        format!(
                            "Percentage: {:.2}%\nLeaderboard position: {}",
                            harold_percentage, leaderboard_positions.2
                        ),
                        false,
                    )
                    .color(Color::MAGENTA);
                embed
            });
        }
        None => (),
    }

    if leaderboard {
        embeds.push({
            let mut embed = CreateEmbed::default();
            embed
                .title("Harold message leaderboard")
                .description("Leaderboard of users based on harold message count.")
                .fields((0..10).map(|i| {
                    let (user_id, user_messages, _) = &harold_message_leaderboard[i as usize];
                    (
                        i + 1,
                        format!("<@{}>, {} harold messages.", user_id, user_messages.1),
                        false,
                    )
                }))
                .color(Color::FOOYOO);
            embed
        });
        embeds.push({
            let mut embed = CreateEmbed::default();
            embed
                .title("Harold percentage leaderboard")
                .description("Leaderboard of users based on harold percentage.")
                .fields((0..10).map(|i| {
                    let (user_id, _, percentage) = &harold_percentage_leaderboard[i as usize];
                    (
                        i + 1,
                        format!(
                            "<@{}>, {:.2}% of messages contain harold.",
                            user_id, percentage
                        ),
                        false,
                    )
                }))
                .color(Color::BLUE);
            embed
        });
        embeds.push({
            let mut embed = CreateEmbed::default();
            embed
                .title("Message leaderboard")
                .description("Leaderboard of users based on message count.")
                .fields((0..10).map(|i| {
                    let (user_id, user_messages, _) = &message_leaderboard[i as usize];
                    (
                        i + 1,
                        format!("<@{}>, {} messages.", user_id, user_messages.0),
                        false,
                    )
                }))
                .color(Color::PURPLE);
            embed
        });
    }

    ctx.channel_id()
        .send_message(ctx.discord(), |m| m.set_embeds(embeds))
        .await?;

    // Reset it after it is done
    {
        let mut harold_message = ctx.data().harold_message.write().await;
        *harold_message = None;
    }

    Ok(())
}

/// Help for all or individual commands
///
/// Command to get help for a specific or all commands
/// ``help [command (optional)]``
#[poise::command(prefix_command, slash_command, category = "General")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "A single command to view help of"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    match command {
        Some(command) => {
            // Remove whitespaces that could come from automatic whitespaces on e.g. mobile devices
            let command = command.trim();
            for help_option in ctx.framework().options().commands.iter() {
                if &help_option.name == &command {
                    let (desc, color) = match help_option.multiline_help {
                        Some(s) => (s(), Color::FOOYOO),
                        None => (format!("No help available for {}", &command), Color::RED),
                    };
                    ctx.send(|m| {
                        m.embed(|e| e.title(&command).description(desc).color(color))
                            .ephemeral(true)
                    })
                    .await?;
                    return Ok(());
                }
            }
            // The user called help for something that doesn't exist
            ctx.send(|m| {
                m.embed(|e| {
                    e.title("No help available")
                        .description(format!("Couldn't find \"{}\" in commands. You probably specified a command that isn't existing", command))
                        .color(Color::RED)
                })
                .ephemeral(true)
            })
            .await?;
        }
        None => {
            let commands = &ctx.framework().options().commands;

            let mut categories: HashMap<&str, Vec<&Command<Data, Error>>> = HashMap::new();

            for command in commands {
                let category = command.category.unwrap_or("General");
                let commands = categories.entry(category).or_insert(Vec::new());
                commands.push(command);
            }
            let mut sorted_categories = categories
                .iter()
                .map(|(k, v)| (*k, v))
                .collect::<Vec<(&str, &Vec<&Command<Data, Error>>)>>();
            sorted_categories.sort_by(|a, b| a.0.cmp(b.0));

            ctx.send(|m| {
                m.embed(|e| {
                    e.title("Help")
                        .fields(sorted_categories.iter().map(|(category, commands)| {
                            let mut commands = (*commands).clone();
                            commands.sort_by(|a, b| a.name.cmp(b.name));
                            let mut command_string = String::new();
                            for command in commands {
                                if command.hide_in_help {
                                    continue;
                                }
                                command_string.push_str(&format!(
                                    "``{}``: {}\n",
                                    command.name,
                                    command.inline_help.unwrap_or("No help available")
                                ));
                            }
                            if command_string.len() == 0 {
                                command_string = "No commands available".to_string();
                            }
                            (category, command_string, false)
                        }))
                        .color(Color::FOOYOO)
                })
                .ephemeral(true)
            })
            .await?;
        }
    }
    Ok(())
}

/// Votemute an User
///
/// When enough regulars vote to mute a user the user gets muted
/// ``votemute [user]``
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    user_cooldown = 30,
    category = "General"
)]
pub async fn votemute(
    ctx: Context<'_>,
    #[rename = "user"]
    #[description = "User to votemute"]
    mut target_user_m: Member,
) -> Result<(), Error> {
    const TIMEOUT_DURATION: usize = 30;
    // pub votemute_users: Mutex<HashMap<UserId, (i64, Vec<UserId>)>>,
    // HashMap<Target, (Timestamp; List of Users that voted)>

    let target_user = target_user_m.user.id;
    let calling_user_m = ctx.author_member().await.unwrap();
    let calling_user = calling_user_m.user.id;
    let config = get_config!(ctx.data(), {
        return Err(Error::from("Unable to obtain config"));
    });
    let required_users = config.votemute_required_users as usize;

    // Regular check
    if !calling_user_m
        .user
        .has_role(
            ctx.discord(),
            ctx.guild_id().unwrap(),
            RoleId(config.regular_role as u64),
        )
        .await?
    {
        ctx.send(|b| {
            b.embed(|e| {
                e.title("You are not a Regular member")
                    .description("You need the Regular role to votemute")
                    .color(Color::RED)
            })
            .ephemeral(true)
        })
        .await?;
        return Ok(());
    }

    // Is the caller the target
    if calling_user == target_user {
        ctx.send(|m| {
            m.embed(|e| {
                e.title("That's a bad idea")
                    .description("You should not try to votemute yourself")
                    .color(Color::RED)
            })
            .ephemeral(true)
        })
        .await?;
        return Ok(());
    }

    // Is the target a moderator
    if target_user_m
        .user
        .has_role(
            ctx.discord(),
            ctx.guild_id().unwrap(),
            RoleId(config.moderator_role as u64),
        )
        .await?
    {
        ctx.send(|b| {
            b.embed(|e| {
                e.title("That's a bad idea")
                    .description("You should not try to votemute a moderator")
                    .color(Color::RED)
            })
            .ephemeral(true)
        })
        .await?;
        return Ok(());
    }

    // Is the target a bot
    if target_user_m.user.bot {
        ctx.send(|b| {
            b.embed(|e| {
                e.title("That's a bad idea")
                    .description("You should not try to votemute a bot")
                    .color(Color::RED)
            })
            .ephemeral(true)
        })
        .await?;
        return Ok(());
    }

    // Is the target already muted
    if target_user_m.communication_disabled_until.is_some() {
        if target_user_m
            .communication_disabled_until
            .unwrap()
            .unix_timestamp()
            > Timestamp::now().unix_timestamp()
        {
            ctx.send(|b| {
                b.embed(|e| {
                    e.title(format!(
                        "User {} is already muted",
                        target_user_m.user.tag()
                    ))
                    .description("There is no need for a votemute")
                    .color(Color::RED)
                })
                .ephemeral(true)
            })
            .await?;
            return Ok(());
        }
    }

    // Actual logic
    let mut users = ctx.data().votemute_users.lock().await;
    if users.contains_key(&target_user) {
        // Reset count, if the first votemute is over 5 minutes ago
        if users.get(&target_user).unwrap().0 < Utc::now().timestamp() {
            users.get_mut(&target_user).unwrap().1.clear();
            users.get_mut(&target_user).unwrap().0 =
                Utc::now().timestamp() + Duration::minutes(5).num_seconds();
        }
        // Add calling user to voted users of the target
        let vec = &mut users.get_mut(&target_user).unwrap().1;
        if !vec.contains(&calling_user) {
            vec.push(calling_user);
        } else {
            ctx.send(|b| {
                b.embed(|e| {
                    e.title(format!(
                        "You already voted to mute {}",
                        target_user_m.user.tag()
                    ))
                    .description("You can't for the same user twice")
                    .color(Color::RED)
                })
                .ephemeral(true)
            })
            .await?;
            return Ok(());
        }

        // Are there enough votes?
        if vec.len() == required_users {
            users.remove(&target_user);
            target_user_m
                .disable_communication_until_datetime(
                    ctx.discord(),
                    Timestamp::from(Utc::now() + Duration::minutes(TIMEOUT_DURATION.try_into()?)),
                )
                .await?;
            ctx.send(|b| {
                b.embed(|e| {
                    e.title(format!(
                        "User {} muted for {} minutes",
                        target_user_m.user.tag(),
                        TIMEOUT_DURATION
                    ))
                    .description("This should calm down the chat")
                    .color(Color::FOOYOO)
                })
            })
            .await?;
            return Ok(());
        }
    } else {
        // Create list of voted people for the target and add the caller
        users.insert(
            target_user,
            (
                Utc::now().timestamp() + Duration::minutes(5).num_seconds(),
                vec![calling_user],
            ),
        );
    }
    ctx.send(|b| {
        b.embed(|e| {
            e.title(format!(
                "{} of {} users voted to mute {} for {} minutes",
                users.get(&target_user).unwrap().1.len(),
                required_users,
                target_user_m.user.tag(),
                TIMEOUT_DURATION
            ))
            .description(format!(
                "Use ``/votemute {}`` or ``ttc!votemute {}`` to vote too",
                target_user_m.user.tag(),
                target_user_m.user.tag()
            ))
            .color(Color::BLITZ_BLUE)
        })
    })
    .await?;

    Ok(())
}
