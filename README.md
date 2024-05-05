# HEY!
this name is kinda lame, eh?
## Current progress
### models:
i think it's kinda settled here, not much drastic change maybe?
- user model: kinda done
- message model: kinda done

### state:
there are some areas that need a lot of improvement or even overhaul
- rooms manager: added `broadcast::channel` to inform if a user has created/joined a channel
- db: kinda done
- auth: kinda done, maybe some activation token?
- error: done

### app:
this one here too
- ui layout: kinda ok but it looks to much like a `discord.clone()`
- ui esthetics: meh, needs a lot of improvement
- color scheme: TODO => not yet implemented
- dynamic display: not yet, but it's better to create client app for that i guess?
- real time msg handling: the websocket connection is now on the top level
- virtual list: (or infinite scroll) => not yet implemented
- message fetching: need some improvement to support virtual list / infinite scroll
- image, voice, file data: TODO => not yet implemented
