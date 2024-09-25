# Bacchus-Serene
*a discord bot to manage events on community servers*

## Available commands
(to see how to launch the bot, please refer to `README.md`)

- `/init`
> Must be called once when the bot joins.
> It will make the bot create a role, "Menad", which allows people to create events.
> The role can be renamed but should not be deleted.
> If it gets deleted, just call `/init` again

- `/event`
  - `create [name] [short_description?] [description?] [thumbnail?] [picture?]`
  > Creates a new event, with the given parameters.
  > Sends an embed message with information about the event
  > Reacting to the embed allows people to join the event
  > People can also be added/removed forcefully using `/event member add [user]`
  > Creates a category and text channel that can only be accessed by those participating in the event.
  >
  > NOTE: PLEASE DO NOT DELETE EVENT CHANNELS / CATEGORIES / ROLES BY HAND

  - `delete`
  > Must be run in an event-managed channel.
  > Deletes the event, its category, channels, and embed message.
  > Requires to be the creator of the event to be run.
  > Note : event managers can be added to an event using `/event member add_manager [user]`

- `member`
  > Allows to manage event members
  > Must be run in an event-managed channel
  - `add [user]`
  > Adds a user to the current event
  - `remove [user]`
  > Removes a user from the current event
  > NOTE: MANAGERS CANNOT BE REMOVED FROM AN EVENT, ONLY REGULAR USERS CAN
  - `add_manager [user]`
  > Adds the given user as a manager
  > NOTE : MANAGERS CANNOT BE REMOVED FROM AN EVENT (to avoid hostile takeovers)
