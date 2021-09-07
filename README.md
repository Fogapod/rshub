# RStationHub
Unofficial game launcher for UnityStation.

RStationHub is a terminal application written in Rust for listing and connecting to UnityStation game servers.
UnityStation is a modern remake of Space Station 13 in Unity.

### Feature comparison to official Hub
| feature | RStationHub | StationHub |
|:---:|:---:|:---:|
| auth[1] | no | yes |
| list online servers | yes | yes |
| connect to server | yes | yes |
| ping servers | no | yes |
| manage installations (add/remove) | yes | yes |
| run local installation | yes | [broken](https://github.com/unitystation/stationhub/issues/128) |
| news/commits section | yes | yes |
| show servers on map (useless) | yes | no |
| written in rust | yes | no |

[1]: even though launching game without auth has major inconveniences, (refer to #Issues), I am not planning to support firebase (unitystation is switching to their own provider)

### Usage
- Use `--help` to get CLI usage.
- Press F1 on any screen to show hotkeys.

### Platform support
- Linux: developed and tested on.
- Windows: might work out of the box, but I have no Windows machine to test it.
- Mac: it might work, but most likely it would not because of platform limitations. I have no machine to test it.

### Issues
There are multiple issues using RStationHub currently:
- Linux, i3 specific: game starts in fullscreen in bad resolution. Solution: uncheck fullscreen mode in game settings.
- You will get auth error when connecting to server. This is because of a workaround for this bug: https://github.com/unitystation/unitystation/issues/7375
- When connecting to server, you will have to uncheck `Host Server` checkbox because of this bug: https://github.com/unitystation/unitystation/issues/7376
- When connecting to server, you will have to enter your password each time. Autologin is broken: https://github.com/unitystation/unitystation/issues/7377

### Installation
There are no prebuilt binaries yet, you can run RStationHub using cargo:
`cargo run` or `cargo run --release` (slow)

### Geolocation
Currently geolocation feature (world map) is opt-in at compile time because of security cencerns.
You will have to run `cargo install --path . --features geolocation` to enable it.

### Special thanks (random order)
- PotatoAlienOf13: for testing and suggestions during initial development and original [idea](https://github.com/PotatoAlienOf13/not-station-hub)
- kalmari: for answering Rust questions
- [gitui](https://github.com/extrawurst/gitui) and [bottom](https://github.com/ClementTsang/bottom) for inspiration
- Unitystation developers for the game
