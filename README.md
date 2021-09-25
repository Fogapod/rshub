# RSHub
Unofficial game launcher for UnityStation.

RSHub is a terminal application written in Rust for listing and connecting to UnityStation game servers.
UnityStation is a modern remake of Space Station 13 in Unity.

![Server List](.assets/screenshots/servers.png)

### Feature comparison to official Hub
| feature | RSHub | StationHub |
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
| ian icon | no | [broken](https://github.com/unitystation/stationhub/issues/111) |

[1] I am not planning to support firebase auth, unitystation is switching to their own provider.

### Platform support
- Linux: developed and tested on.
- Windows: seem to work, though I do not have machine to test it.
- Mac: it might work, but most likely it would not because of platform limitations. I have no machine to test it.

### Installation
Prebuilt binaries can be found in [releases](https://github.com/Fogapod/rshub/releases).

RSHub requires nightly rust toolchain to build (unstable strip feature).
If you do not have nightly toolchain, you can prefix cargo commands with `RUSTC_BOOTSTRAP=1 cargo ...` as a workaround.

Install from crates.io (stable version): `cargo install rshub`  
Or with [geolocation](#geolocation) feature: `cargo install rshub --feature geolocation`

Run from source (latest version):
`cargo run` or `cargo run --release` (slow)

### Usage
- Use `--help` to get CLI usage.
- Press F1 on any screen to show hotkeys.

### Issues
Possible problems and fixes:
- Linux, i3 specific: game starts in fullscreen in bad resolution. Solution: uncheck fullscreen mode in game settings.
- rshub 0.1.5 only supports servers of version UnityStationDevelop-21092504 and later because of auth changes. If you need to connect to older builds, you must use rshub 0.1.4.

### Geolocation
Currently geolocation feature (world map) is opt-in at compile time because of security cencerns.
You will have to add `--features geolocation` to cargo commands to enable it.
This is because service I use for geolocation has too high ratelimits and I had to set up my own instance.  
While solves ratelimits problem, it lets me gather IP addresses (and locations) of hub users, so I made it strictly opt-in.

### Special thanks (random order)
- PotatoAlienOf13: for testing and suggestions during initial development and original [idea](https://github.com/PotatoAlienOf13/not-station-hub)
- kalmari: for answering Rust questions
- [gitui](https://github.com/extrawurst/gitui) and [bottom](https://github.com/ClementTsang/bottom) for inspiration
- Unitystation developers for the game

### More screenshots
![Commits](.assets/screenshots/commits.png)
![Download](.assets/screenshots/download.png)
![Shortcuts](.assets/screenshots/shortcuts.png)
![World](.assets/screenshots/world.png)
