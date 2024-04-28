# HEY!
this name is kinda lame, eh?
## Current progress
### models:
i think it's kinda settled here, not much drastic change maybe?
- user model: kinda done
- message model: kinda done

### state:
there are some areas that need a lot of improvement or even overhaul
- rooms manager: maybe need better implementation on how to handle room connection, some sort of `channel::broadcast` maybe? so i can do a subscribe like normal pub-sub
- db: kinda done
- auth: kinda done, maybe some activation token?
- error: TODO => create separate error crate

### app:
this one here too
- ui layout: kinda ok but it looks to much like a `discord.clone()`
- ui esthetics: meh, needs a lot of improvement
- color scheme: TODO => not yet implemented
- dynamic display: not yet, but it's better to create separate app for that i guess?
- current user data: maybe make it available on the top parent level?
- real time msg handling: currently handled by websocket which is established from each channel, i don't think this is a good idea, maybe move the connection into top parent level, and then iterate the channel based on received data from ws message?
- virtual list: (or infinite scroll) => not yet implemented
- message fetching: need some improvement to support virtual list / infinite scroll
- image, voice, file data: TODO => not yet implemented
